use embassy_time::Instant;
use logic::{Float, equations::Equation};
use picoserve::{
    extract::Json as JsonBody,
    response::{IntoResponse, Json, StatusCode},
};

const STUB_TARGET: Equation = Equation::Constant { c: 5.0 };

#[derive(serde::Serialize)]
struct TargetValues {
    ph: Option<Float>,
    ec: Option<Float>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Equations {
    pub ph: Equation,
    pub ec: Equation,
}

fn current(equation: Equation) -> Option<Float> {
    equation.y(Instant::now().as_secs() as Float).ok()
}

pub async fn target() -> impl IntoResponse {
    Json(TargetValues {
        ph: current(STUB_TARGET),
        ec: current(STUB_TARGET),
    })
}

pub async fn equations() -> impl IntoResponse {
    Json(Equations {
        ph: STUB_TARGET,
        ec: STUB_TARGET,
    })
}

// Stub: the equations are accepted but not applied yet.
pub async fn set_equations(JsonBody(_equations): JsonBody<Equations>) -> StatusCode {
    StatusCode::OK
}
