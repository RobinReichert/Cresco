use crate::{Float, equations::Equation};

#[derive(Clone, serde::Serialize)]
pub struct Measurements {
    pub ph: Option<Float>,
    pub ec: Option<Float>,
}

impl Measurements {
    pub const DEFAULT: Self = Self { ph: None, ec: None };
}

#[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Targets {
    pub ph: Equation,
    pub ec: Equation,
}

impl Targets {
    pub const DEFAULT: Self = Self {
        ph: Equation::Constant { c: 5.0 },
        ec: Equation::Constant { c: 5.0 },
    };
}
