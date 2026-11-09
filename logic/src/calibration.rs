use crate::Float;

use sequential_storage::map::SerializationError;

#[derive(Clone, Copy)]
pub struct Point {
    pub x: Float,
    pub y: Float,
}

pub struct Linear {
    m: Option<Float>,
    t: Option<Float>,
}

impl<'d> sequential_storage::map::Value<'d> for Linear {
    fn serialize_into(&self, buffer: &mut [u8]) -> Result<usize, SerializationError> {
        match (self.m, self.t) {
            (Some(m), Some(t)) => {
                let m_bytes = m.to_le_bytes();
                let t_bytes = t.to_le_bytes();
                buffer[0..m_bytes.len()].copy_from_slice(&m_bytes);
                buffer[m_bytes.len()..m_bytes.len() + t_bytes.len()].copy_from_slice(&t_bytes);
                Ok(m_bytes.len() + t_bytes.len())
            }
            _ => Err(SerializationError::InvalidData),
        }
    }

    fn deserialize_from(buffer: &'d [u8]) -> Result<(Self, usize), SerializationError> {
        let size_of_m = core::mem::size_of::<Float>();
        let m = Float::from_le_bytes(buffer[0..size_of_m].try_into().unwrap());
        let size_of_t = core::mem::size_of::<Float>();
        let t = Float::from_le_bytes(buffer[size_of_m..size_of_m + size_of_t].try_into().unwrap());
        Ok((
            Linear {
                m: Some(m),
                t: Some(t),
            },
            size_of_t + size_of_m,
        ))
    }
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
