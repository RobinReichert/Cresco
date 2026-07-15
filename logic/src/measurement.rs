use crate::{
    Float,
    calibration::{self, Point},
};

pub enum MeasurementState {
    Start,
    Idle,
    MeasuringEc,
    MeasuringPh { ec: Float },
    CalibratingEcFirst,
    CalibratingEcSecond { first: Float, actual_first: Float },
    CalibratingPhFirst,
    CalibratingPhSecond { first: Float, actual_first: Float },
}

pub enum MeasurementEvent {
    Start,
    StartMeasurement,
    StartEcCalibration,
    StartPhCalibration,
    FirstMeasured { first: Float, actual_first: Float },
    SecondMeasured { second: Float, actual_second: Float },
    Abort,
    EcMeasured { ec: Float },
    PhMeasured { ph: Float },
}

#[derive(Debug, PartialEq)]
pub enum MeasurementError {
    NotCalibratedYet,
    CalibrationFailed,
}

#[derive(Debug, PartialEq)]
pub enum MeasurementAction {
    Ignore,
    MeasureEc,
    MeasurePh,
    WriteMeasurements { ec: Float, ph: Float },
    ShowError { error: MeasurementError },
    RetrieveEcFirst,
    RetrieveEcSecond,
    RetrievePhFirst,
    RetrievePhSecond,
    WaitForNext,
}

pub struct MeasurementManager {
    state: MeasurementState,
    ec_calibration: calibration::Linear,
    ph_calibration: calibration::Linear,
}

impl MeasurementManager {
    pub fn new() -> Self {
        let ec_calibration = calibration::Linear::new();
        let ph_calibration = calibration::Linear::new();
        Self {
            state: MeasurementState::Start,
            ec_calibration,
            ph_calibration,
        }
    }

    pub fn handle_event(&mut self, event: MeasurementEvent) -> MeasurementAction {
        match (&self.state, event) {
            (MeasurementState::Start, MeasurementEvent::Start) => {
                self.state = MeasurementState::Idle;
                MeasurementAction::WaitForNext
            }
            (MeasurementState::Idle, MeasurementEvent::StartMeasurement) => {
                if self.ec_calibration.is_calibrated() && self.ph_calibration.is_calibrated() {
                    self.state = MeasurementState::MeasuringEc;
                    MeasurementAction::MeasureEc
                } else {
                    MeasurementAction::ShowError {
                        error: MeasurementError::NotCalibratedYet,
                    }
                }
            }
            (MeasurementState::Idle, MeasurementEvent::Abort) => MeasurementAction::WaitForNext,
            (MeasurementState::MeasuringEc, MeasurementEvent::EcMeasured { ec }) => {
                self.state = MeasurementState::MeasuringPh { ec };
                MeasurementAction::MeasurePh
            }
            (MeasurementState::MeasuringEc, MeasurementEvent::Abort) => {
                self.state = MeasurementState::Idle;
                MeasurementAction::WaitForNext
            }
            (MeasurementState::MeasuringPh { ec }, MeasurementEvent::PhMeasured { ph }) => {
                let ec = *ec;
                self.state = MeasurementState::Idle;
                match (self.ec_calibration.apply(ec), self.ph_calibration.apply(ph)) {
                    (Ok(calibrated_ec), Ok(calibrated_ph)) => {
                        MeasurementAction::WriteMeasurements {
                            ec: calibrated_ec,
                            ph: calibrated_ph,
                        }
                    }
                    (_, _) => MeasurementAction::ShowError {
                        error: MeasurementError::NotCalibratedYet,
                    },
                }
            }
            (MeasurementState::MeasuringPh { .. }, MeasurementEvent::Abort) => {
                self.state = MeasurementState::Idle;
                MeasurementAction::WaitForNext
            }
            (MeasurementState::Idle, MeasurementEvent::StartEcCalibration) => {
                self.state = MeasurementState::CalibratingEcFirst;
                MeasurementAction::RetrieveEcFirst
            }
            (
                MeasurementState::CalibratingEcFirst,
                MeasurementEvent::FirstMeasured {
                    first,
                    actual_first,
                },
            ) => {
                self.state = MeasurementState::CalibratingEcSecond {
                    first,
                    actual_first,
                };
                MeasurementAction::RetrieveEcSecond
            }
            (MeasurementState::CalibratingEcFirst, MeasurementEvent::Abort) => {
                self.state = MeasurementState::Idle;
                MeasurementAction::WaitForNext
            }
            (
                MeasurementState::CalibratingEcSecond {
                    first,
                    actual_first,
                },
                MeasurementEvent::SecondMeasured {
                    second,
                    actual_second,
                },
            ) => {
                let first = *first;
                let actual_first = *actual_first;
                self.state = MeasurementState::Idle;
                match self.ec_calibration.calibrate(
                    Point {
                        y: actual_first,
                        x: first,
                    },
                    Point {
                        y: actual_second,
                        x: second,
                    },
                ) {
                    Ok(_) => MeasurementAction::WaitForNext,
                    Err(_) => MeasurementAction::ShowError {
                        error: MeasurementError::CalibrationFailed,
                    },
                }
            }
            (MeasurementState::CalibratingEcSecond { .. }, MeasurementEvent::Abort) => {
                self.state = MeasurementState::Idle;
                MeasurementAction::WaitForNext
            }
            (MeasurementState::Idle, MeasurementEvent::StartPhCalibration) => {
                self.state = MeasurementState::CalibratingPhFirst;
                MeasurementAction::RetrievePhFirst
            }
            (
                MeasurementState::CalibratingPhFirst,
                MeasurementEvent::FirstMeasured {
                    first,
                    actual_first,
                },
            ) => {
                self.state = MeasurementState::CalibratingPhSecond {
                    first,
                    actual_first,
                };
                MeasurementAction::RetrievePhSecond
            }
            (MeasurementState::CalibratingPhFirst, MeasurementEvent::Abort) => {
                self.state = MeasurementState::Idle;
                MeasurementAction::WaitForNext
            }
            (
                MeasurementState::CalibratingPhSecond {
                    first,
                    actual_first,
                },
                MeasurementEvent::SecondMeasured {
                    second,
                    actual_second,
                },
            ) => {
                let first = *first;
                let actual_first = *actual_first;
                self.state = MeasurementState::Idle;
                match self.ph_calibration.calibrate(
                    Point {
                        y: actual_first,
                        x: first,
                    },
                    Point {
                        y: actual_second,
                        x: second,
                    },
                ) {
                    Ok(_) => MeasurementAction::WaitForNext,
                    Err(_) => MeasurementAction::ShowError {
                        error: MeasurementError::CalibrationFailed,
                    },
                }
            }
            (MeasurementState::CalibratingPhSecond { .. }, MeasurementEvent::Abort) => {
                self.state = MeasurementState::Idle;
                MeasurementAction::WaitForNext
            }
            (_, _) => MeasurementAction::Ignore,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn started_manager() -> MeasurementManager {
        let mut manager = MeasurementManager::new();
        assert_eq!(
            manager.handle_event(MeasurementEvent::Start),
            MeasurementAction::WaitForNext
        );
        manager
    }

    // ec: raw 0.0 -> 0.0, raw 10.0 -> 100.0 (slope 10, intercept 0)
    // ph: raw 0.0 -> 7.0, raw 10.0 -> 17.0 (slope 1, intercept 7)
    fn calibrated_manager() -> MeasurementManager {
        let mut manager = started_manager();
        assert_eq!(
            manager.handle_event(MeasurementEvent::StartEcCalibration),
            MeasurementAction::RetrieveEcFirst
        );
        assert_eq!(
            manager.handle_event(MeasurementEvent::FirstMeasured {
                first: 0.0,
                actual_first: 0.0,
            }),
            MeasurementAction::RetrieveEcSecond
        );
        assert_eq!(
            manager.handle_event(MeasurementEvent::SecondMeasured {
                second: 10.0,
                actual_second: 100.0,
            }),
            MeasurementAction::WaitForNext
        );
        assert_eq!(
            manager.handle_event(MeasurementEvent::StartPhCalibration),
            MeasurementAction::RetrievePhFirst
        );
        assert_eq!(
            manager.handle_event(MeasurementEvent::FirstMeasured {
                first: 0.0,
                actual_first: 7.0,
            }),
            MeasurementAction::RetrievePhSecond
        );
        assert_eq!(
            manager.handle_event(MeasurementEvent::SecondMeasured {
                second: 10.0,
                actual_second: 17.0,
            }),
            MeasurementAction::WaitForNext
        );
        manager
    }

    #[test]
    fn test_start_transitions_to_idle_and_waits() {
        started_manager();
    }

    #[test]
    fn test_abort_while_idle_is_ignored_safely() {
        let mut manager = started_manager();
        assert_eq!(
            manager.handle_event(MeasurementEvent::Abort),
            MeasurementAction::WaitForNext
        );
    }

    #[test]
    fn test_start_measurement_when_uncalibrated_shows_error() {
        let mut manager = started_manager();
        assert_eq!(
            manager.handle_event(MeasurementEvent::StartMeasurement),
            MeasurementAction::ShowError {
                error: MeasurementError::NotCalibratedYet,
            }
        );
    }

    #[test]
    fn test_start_measurement_when_calibrated_measures_ec() {
        let mut manager = calibrated_manager();
        assert_eq!(
            manager.handle_event(MeasurementEvent::StartMeasurement),
            MeasurementAction::MeasureEc
        );
    }

    #[test]
    fn test_ec_measured_transitions_to_measuring_ph() {
        let mut manager = calibrated_manager();
        manager.handle_event(MeasurementEvent::StartMeasurement);
        assert_eq!(
            manager.handle_event(MeasurementEvent::EcMeasured { ec: 5.0 }),
            MeasurementAction::MeasurePh
        );
    }

    #[test]
    fn test_abort_during_measuring_ec_returns_to_idle() {
        let mut manager = calibrated_manager();
        manager.handle_event(MeasurementEvent::StartMeasurement);
        assert_eq!(
            manager.handle_event(MeasurementEvent::Abort),
            MeasurementAction::WaitForNext
        );
    }

    #[test]
    fn test_ph_measured_writes_calibrated_measurements() {
        let mut manager = calibrated_manager();
        manager.handle_event(MeasurementEvent::StartMeasurement);
        manager.handle_event(MeasurementEvent::EcMeasured { ec: 5.0 });
        assert_eq!(
            manager.handle_event(MeasurementEvent::PhMeasured { ph: 5.0 }),
            MeasurementAction::WriteMeasurements { ec: 50.0, ph: 12.0 }
        );
    }

    #[test]
    fn test_abort_during_measuring_ph_returns_to_idle() {
        let mut manager = calibrated_manager();
        manager.handle_event(MeasurementEvent::StartMeasurement);
        manager.handle_event(MeasurementEvent::EcMeasured { ec: 5.0 });
        assert_eq!(
            manager.handle_event(MeasurementEvent::Abort),
            MeasurementAction::WaitForNext
        );
    }

    #[test]
    fn test_start_ec_calibration_from_idle() {
        let mut manager = started_manager();
        assert_eq!(
            manager.handle_event(MeasurementEvent::StartEcCalibration),
            MeasurementAction::RetrieveEcFirst
        );
    }

    #[test]
    fn test_first_measured_during_ec_calibration_retrieves_second() {
        let mut manager = started_manager();
        manager.handle_event(MeasurementEvent::StartEcCalibration);
        assert_eq!(
            manager.handle_event(MeasurementEvent::FirstMeasured {
                first: 0.0,
                actual_first: 0.0,
            }),
            MeasurementAction::RetrieveEcSecond
        );
    }

    #[test]
    fn test_abort_during_ec_calibration_first_returns_to_idle() {
        let mut manager = started_manager();
        manager.handle_event(MeasurementEvent::StartEcCalibration);
        assert_eq!(
            manager.handle_event(MeasurementEvent::Abort),
            MeasurementAction::WaitForNext
        );
    }

    #[test]
    fn test_second_measured_completes_ec_calibration_successfully() {
        let mut manager = started_manager();
        manager.handle_event(MeasurementEvent::StartEcCalibration);
        manager.handle_event(MeasurementEvent::FirstMeasured {
            first: 0.0,
            actual_first: 0.0,
        });
        assert_eq!(
            manager.handle_event(MeasurementEvent::SecondMeasured {
                second: 10.0,
                actual_second: 100.0,
            }),
            MeasurementAction::WaitForNext
        );
    }

    #[test]
    fn test_second_measured_fails_ec_calibration_when_readings_identical() {
        let mut manager = started_manager();
        manager.handle_event(MeasurementEvent::StartEcCalibration);
        manager.handle_event(MeasurementEvent::FirstMeasured {
            first: 5.0,
            actual_first: 0.0,
        });
        assert_eq!(
            manager.handle_event(MeasurementEvent::SecondMeasured {
                second: 5.0,
                actual_second: 100.0,
            }),
            MeasurementAction::ShowError {
                error: MeasurementError::CalibrationFailed,
            }
        );
    }

    #[test]
    fn test_abort_during_ec_calibration_second_returns_to_idle() {
        let mut manager = started_manager();
        manager.handle_event(MeasurementEvent::StartEcCalibration);
        manager.handle_event(MeasurementEvent::FirstMeasured {
            first: 0.0,
            actual_first: 0.0,
        });
        assert_eq!(
            manager.handle_event(MeasurementEvent::Abort),
            MeasurementAction::WaitForNext
        );
    }

    #[test]
    fn test_start_ph_calibration_from_idle() {
        let mut manager = started_manager();
        assert_eq!(
            manager.handle_event(MeasurementEvent::StartPhCalibration),
            MeasurementAction::RetrievePhFirst
        );
    }

    #[test]
    fn test_first_measured_during_ph_calibration_retrieves_second() {
        let mut manager = started_manager();
        manager.handle_event(MeasurementEvent::StartPhCalibration);
        assert_eq!(
            manager.handle_event(MeasurementEvent::FirstMeasured {
                first: 0.0,
                actual_first: 7.0,
            }),
            MeasurementAction::RetrievePhSecond
        );
    }

    #[test]
    fn test_abort_during_ph_calibration_first_returns_to_idle() {
        let mut manager = started_manager();
        manager.handle_event(MeasurementEvent::StartPhCalibration);
        assert_eq!(
            manager.handle_event(MeasurementEvent::Abort),
            MeasurementAction::WaitForNext
        );
    }

    #[test]
    fn test_second_measured_completes_ph_calibration_successfully() {
        let mut manager = started_manager();
        manager.handle_event(MeasurementEvent::StartPhCalibration);
        manager.handle_event(MeasurementEvent::FirstMeasured {
            first: 0.0,
            actual_first: 7.0,
        });
        assert_eq!(
            manager.handle_event(MeasurementEvent::SecondMeasured {
                second: 10.0,
                actual_second: 17.0,
            }),
            MeasurementAction::WaitForNext
        );
    }

    #[test]
    fn test_second_measured_fails_ph_calibration_when_readings_identical() {
        let mut manager = started_manager();
        manager.handle_event(MeasurementEvent::StartPhCalibration);
        manager.handle_event(MeasurementEvent::FirstMeasured {
            first: 5.0,
            actual_first: 7.0,
        });
        assert_eq!(
            manager.handle_event(MeasurementEvent::SecondMeasured {
                second: 5.0,
                actual_second: 17.0,
            }),
            MeasurementAction::ShowError {
                error: MeasurementError::CalibrationFailed,
            }
        );
    }

    #[test]
    fn test_abort_during_ph_calibration_second_returns_to_idle() {
        let mut manager = started_manager();
        manager.handle_event(MeasurementEvent::StartPhCalibration);
        manager.handle_event(MeasurementEvent::FirstMeasured {
            first: 0.0,
            actual_first: 7.0,
        });
        assert_eq!(
            manager.handle_event(MeasurementEvent::Abort),
            MeasurementAction::WaitForNext
        );
    }

    #[test]
    fn test_unexpected_event_in_idle_is_ignored() {
        let mut manager = started_manager();
        assert_eq!(
            manager.handle_event(MeasurementEvent::EcMeasured { ec: 1.0 }),
            MeasurementAction::Ignore
        );
    }

    #[test]
    fn test_unexpected_event_during_calibration_is_ignored() {
        let mut manager = started_manager();
        manager.handle_event(MeasurementEvent::StartEcCalibration);
        assert_eq!(
            manager.handle_event(MeasurementEvent::PhMeasured { ph: 1.0 }),
            MeasurementAction::Ignore
        );
    }
}
