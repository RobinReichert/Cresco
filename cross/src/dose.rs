use crate::blackboard;
use crate::stepper::{Stepper, dummy::Dummy};
use defmt::{debug, info};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Duration, Instant, Timer};
use logic::Float;
use logic::pid::PidController;

pub type ConcreteStepper<'a> = Dummy<'a>;

const PERIOD_LENGTH: Duration = Duration::from_secs(60);
const THRESHOLD: Float = 1.0;
const RATE: Float = 1.0;

#[embassy_executor::task]
pub async fn dose_task(
    mut ec_pump: ConcreteStepper<'static>,
    mut ph_up_pump: ConcreteStepper<'static>,
    mut ph_down_pump: ConcreteStepper<'static>,
    fresh: &'static Signal<CriticalSectionRawMutex, u64>,
) {
    info!("dose: task start");
    let mut ec_pid_controller = PidController::new(1.0, 1.0, 1.0);
    let mut ph_pid_controller = PidController::new(1.0, 1.0, 1.0);
    let mut next_period = Instant::now();
    loop {
        Timer::at(next_period).await;
        debug!("dose: period elapsed");
        let timestamp = fresh.wait().await;
        debug!("dose: fresh measurement available");
        next_period = Instant::now() + PERIOD_LENGTH;
        let measurements = blackboard::measurements().await;
        let targets = blackboard::targets().await;
        match (measurements.ec, measurements.ph) {
            (Some(ec), Some(ph)) => {
                let now_secs = Instant::now().as_secs();
                let ec_target = match targets.ec.y(now_secs as Float) {
                    Ok(y) => y,
                    Err(_) => continue,
                };
                let ec_steps = ec_pid_controller.calculate(
                    ec,
                    ec_target,
                    (timestamp / PERIOD_LENGTH.as_millis()) as usize,
                ) * RATE;
                debug!(
                    "dose: {} steps from ec: {} to target: {} ",
                    ec_steps, ec, ec_target
                );
                if ec_steps > THRESHOLD {
                    debug!("dose: move ec pump by {} steps", ec_steps);
                    let _ = ec_pump.move_steps(ec_steps as isize).await;
                    let _ = ec_pump.release().await;
                    ph_pid_controller.reset();
                } else {
                    debug!(
                        "dose: {} steps are below threshold: {}",
                        ec_steps, THRESHOLD
                    );
                    let ph_target = match targets.ph.y(now_secs as Float) {
                        Ok(y) => y,
                        Err(_) => continue,
                    };
                    let ph_steps = ph_pid_controller.calculate(
                        ph,
                        ph_target,
                        (timestamp / PERIOD_LENGTH.as_millis()) as usize,
                    ) * RATE;
                    debug!(
                        "dose: {} steps from ph: {} to target: {} ",
                        ph_steps, ph, ph_target
                    );
                    if ph_steps.abs() > THRESHOLD {
                        if ph_steps > 0.0 {
                            debug!("dose: move ph up pump by {} steps", ph_steps.abs());
                            let _ = ph_up_pump.move_steps(ph_steps.abs() as isize).await;
                            let _ = ph_up_pump.release().await;
                        } else {
                            debug!("dose: move ph down pump by {} steps", ph_steps.abs());
                            let _ = ph_down_pump.move_steps(ph_steps.abs() as isize).await;
                            let _ = ph_down_pump.release().await;
                        };
                    } else {
                        debug!(
                            "dose: {} steps are below threshold: {}",
                            ec_steps, THRESHOLD
                        );
                    }
                }
            }
            _ => {
                debug!("dose: measurements none");
            }
        }
    }
}
