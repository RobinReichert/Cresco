use crate::Float;

pub struct Linear {
    m: Option<Float>,
    t: Option<Float>,
}

impl Linear {
    pub fn new() -> Self {
        Self { m: None, t: None }
    }
}
