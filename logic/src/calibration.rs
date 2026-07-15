use crate::Float;

#[derive(Clone, Copy)]
pub struct Point {
    pub x: Float,
    pub y: Float,
}

pub struct Linear {
    m: Option<Float>,
    t: Option<Float>,
}

impl Linear {
    pub fn new() -> Self {
        Self { m: None, t: None }
    }

    pub fn is_calibrated(&self) -> bool {
        self.m.is_some() && self.t.is_some()
    }

    pub fn calibrate(&mut self, first: Point, second: Point) -> Result<(), ()> {
        if second.x == first.x {
            return Err(());
        }
        let m = (second.y - first.y) / (second.x - first.x);
        self.t = Some(first.y - m * first.x);
        self.m = Some(m);
        return Ok(());
    }

    pub fn apply(&self, x: Float) -> Result<Float, ()> {
        if let Some((m, t)) = self.m.zip(self.t) {
            return Ok(m * x + t);
        }
        Err(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_calibrate_same_x_errors() {
        let mut linear = Linear::new();
        let first = Point { x: 1.0, y: 0.0 };
        let second = Point { x: 1.0, y: 5.0 };
        assert_eq!(linear.calibrate(first, second), Err(()));
    }

    #[test]
    fn test_apply_before_calibrate_errors() {
        let linear = Linear::new();
        assert_eq!(linear.apply(1.0), Err(()));
    }

    #[test]
    fn test_is_calibrated_before_calibrate_is_false() {
        let linear = Linear::new();
        assert!(!linear.is_calibrated());
    }

    #[test]
    fn test_is_calibrated_after_calibrate_is_true() {
        let mut linear = Linear::new();
        let first = Point { x: 0.0, y: 0.0 };
        let second = Point { x: 2.0, y: 4.0 };
        assert_eq!(linear.calibrate(first, second), Ok(()));
        assert!(linear.is_calibrated());
    }

    #[test]
    fn test_is_calibrated_after_failed_calibrate_is_false() {
        let mut linear = Linear::new();
        let first = Point { x: 1.0, y: 0.0 };
        let second = Point { x: 1.0, y: 5.0 };
        assert_eq!(linear.calibrate(first, second), Err(()));
        assert!(!linear.is_calibrated());
    }

    #[test]
    fn test_calibrate_and_apply() {
        let mut linear = Linear::new();
        let first = Point { x: 0.0, y: 0.0 };
        let second = Point { x: 2.0, y: 4.0 };
        assert_eq!(linear.calibrate(first, second), Ok(()));

        assert_eq!(linear.apply(0.0).unwrap(), 0.0);
        assert_eq!(linear.apply(2.0).unwrap(), 4.0);
        assert_eq!(linear.apply(1.0).unwrap(), 2.0);
    }
}
