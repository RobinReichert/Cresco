use crate::Float;

use sequential_storage::map::SerializationError;

#[derive(Clone, Copy)]
pub struct Point {
    pub x: Float,
    pub y: Float,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Linear {
    m: Float,
    t: Float,
}

impl<'d> sequential_storage::map::Value<'d> for Linear {
    fn serialize_into(&self, buffer: &mut [u8]) -> Result<usize, SerializationError> {
        let m_bytes = self.m.to_le_bytes();
        let t_bytes = self.t.to_le_bytes();
        buffer[0..m_bytes.len()].copy_from_slice(&m_bytes);
        buffer[m_bytes.len()..m_bytes.len() + t_bytes.len()].copy_from_slice(&t_bytes);
        Ok(m_bytes.len() + t_bytes.len())
    }

    fn deserialize_from(buffer: &'d [u8]) -> Result<(Self, usize), SerializationError> {
        let size_of_m = core::mem::size_of::<Float>();
        let m = Float::from_le_bytes(buffer[0..size_of_m].try_into().unwrap());
        let size_of_t = core::mem::size_of::<Float>();
        let t = Float::from_le_bytes(buffer[size_of_m..size_of_m + size_of_t].try_into().unwrap());
        Ok((Linear { m, t }, size_of_t + size_of_m))
    }
}

impl Linear {
    pub fn new(m: Float, t: Float) -> Self {
        Self { m, t }
    }

    pub fn from_points(first: Point, second: Point) -> Result<Self, ()> {
        if second.x == first.x || second.y == first.y {
            return Err(());
        }
        let m = (second.y - first.y) / (second.x - first.x);
        Ok(Self::new(m, first.y - m * first.x))
    }

    pub fn apply(&self, x: Float) -> Float {
        self.m * x + self.t
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use sequential_storage::map::Value;

    #[test]
    fn test_apply() {
        let linear = Linear::new(2.0, 1.0);

        assert_eq!(linear.apply(0.0), 1.0);
        assert_eq!(linear.apply(2.0), 5.0);
        assert_eq!(linear.apply(-1.0), -1.0);
    }

    #[test]
    fn test_from_points_same_x_errors() {
        let first = Point { x: 1.0, y: 0.0 };
        let second = Point { x: 1.0, y: 5.0 };
        assert_eq!(Linear::from_points(first, second), Err(()));
    }

    #[test]
    fn test_from_points_same_y_errors() {
        let first = Point { x: 0.5, y: 5.0 };
        let second = Point { x: 1.0, y: 5.0 };
        assert_eq!(Linear::from_points(first, second), Err(()));
    }

    #[test]
    fn test_from_points_passes_through_both_points() {
        let first = Point { x: 0.0, y: 0.0 };
        let second = Point { x: 2.0, y: 4.0 };
        let linear = Linear::from_points(first, second).unwrap();

        assert_eq!(linear.apply(0.0), 0.0);
        assert_eq!(linear.apply(2.0), 4.0);
        assert_eq!(linear.apply(1.0), 2.0);
    }

    const STORED_LEN: usize = 2 * core::mem::size_of::<Float>();

    #[test]
    fn test_serialize_deserialize_round_trip() {
        let original = Linear::new(-1.5, 5.25);
        let mut buffer = [0u8; STORED_LEN];
        let len = original.serialize_into(&mut buffer).unwrap();

        let (restored, _) = Linear::deserialize_from(&buffer[..len]).unwrap();

        assert_eq!(restored, original);
    }
}
