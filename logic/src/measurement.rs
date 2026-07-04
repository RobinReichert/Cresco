use crate::{
    Float,
    calibration::{self, Point},
};

pub enum MeasurementState {
    Idle,
    MeasuringEc,
    MeasuringPh { ec: Float },
    CalibratingEcFirst,
    CalibratingEcSecond { first: Float },
    CalibratingPhFirst,
    CalibratingPhSecond { first: Float },
}

pub enum MeasurementEvent {
    StartMeasurement,
    StartEcCalibration,
    StartPhCalibration,
    FirstMeasured { first: Float },
    SecondMeasured { second: Float },
    Abort,
    EcMeasured { ec: Float },
    PhMeasured { ph: Float },
}

pub enum MeasurementError {
    NotCalibratedYet,
    CalibrationFailed,
}

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
            state: MeasurementState::Idle,
            ec_calibration,
            ph_calibration,
        }
    }

    pub fn handle_event(&mut self, event: MeasurementEvent) -> MeasurementAction {
        match (&self.state, event) {
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
            (MeasurementState::CalibratingEcFirst, MeasurementEvent::FirstMeasured { first }) => {
                self.state = MeasurementState::CalibratingEcSecond { first };
                MeasurementAction::RetrieveEcSecond
            }
            (MeasurementState::CalibratingEcFirst, MeasurementEvent::Abort) => {
                self.state = MeasurementState::Idle;
                MeasurementAction::WaitForNext
            }
            (
                MeasurementState::CalibratingEcSecond { first },
                MeasurementEvent::SecondMeasured { second },
            ) => {
                let first = *first;
                self.state = MeasurementState::Idle;
                match self
                    .ec_calibration
                    .calibrate(Point { y: 5.0, x: first }, Point { y: 4.0, x: second })
                {
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
            (MeasurementState::CalibratingPhFirst, MeasurementEvent::FirstMeasured { first }) => {
                self.state = MeasurementState::CalibratingPhSecond { first };
                MeasurementAction::RetrievePhSecond
            }
            (MeasurementState::CalibratingPhFirst, MeasurementEvent::Abort) => {
                self.state = MeasurementState::Idle;
                MeasurementAction::WaitForNext
            }
            (
                MeasurementState::CalibratingPhSecond { first },
                MeasurementEvent::SecondMeasured { second },
            ) => {
                let first = *first;
                self.state = MeasurementState::Idle;
                match self
                    .ph_calibration
                    .calibrate(Point { y: 5.0, x: first }, Point { y: 4.0, x: second })
                {
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
