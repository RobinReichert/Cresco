use crate::{blackboard, probe::AnalogProbe};
use crate::{shared_flash::SharedFlashInterface, storage::CalibrationStorage};
use defmt::{info, panic};
use embassy_embedded_hal::adapter::BlockingAsync;
use embassy_futures::select::{Either, select};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, mutex::Mutex};
use embassy_time::{Duration, Instant, Timer};
use esp_hal::peripherals::{GPIO0, GPIO1};
use esp_storage::FlashStorage;
use logic::{
    Float,
    calibration::Linear,
    measurement::{MeasurementAction, MeasurementError, MeasurementEvent, MeasurementManager},
};
use sequential_storage::cache::NoCache;

pub type ConcretePhProbe<'a> = AnalogProbe<'a, GPIO0<'a>>;
pub type ConcreteEcProbe<'a> = AnalogProbe<'a, GPIO1<'a>>;

const MEASUREMENT_PERIOD: Duration = Duration::from_secs(60);

pub enum MeasurementCommand {
    Abort,
    StartEcCalibration,
    StartPhCalibration,
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
pub type ConcreteCalibrationStorage<'a> =
    CalibrationStorage<SharedFlashInterface<'a, BlockingAsync<FlashStorage<'static>>>, NoCache>;

#[embassy_executor::task]
pub async fn measurement_task(
    mut ph_probe: ConcretePhProbe<'static>,
    mut ec_probe: ConcreteEcProbe<'static>,
    commands: &'static Channel<CriticalSectionRawMutex, MeasurementCommand, 1>,
    status: &'static StatusCell,
    mut calibration_storage: ConcreteCalibrationStorage<'static>,
) {
    let ph = calibration_storage
        .get_ph_calibration()
        .await
        .unwrap_or_else(|_| Linear::new());
    let ec = calibration_storage
        .get_ec_calibration()
        .await
        .unwrap_or_else(|_| Linear::new());
    let mut manager = MeasurementManager::with_calibrations(ec, ph);
    let mut event: Option<MeasurementEvent> = Some(MeasurementEvent::Start);
    let mut calibrating = false;
    loop {
        match manager.handle_event(event.take().expect("no event: this should not happen")) {
            MeasurementAction::WaitForNext => {
                if calibrating {
                    calibrating = false;
                    *status.lock().await = MeasurementStatus::CalibrationSucceeded;
                }
                event = Some(next_trigger(commands).await);
            }
            MeasurementAction::MeasureEc => {
                event = Some(match ec_probe.read().await {
                    Ok(ec) => MeasurementEvent::EcMeasured { ec },
                    Err(_) => MeasurementEvent::Abort,
                })
            }
            MeasurementAction::MeasurePh => {
                event = Some(match ph_probe.read().await {
                    Ok(ph) => MeasurementEvent::PhMeasured { ph },
                    Err(_) => MeasurementEvent::Abort,
                })
            }
            MeasurementAction::WriteMeasurements { ec, ph } => {
                blackboard::set_ph(ph).await;
                blackboard::set_ec(ec).await;
                event = Some(next_trigger(commands).await);
            }
            MeasurementAction::ShowError { error } => {
                calibrating = false;
                *status.lock().await = match error {
                    MeasurementError::NotCalibratedYet => MeasurementStatus::NotCalibratedYet,
                    MeasurementError::CalibrationFailed => MeasurementStatus::CalibrationFailed,
                };
                event = Some(next_trigger(commands).await);
            }
            MeasurementAction::RetrieveEcFirst => {
                calibrating = true;
                *status.lock().await = MeasurementStatus::AwaitingFirstPoint;
                event = Some(
                    if let MeasurementCommand::EcMeasurement { actual_ec } =
                        commands.receive().await
                    {
                        match ec_probe.read().await {
                            Ok(ec) => MeasurementEvent::FirstMeasured {
                                first: ec,
                                actual_first: actual_ec,
                            },
                            Err(_) => {
                                calibrating = false;
                                *status.lock().await = MeasurementStatus::ProbeReadFailed;
                                MeasurementEvent::Abort
                            }
                        }
                    } else {
                        calibrating = false;
                        *status.lock().await = MeasurementStatus::CalibrationAborted;
                        MeasurementEvent::Abort
                    },
                )
            }
            MeasurementAction::RetrieveEcSecond => {
                *status.lock().await = MeasurementStatus::AwaitingSecondPoint;
                event = Some(
                    if let MeasurementCommand::EcMeasurement { actual_ec } =
                        commands.receive().await
                    {
                        match ec_probe.read().await {
                            Ok(ec) => MeasurementEvent::SecondMeasured {
                                second: ec,
                                actual_second: actual_ec,
                            },
                            Err(_) => {
                                calibrating = false;
                                *status.lock().await = MeasurementStatus::ProbeReadFailed;
                                MeasurementEvent::Abort
                            }
                        }
                    } else {
                        calibrating = false;
                        *status.lock().await = MeasurementStatus::CalibrationAborted;
                        MeasurementEvent::Abort
                    },
                )
            }
            MeasurementAction::RetrievePhFirst => {
                calibrating = true;
                *status.lock().await = MeasurementStatus::AwaitingFirstPoint;
                event = Some(
                    if let MeasurementCommand::PhMeasurement { actual_ph } =
                        commands.receive().await
                    {
                        match ph_probe.read().await {
                            Ok(ph) => MeasurementEvent::FirstMeasured {
                                first: ph,
                                actual_first: actual_ph,
                            },
                            Err(_) => {
                                calibrating = false;
                                *status.lock().await = MeasurementStatus::ProbeReadFailed;
                                MeasurementEvent::Abort
                            }
                        }
                    } else {
                        calibrating = false;
                        *status.lock().await = MeasurementStatus::CalibrationAborted;
                        MeasurementEvent::Abort
                    },
                )
            }
            MeasurementAction::RetrievePhSecond => {
                *status.lock().await = MeasurementStatus::AwaitingSecondPoint;
                event = Some(
                    if let MeasurementCommand::PhMeasurement { actual_ph } =
                        commands.receive().await
                    {
                        match ph_probe.read().await {
                            Ok(ph) => MeasurementEvent::SecondMeasured {
                                second: ph,
                                actual_second: actual_ph,
                            },
                            Err(_) => {
                                calibrating = false;
                                *status.lock().await = MeasurementStatus::ProbeReadFailed;
                                MeasurementEvent::Abort
                            }
                        }
                    } else {
                        calibrating = false;
                        *status.lock().await = MeasurementStatus::CalibrationAborted;
                        MeasurementEvent::Abort
                    },
                )
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
            Either::Second(_) => info!("something went wrong"),
        }
    }
}
