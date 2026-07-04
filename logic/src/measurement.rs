use crate::{
    Float,
    calibration::{self, Point},
};

pub enum MeasurementState {
    Idle,
    MeasureEc,
    MeasurePh { ec: Float },
    CalibrateEcFirst,
    CalibrateEcSecond { first: Float },
    CalibratePhFirst,
    CalibratePhSecond { first: Float },
}

pub enum MeasurementEvent {
    Period,
    StartEcCalibration,
    StartPhCalibration,
    FirstRetrieved { first: Float },
    SecondRetrieved { second: Float },
    Abort,
    EcMeasured { ec: Float },
    PhMeasured { ph: Float },
}

pub enum MeasurementError {
    NotCalibratedYet,
    FailedCalibration,
}

pub enum MeasurementAction {
    Ignore,
    MeasureEc,
    MeasurePh,
    WriteMeasurements { ec: Float, ph: Float },
    ShowError { code: MeasurementError },
    RetrieveEcFirst,
    RetrieveEcSecond,
    RetrievePhFirst,
    RetrievePhSecond,
    StartPeriod,
}

pub struct MeasurementManager {
    state: MeasurementState,
    ec_calibration: calibration::Linear,
    ph_calibration: calibration::Linear,
}

impl MeasurementManager {
    pub fn handle_event(&mut self, event: MeasurementEvent) -> MeasurementAction {
        match (&self.state, event) {
            (MeasurementState::Idle, MeasurementEvent::Period) => {
                if self.ec_calibration.is_calibrated() && self.ph_calibration.is_calibrated() {
                    self.state = MeasurementState::MeasureEc;
                    MeasurementAction::MeasureEc
                } else {
                    MeasurementAction::ShowError {
                        code: MeasurementError::NotCalibratedYet,
                    }
                }
            }
            (MeasurementState::MeasureEc, MeasurementEvent::EcMeasured { ec }) => {
                self.state = MeasurementState::MeasurePh { ec };
                MeasurementAction::MeasurePh
            }
            (MeasurementState::MeasureEc, MeasurementEvent::Abort) => {
                self.state = MeasurementState::Idle;
                MeasurementAction::StartPeriod
            }
            (MeasurementState::MeasurePh { ec }, MeasurementEvent::PhMeasured { ph }) => {
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
                        code: MeasurementError::NotCalibratedYet,
                    },
                }
            }
            (MeasurementState::MeasurePh { .. }, MeasurementEvent::Abort) => {
                self.state = MeasurementState::Idle;
                MeasurementAction::StartPeriod
            }
            (MeasurementState::Idle, MeasurementEvent::StartEcCalibration) => {
                self.state = MeasurementState::CalibrateEcFirst;
                MeasurementAction::RetrieveEcFirst
            }
            (MeasurementState::CalibrateEcFirst, MeasurementEvent::FirstRetrieved { first }) => {
                self.state = MeasurementState::CalibrateEcSecond { first };
                MeasurementAction::RetrieveEcSecond
            }
            (MeasurementState::CalibrateEcFirst, MeasurementEvent::Abort) => {
                self.state = MeasurementState::Idle;
                MeasurementAction::StartPeriod
            }
            (
                MeasurementState::CalibrateEcSecond { first },
                MeasurementEvent::SecondRetrieved { second },
            ) => {
                let first = *first;
                self.state = MeasurementState::Idle;
                match self
                    .ec_calibration
                    .calibrate(Point { y: 5.0, x: first }, Point { y: 4.0, x: second })
                {
                    Ok(_) => MeasurementAction::StartPeriod,
                    Err(_) => MeasurementAction::ShowError {
                        code: MeasurementError::FailedCalibration,
                    },
                }
            }
            (MeasurementState::CalibrateEcSecond { .. }, MeasurementEvent::Abort) => {
                self.state = MeasurementState::Idle;
                MeasurementAction::StartPeriod
            }
            (MeasurementState::Idle, MeasurementEvent::StartPhCalibration) => {
                self.state = MeasurementState::CalibratePhFirst;
                MeasurementAction::RetrievePhFirst
            }
            (MeasurementState::CalibratePhFirst, MeasurementEvent::FirstRetrieved { first }) => {
                self.state = MeasurementState::CalibratePhSecond { first };
                MeasurementAction::RetrievePhSecond
            }
            (MeasurementState::CalibratePhFirst, MeasurementEvent::Abort) => {
                self.state = MeasurementState::Idle;
                MeasurementAction::StartPeriod
            }
            (
                MeasurementState::CalibratePhSecond { first },
                MeasurementEvent::SecondRetrieved { second },
            ) => {
                let first = *first;
                self.state = MeasurementState::Idle;
                match self
                    .ph_calibration
                    .calibrate(Point { y: 5.0, x: first }, Point { y: 4.0, x: second })
                {
                    Ok(_) => MeasurementAction::StartPeriod,
                    Err(_) => MeasurementAction::ShowError {
                        code: MeasurementError::FailedCalibration,
                    },
                }
            }
            (MeasurementState::CalibratePhSecond { .. }, MeasurementEvent::Abort) => {
                self.state = MeasurementState::Idle;
                MeasurementAction::StartPeriod
            }
            (_, _) => MeasurementAction::Ignore,
        }
    }
}
