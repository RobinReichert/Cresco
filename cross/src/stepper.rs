#[derive(Clone, Copy)]
pub enum StepperState {
    Idle,
    Holding,
    Moving,
}

pub trait Stepper {
    const RESOLUTION: isize; //steps per revolution
    type Error;

    async fn move_steps(&mut self, steps: isize) -> Result<(), Self::Error>;
    async fn stop(&mut self) -> Result<(), Self::Error>; //abort current move
    async fn hold(&mut self) -> Result<(), Self::Error>; //apply holding force
    async fn release(&mut self) -> Result<(), Self::Error>; //release hold
    fn state(&self) -> StepperState;
}
