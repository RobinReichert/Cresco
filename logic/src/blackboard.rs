use crate::Float;

#[derive(Clone, serde::Serialize)]
pub struct Measurements {
    pub ph: Option<Float>,
    pub ec: Option<Float>,
}

impl Measurements {
    pub const DEFAULT: Self = Self { ph: None, ec: None };
}
