use core::future;

#[derive(Clone, Copy)]
pub enum StepperState {
    Idle,
    Holding,
}

pub trait Stepper {
    type Error;

    fn move_steps(
        &mut self,
        steps: isize,
    ) -> impl future::Future<Output = Result<(), Self::Error>> + Send;
    fn hold(&mut self) -> impl future::Future<Output = Result<(), Self::Error>> + Send; //apply holding force
    fn release(&mut self) -> impl future::Future<Output = Result<(), Self::Error>> + Send; //release hold
    fn state(&self) -> StepperState;
    fn resolution(&self) -> usize;
}

pub mod drv8833 {

    use super::*;
    use esp_hal::gpio::{Level, Output};

    #[derive(Clone, Copy, defmt::Format)]
    #[repr(u8)]
    enum Step {
        First = 0b1000,
        Second = 0b0010,
        Third = 0b0100,
        Fourth = 0b0001,
    }

    impl Step {
        fn next(&mut self) {
            *self = match *self {
                Self::First => Self::Second,
                Self::Second => Self::Third,
                Step::Third => Step::Fourth,
                Step::Fourth => Step::First,
            };
        }

        fn previous(&mut self) {
            *self = match *self {
                Self::First => Self::Fourth,
                Self::Second => Self::First,
                Step::Third => Step::Second,
                Step::Fourth => Step::Third,
            };
        }
    }

    pub struct Drv8833<'a> {
        pins: [Output<'a>; 4],
        state: StepperState,
        current_step: Step,
        resolution: usize,
    }

    impl<'a> Drv8833<'a> {
        pub fn new(
            ain1: Output<'a>,
            ain2: Output<'a>,
            bin1: Output<'a>,
            bin2: Output<'a>,
            resolution: usize,
        ) -> Self {
            let pins = [ain1, ain2, bin1, bin2];
            Self {
                pins,
                state: StepperState::Idle,
                current_step: Step::First,
                resolution,
            }
        }

        fn set_state(&mut self) {
            let bits = self.current_step as u8;
            for (i, pin) in self.pins.iter_mut().enumerate() {
                let on = (bits >> (3 - i)) & 1 == 1;
                pin.set_level(if on { Level::High } else { Level::Low });
            }
        }
    }

    impl<'a> Stepper for Drv8833<'a> {
        type Error = ();

        async fn move_steps(&mut self, steps: isize) -> Result<(), Self::Error> {
            let abs_steps = steps.abs();
            for _ in 0..abs_steps {
                if steps > 0 {
                    self.current_step.next();
                } else {
                    self.current_step.previous();
                }
                self.set_state();
                let _ = embassy_time::Timer::after_micros(1024).await;
            }
            if matches!(self.state(), StepperState::Idle) {
                for pin in self.pins.iter_mut() {
                    pin.set_low();
                }
            }
            Ok(())
        }

        async fn hold(&mut self) -> Result<(), Self::Error> {
            self.set_state();
            self.state = StepperState::Holding;
            Ok(())
        }

        async fn release(&mut self) -> Result<(), Self::Error> {
            for pin in self.pins.iter_mut() {
                pin.set_low();
            }
            self.state = StepperState::Idle;
            Ok(())
        }

        fn state(&self) -> StepperState {
            self.state
        }

        fn resolution(&self) -> usize {
            self.resolution
        }
    }
}
