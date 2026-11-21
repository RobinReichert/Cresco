use crate::{blackboard, storage::ConcreteStorage};
use embassy_time::Instant;
use logic::{Float, blackboard::Targets, equations::Equation};
use picoserve::{
    extract::{Json as JsonBody, State},
    response::{IntoResponse, Json, StatusCode},
};

#[derive(serde::Serialize)]
struct TargetValues {
    ph: Option<Float>,
    ec: Option<Float>,
}

fn current(equation: Equation) -> Option<Float> {
    equation.y(Instant::now().as_secs() as Float).ok()
}

pub async fn target() -> impl IntoResponse {
    let targets = blackboard::targets().await;
    Json(TargetValues {
        ph: current(targets.ph),
        ec: current(targets.ec),
    })
}

pub async fn equations() -> impl IntoResponse {
    Json(blackboard::targets().await)
}

pub async fn set_equations(
    State(storage): State<&'static ConcreteStorage>,
    JsonBody(targets): JsonBody<Targets>,
) -> StatusCode {
    blackboard::set_targets(targets).await;
    let ph_stored = storage.set_ph_target(targets.ph).await;
    let ec_stored = storage.set_ec_target(targets.ec).await;
    if ph_stored && ec_stored {
        StatusCode::OK
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}
