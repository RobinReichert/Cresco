use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use logic::{Float, blackboard::Measurements};

static MEASUREMENTS: Mutex<CriticalSectionRawMutex, Measurements> =
    Mutex::new(Measurements::DEFAULT);

pub async fn snapshot() -> Measurements {
    MEASUREMENTS.lock().await.clone()
}
pub async fn set_ph(ph: Float) {
    MEASUREMENTS.lock().await.ph = Some(ph)
}
