use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use logic::{
    Float,
    blackboard::{Measurements, Targets},
};

static MEASUREMENTS: Mutex<CriticalSectionRawMutex, Measurements> =
    Mutex::new(Measurements::DEFAULT);

static TARGETS: Mutex<CriticalSectionRawMutex, Targets> = Mutex::new(Targets::DEFAULT);

pub async fn measurements() -> Measurements {
    MEASUREMENTS.lock().await.clone()
}
pub async fn set_ph(ph: Float) {
    MEASUREMENTS.lock().await.ph = Some(ph)
}
pub async fn set_ec(ec: Float) {
    MEASUREMENTS.lock().await.ec = Some(ec)
}

pub async fn targets() -> Targets {
    *TARGETS.lock().await
}
pub async fn set_targets(targets: Targets) {
    *TARGETS.lock().await = targets
}
