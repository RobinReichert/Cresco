use crate::storage::ConcreteStorage;
use crate::{blackboard, probe::AnalogProbe};
use defmt::{debug, info, panic, warn};
use embassy_futures::select::{Either, select};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, mutex::Mutex, signal::Signal,
};
use embassy_time::{Duration, Instant, Timer};
use esp_hal::peripherals::{GPIO0, GPIO1};
use logic::{
    Float,
    measurement::{MeasurementAction, MeasurementError, MeasurementEvent, MeasurementManager},
};

pub type ConcretePhProbe<'a> = AnalogProbe<'a, GPIO0<'a>>;
pub type ConcreteEcProbe<'a> = AnalogProbe<'a, GPIO1<'a>>;

const MEASUREMENT_PERIOD: Duration = Duration::from_secs(60);

pub enum MeasurementCommand {
    Abort,
    StartEcCalibration,
    StartPhCalibration,
    FinishCalibration,
    EcMeasurement { actual_ec: Float },
    PhMeasurement { actual_ph: Float },
}

#[derive(Clone, Copy, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MeasurementStatus {
    NotCalibratedYet,
    AwaitingFirstPoint,
    AwaitingSecondPoint,
    ProbeReadFailed,
    CalibrationFailed,
    CalibrationSucceeded,
    CalibrationAborted,
}

pub type StatusCell = Mutex<CriticalSectionRawMutex, MeasurementStatus>;

#[embassy_executor::task]
pub async fn measurement_task(
    mut ph_probe: ConcretePhProbe<'static>,
    mut ec_probe: ConcreteEcProbe<'static>,
    commands: &'static Channel<CriticalSectionRawMutex, MeasurementCommand, 1>,
    status: &'static StatusCell,
    storage: &'static ConcreteStorage,
    fresh: &'static Signal<CriticalSectionRawMutex, u64>,
) {
    info!("measurement: task start");
    let ph = storage.get_ph_calibration().await.ok();
    let ec = storage.get_ec_calibration().await.ok();
    let mut manager = MeasurementManager::with_calibrations(ec, ph);
    let mut event: Option<MeasurementEvent> = Some(MeasurementEvent::Start);
    loop {
        match manager.handle_event(event.take().expect("no event: this should not happen")) {
            MeasurementAction::WaitForNext => {
                event = Some(next_trigger(commands).await);
            }
            MeasurementAction::StoreEcCalibration { calibration } => {
                info!("measurement: store ec calibration");
                if !storage.set_ec_calibration(calibration).await {
                    warn!("measurement: store ec calibration failed");
                }
                event = Some(MeasurementEvent::CalibrationStored);
            }
            MeasurementAction::StorePhCalibration { calibration } => {
                info!("measurement: store ph calibration");
                if !storage.set_ph_calibration(calibration).await {
                    warn!("measurement: store ph calibration failed");
                }
                event = Some(MeasurementEvent::CalibrationStored);
            }
            MeasurementAction::MeasureEc => {
                debug!("measurement: measure ec");
                event = Some(match ec_probe.read().await {
                    Ok(ec) => {
                        debug!("measurement: raw ec: {}", ec);
                        MeasurementEvent::EcMeasured { ec }
                    }
                    Err(e) => {
                        warn!("measurement: ec measurement failed: {}", e);
                        MeasurementEvent::Abort
                    }
                })
            }
            MeasurementAction::MeasurePh => {
                debug!("measurement: measure ph");
                event = Some(match ph_probe.read().await {
                    Ok(ph) => {
                        debug!("measurement: raw ph: {}", ph);
                        MeasurementEvent::PhMeasured { ph }
                    }
                    Err(e) => {
                        warn!("measurement: ph measurement failed: {}", e);
                        MeasurementEvent::Abort
                    }
                })
            }
            MeasurementAction::WriteMeasurements { ec, ph } => {
                info!("measurement: write ph: {} and ec: {} to blackboard", ph, ec);
                blackboard::set_ph(ph).await;
                blackboard::set_ec(ec).await;
                fresh.signal(Instant::now().as_millis());
                event = Some(next_trigger(commands).await);
            }
            MeasurementAction::ShowError { error } => {
                *status.lock().await = match error {
                    MeasurementError::NotCalibratedYet => MeasurementStatus::NotCalibratedYet,
                    MeasurementError::CalibrationFailed => MeasurementStatus::CalibrationFailed,
                };
                event = Some(next_trigger(commands).await);
            }
            MeasurementAction::RetrieveEcFirst => {
                *status.lock().await = MeasurementStatus::AwaitingFirstPoint;
                debug!("measurement: wait for first ec value");
                event = Some(
                    if let MeasurementCommand::EcMeasurement { actual_ec } =
                        commands.receive().await
                    {
                        debug!(
                            "measurement: measure raw ec that corresponds to actual ec: {}",
                            actual_ec
                        );
                        match ec_probe.read().await {
                            Ok(ec) => {
                                info!(
                                    "measurement: raw ec corresponding to actual ec: {} is: {}",
                                    actual_ec, ec
                                );
                                MeasurementEvent::FirstMeasured {
                                    first: ec,
                                    actual_first: actual_ec,
                                }
                            }
                            Err(e) => {
                                warn!("measurement: ec measurement failed: {}", e);
                                *status.lock().await = MeasurementStatus::ProbeReadFailed;
                                info!("measurement: abort calibration");
                                MeasurementEvent::Abort
                            }
                        }
                    } else {
                        debug!("measurement: non ec value received");
                        *status.lock().await = MeasurementStatus::CalibrationAborted;
                        info!("measurement: abort calibration");
                        MeasurementEvent::Abort
                    },
                )
            }
            MeasurementAction::RetrieveEcSecond => {
                *status.lock().await = MeasurementStatus::AwaitingSecondPoint;
                debug!("measurement: wait for second ec value");
                event = Some(
                    if let MeasurementCommand::EcMeasurement { actual_ec } =
                        commands.receive().await
                    {
                        debug!(
                            "measurement: measure raw ec that corresponds to actual ec: {}",
                            actual_ec
                        );
                        match ec_probe.read().await {
                            Ok(ec) => {
                                info!(
                                    "measurement: raw ec corresponding to actual ec: {} is: {}",
                                    actual_ec, ec
                                );
                                MeasurementEvent::SecondMeasured {
                                    second: ec,
                                    actual_second: actual_ec,
                                }
                            }
                            Err(e) => {
                                warn!("measurement: ec measurement failed: {}", e);
                                *status.lock().await = MeasurementStatus::ProbeReadFailed;
                                info!("measurement: abort calibration");
                                MeasurementEvent::Abort
                            }
                        }
                    } else {
                        debug!("measurement: non ec value received");
                        *status.lock().await = MeasurementStatus::CalibrationAborted;
                        info!("measurement: abort calibration");
                        MeasurementEvent::Abort
                    },
                )
            }
            MeasurementAction::RetrievePhFirst => {
                *status.lock().await = MeasurementStatus::AwaitingFirstPoint;
                debug!("measurement: wait for first ph value");
                event = Some(
                    if let MeasurementCommand::PhMeasurement { actual_ph } =
                        commands.receive().await
                    {
                        debug!(
                            "measurement: measure raw ph that corresponds to actual ph: {}",
                            actual_ph
                        );
                        match ph_probe.read().await {
                            Ok(ph) => {
                                info!(
                                    "measurement: raw ph corresponding to actual ph: {} is: {}",
                                    actual_ph, ph
                                );
                                MeasurementEvent::FirstMeasured {
                                    first: ph,
                                    actual_first: actual_ph,
                                }
                            }
                            Err(e) => {
                                warn!("measurement: ph measurement failed: {}", e);
                                *status.lock().await = MeasurementStatus::ProbeReadFailed;
                                info!("measurement: abort calibration");
                                MeasurementEvent::Abort
                            }
                        }
                    } else {
                        debug!("measurement: non ph value received");
                        *status.lock().await = MeasurementStatus::CalibrationAborted;
                        info!("measurement: abort calibration");
                        MeasurementEvent::Abort
                    },
                )
            }
            MeasurementAction::RetrievePhSecond => {
                *status.lock().await = MeasurementStatus::AwaitingSecondPoint;
                debug!("measurement: wait for second ph value");
                event = Some(
                    if let MeasurementCommand::PhMeasurement { actual_ph } =
                        commands.receive().await
                    {
                        debug!(
                            "measurement: measure raw ph that corresponds to actual ph: {}",
                            actual_ph
                        );
                        match ph_probe.read().await {
                            Ok(ph) => {
                                info!(
                                    "measurement: raw ph corresponding to actual ph: {} is: {}",
                                    actual_ph, ph
                                );
                                MeasurementEvent::SecondMeasured {
                                    second: ph,
                                    actual_second: actual_ph,
                                }
                            }
                            Err(e) => {
                                warn!("measurement: ph measurement failed: {:?}", e);
                                *status.lock().await = MeasurementStatus::ProbeReadFailed;
                                info!("measurement: abort calibration");
                                MeasurementEvent::Abort
                            }
                        }
                    } else {
                        debug!("measurement: non ph value received");
                        *status.lock().await = MeasurementStatus::CalibrationAborted;
                        info!("measurement: abort calibration");
                        MeasurementEvent::Abort
                    },
                )
            }
            MeasurementAction::WaitForFinish => {
                *status.lock().await = MeasurementStatus::CalibrationSucceeded;
                debug!("measurement: wait for finish calibration command");
                event = Some(match commands.receive().await {
                    MeasurementCommand::FinishCalibration => {
                        info!("measurement: finish calibration");
                        MeasurementEvent::FinishCalibration
                    }
                    _ => {
                        debug!("measurement: different command than finish calibration received");
                        *status.lock().await = MeasurementStatus::CalibrationAborted;
                        info!("measurement: abort calibration");
                        MeasurementEvent::Abort
                    }
                });
            }
            MeasurementAction::Ignore => {
                panic!("invalid state, event combination");
            }
        }
    }
}

async fn next_trigger(
    commands: &'static Channel<CriticalSectionRawMutex, MeasurementCommand, 1>,
) -> MeasurementEvent {
    let deadline = Instant::now() + MEASUREMENT_PERIOD;
    debug!("measurement: wait for next trigger");
    loop {
        match select(Timer::at(deadline), commands.receive()).await {
            Either::First(_) => return MeasurementEvent::StartMeasurement,
            Either::Second(MeasurementCommand::StartEcCalibration) => {
                return MeasurementEvent::StartEcCalibration;
            }
            Either::Second(MeasurementCommand::StartPhCalibration) => {
                return MeasurementEvent::StartPhCalibration;
            }
            Either::Second(MeasurementCommand::Abort) => return MeasurementEvent::Abort,
            Either::Second(_) => warn!("measurement: something went wrong"),
        }
    }
}
