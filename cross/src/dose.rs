use crate::blackboard;
use crate::stepper::{Stepper, drv8833::Drv8833};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Duration, Instant, Timer};
use logic::Float;
use logic::pid::PidController;

pub type ConcreteStepper<'a> = Drv8833<'a>;

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
    let mut ec_pid_controller = PidController::new(1.0, 0.01, 1.0);
    let mut ph_pid_controller = PidController::new(1.0, 0.01, 1.0);
    let mut next_period = Instant::now();
    loop {
        Timer::at(next_period).await;
        let timestamp = fresh.wait().await;
        next_period = Instant::now() + PERIOD_LENGTH;
        let measurements = blackboard::snapshot().await;
        match (measurements.ec, measurements.ph) {
            (Some(ec), Some(ph)) => {
                let ec_steps = ec_pid_controller.calculate(ec, 5.0, timestamp as usize) * RATE;
                if ec_steps > THRESHOLD {
                    let _ = ec_pump.move_steps(ec_steps as isize).await;
                    let _ = ec_pump.release().await;
                } else {
                    let ph_steps = ph_pid_controller.calculate(ph, 5.0, timestamp as usize) * RATE;
                    if ph_steps.abs() > THRESHOLD {
                        if ph_steps > 0.0 {
                            let _ = ph_up_pump.move_steps(ph_steps.abs() as isize).await;
                            let _ = ph_up_pump.release().await;
                        } else {
                            let _ = ph_down_pump.move_steps(ph_steps.abs() as isize).await;
                            let _ = ph_down_pump.release().await;
                        };
                    }
                }
            }
            _ => {}
        }
    }
}
