use crate::blackboard;
use embassy_time::Instant;
use logic::{Float, blackboard::Targets, equations::Equation};
use picoserve::{
    extract::Json as JsonBody,
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

pub async fn set_equations(JsonBody(targets): JsonBody<Targets>) -> StatusCode {
    blackboard::set_targets(targets).await;
    StatusCode::OK
}
