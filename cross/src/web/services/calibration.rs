use crate::measurement::{MeasurementCommand, MeasurementStatus, StatusCell};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use logic::Float;
use picoserve::{
    extract::{Form, State},
    response::{IntoResponse, Json, StatusCode},
};

#[derive(Clone, Copy)]
pub struct CalibrationState {
    pub commands: &'static Channel<CriticalSectionRawMutex, MeasurementCommand, 1>,
    pub status: &'static StatusCell,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sensor {
    Ph,
    Ec,
}

#[derive(serde::Deserialize)]
pub struct CalibrationStartForm {
    sensor: Sensor,
}

#[derive(serde::Deserialize)]
pub struct CalibrationPointForm {
    sensor: Sensor,
    actual: Float,
}

#[derive(serde::Serialize)]
struct CalibrationStatusData {
    status: MeasurementStatus,
}

fn send_command(state: &CalibrationState, command: MeasurementCommand) -> StatusCode {
    match state.commands.try_send(command) {
        Ok(()) => StatusCode::OK,
        Err(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}

pub async fn status(State(state): State<CalibrationState>) -> impl IntoResponse {
    Json(CalibrationStatusData {
        status: *state.status.lock().await,
    })
}

pub async fn start(
    State(state): State<CalibrationState>,
    Form(form): Form<CalibrationStartForm>,
) -> StatusCode {
    send_command(
        &state,
        match form.sensor {
            Sensor::Ph => MeasurementCommand::StartPhCalibration,
            Sensor::Ec => MeasurementCommand::StartEcCalibration,
        },
    )
}

pub async fn point(
    State(state): State<CalibrationState>,
    Form(form): Form<CalibrationPointForm>,
) -> StatusCode {
    send_command(
        &state,
        match form.sensor {
            Sensor::Ph => MeasurementCommand::PhMeasurement {
                actual_ph: form.actual,
            },
            Sensor::Ec => MeasurementCommand::EcMeasurement {
                actual_ec: form.actual,
            },
        },
    )
}

pub async fn abort(State(state): State<CalibrationState>) -> StatusCode {
    send_command(&state, MeasurementCommand::Abort)
}
