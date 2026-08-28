#[derive(Debug)]
pub struct NonUnitF32Error {
    value: f32
}

impl std::fmt::Display for NonUnitF32Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "The specified value is not in the interval [0, 1], it was {}", self.value)
    }
}

impl std::error::Error for NonUnitF32Error { }

#[derive(Copy, Clone)]
pub struct UnitF32 {
    value: f32
}

impl UnitF32 {
    pub fn new(value: f32) -> Result<Self, NonUnitF32Error> {
        if value < 0.0 || value > 1.0 { Err(NonUnitF32Error { value }) }
        else { Ok(Self { value }) }
    }

    pub fn value(&self) -> f32 {
        self.value
    }
}

impl std::ops::Deref for UnitF32 {
    type Target = f32;

    fn deref(&self) -> &f32 {
        &self.value
    }
}

impl From<UnitF32> for f32 {
    fn from(p: UnitF32) -> f32 {
        p.value
    }
}