use crate::Float;

use sequential_storage::map::SerializationError;

const FLOAT_SIZE: usize = core::mem::size_of::<Float>();
const TAG_SIZE: usize = 1;
const TAG_LINEAR: u8 = 1;
const LINEAR_SIZE: usize = TAG_SIZE + 2 * FLOAT_SIZE;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Equation {
    Linear { m: Float, t: Float },
}

#[derive(Clone, Copy)]
pub struct Point {
    pub x: Float,
    pub y: Float,
}

impl<'d> sequential_storage::map::Value<'d> for Equation {
    fn serialize_into(&self, buffer: &mut [u8]) -> Result<usize, SerializationError> {
        match self {
            Equation::Linear { m, t } => {
                if buffer.len() < LINEAR_SIZE {
                    return Err(SerializationError::BufferTooSmall);
                }
                buffer[0] = TAG_LINEAR;
                buffer[TAG_SIZE..TAG_SIZE + FLOAT_SIZE].copy_from_slice(&m.to_le_bytes());
                buffer[TAG_SIZE + FLOAT_SIZE..LINEAR_SIZE].copy_from_slice(&t.to_le_bytes());
                Ok(LINEAR_SIZE)
            }
        }
    }

    fn deserialize_from(buffer: &'d [u8]) -> Result<(Self, usize), SerializationError> {
        match buffer.first() {
            Some(&TAG_LINEAR) if buffer.len() >= LINEAR_SIZE => {
                let m = Float::from_le_bytes(
                    buffer[TAG_SIZE..TAG_SIZE + FLOAT_SIZE].try_into().unwrap(),
                );
                let t = Float::from_le_bytes(
                    buffer[TAG_SIZE + FLOAT_SIZE..LINEAR_SIZE]
                        .try_into()
                        .unwrap(),
                );
                Ok((Equation::Linear { m, t }, LINEAR_SIZE))
            }
            _ => Err(SerializationError::InvalidData),
        }
    }
}

impl Equation {
    pub fn linear_from_points(first: Point, second: Point) -> Result<Self, ()> {
        if second.x == first.x || second.y == first.y {
            return Err(());
        }
        let m = (second.y - first.y) / (second.x - first.x);
        Ok(Equation::Linear {
            m,
            t: first.y - m * first.x,
        })
    }

    pub fn y(&self, x: Float) -> Result<Float, ()> {
        match self {
            Equation::Linear { m, t } => Ok(m * x + t),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use sequential_storage::map::Value;

    #[test]
    fn test_linear_y() {
        let linear = Equation::Linear { m: 2.0, t: 1.0 };

        assert_eq!(linear.y(0.0), Ok(1.0));
        assert_eq!(linear.y(2.0), Ok(5.0));
        assert_eq!(linear.y(-1.0), Ok(-1.0));
    }

    #[test]
    fn test_linear_from_points_same_x_errors() {
        let first = Point { x: 1.0, y: 0.0 };
        let second = Point { x: 1.0, y: 5.0 };
        assert_eq!(Equation::linear_from_points(first, second), Err(()));
    }

    #[test]
    fn test_linear_from_points_same_y_errors() {
        let first = Point { x: 0.5, y: 5.0 };
        let second = Point { x: 1.0, y: 5.0 };
        assert_eq!(Equation::linear_from_points(first, second), Err(()));
    }

    #[test]
    fn test_linear_from_points_passes_through_both_points() {
        let first = Point { x: 0.0, y: 0.0 };
        let second = Point { x: 2.0, y: 4.0 };
        let linear = Equation::linear_from_points(first, second).unwrap();

        assert_eq!(linear.y(0.0), Ok(0.0));
        assert_eq!(linear.y(2.0), Ok(4.0));
        assert_eq!(linear.y(1.0), Ok(2.0));
    }

    #[test]
    fn test_serialize_deserialize_round_trip() {
        let original = Equation::Linear { m: -1.5, t: 5.25 };
        let mut buffer = [0u8; LINEAR_SIZE];
        let len = original.serialize_into(&mut buffer).unwrap();

        let (restored, read) = Equation::deserialize_from(&buffer[..len]).unwrap();

        assert_eq!(restored, original);
        assert_eq!(read, len);
    }

    #[test]
    fn test_serialize_into_small_buffer_errors() {
        let mut buffer = [0u8; LINEAR_SIZE - 1];
        assert_eq!(
            Equation::Linear { m: 1.0, t: 0.0 }.serialize_into(&mut buffer),
            Err(SerializationError::BufferTooSmall)
        );
    }

    #[test]
    fn test_deserialize_unknown_tag_errors() {
        let buffer = [0u8; LINEAR_SIZE];
        assert_eq!(
            Equation::deserialize_from(&buffer),
            Err(SerializationError::InvalidData)
        );
    }

    #[test]
    fn test_deserialize_untagged_data_errors() {
        // calibrations stored before the tag byte existed: 2 floats, no tag
        let mut buffer = [0u8; 2 * FLOAT_SIZE];
        let m: Float = 10.0;
        buffer[..FLOAT_SIZE].copy_from_slice(&m.to_le_bytes());
        assert_eq!(
            Equation::deserialize_from(&buffer),
            Err(SerializationError::InvalidData)
        );
    }
}
