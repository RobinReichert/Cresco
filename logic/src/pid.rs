pub type Float = f64;

pub struct PidController {
    kp: Float,
    ki: Float,
    kd: Float,
    integral: Float,
    last_error: Float,
    last_millis: usize,
}

impl PidController {
    pub fn new(p: Float, i: Float, d: Float) -> Self {
        Self {
            kp: p,
            ki: i,
            kd: d,
            integral: 0.0,
            last_error: 0.0,
            last_millis: 0,
        }
    }

    pub fn calculate(&mut self, current: Float, target: Float, millis: usize) -> Float {
        let dt = millis - self.last_millis;
        let error = target - current;

        let derivative = if dt > 0 {
            (error - self.last_error) / dt as Float
        } else {
            0 as Float
        };

        self.last_error = error;

        if self.last_millis == 0 {
            self.last_millis = millis;
            return self.kp * error;
        }
        self.last_millis = millis;

        self.integral += error * dt as Float;

        self.kp * error //proportional
        + self.ki * self.integral //integral
        + self.kd * derivative // derivative
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_first_call_no_spike() {
        let mut controller = PidController::new(1.0, 1.0, 1.0);
        let out = controller.calculate(2.0, 6.0, 40_000);
        assert!(out < 20.0);
    }

    #[test]
    fn test_two_concurrent_calculations() {
        let mut controller = PidController::new(1.0, 1.0, 1.0);
        controller.calculate(2.0, 6.0, 1000);
        let out = controller.calculate(4.0, 6.0, 1000);
        assert!(out.is_finite());
    }

    #[test]
    fn test_only_proportional() {
        let mut controller = PidController::new(0.5, 0.0, 0.0);
        controller.calculate(2.0, 6.0, 1000);
        let out = controller.calculate(4.0, 6.0, 2000);
        assert_eq!(out, 1.0);
    }

    #[test]
    fn test_only_proportional_direction() {
        let mut controller = PidController::new(0.5, 0.0, 0.0);
        let a = controller.calculate(2.0, 6.0, 1000);
        let b = controller.calculate(10.0, 6.0, 2000);
        assert!(a > b);
    }

    #[test]
    fn test_only_integral() {
        let mut controller = PidController::new(0.0, 0.5, 0.0);
        controller.calculate(2.0, 6.0, 1000);
        let a = controller.calculate(4.0, 6.0, 2000);
        let b = controller.calculate(4.0, 6.0, 3000);
        assert!(a < b);
    }

    #[test]
    fn test_only_derivative_constant() {
        let mut controller = PidController::new(0.0, 0.0, 0.5);
        controller.calculate(2.0, 6.0, 1000);
        let out = controller.calculate(2.0, 6.0, 2000);
        assert_eq!(out, 0.0);
    }

    #[test]
    fn test_only_derivative() {
        let mut controller = PidController::new(0.0, 0.0, 0.5);
        controller.calculate(2.0, 6.0, 1000);
        let out = controller.calculate(3.0, 6.0, 2000);
        assert!(out < 0.0);
    }
}
