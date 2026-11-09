use crate::blackboard;
use picoserve::response::{IntoResponse, Json};

pub async fn measurements() -> impl IntoResponse {
    Json(blackboard::snapshot().await)
}
