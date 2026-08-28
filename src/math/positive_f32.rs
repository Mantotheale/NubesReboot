#[derive(Debug)]
pub struct NonPositiveF32Error {
    value: f32
}

impl std::fmt::Display for NonPositiveF32Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "The specified value is not positive, it was {}", self.value)
    }
}

impl std::error::Error for NonPositiveF32Error { }

#[derive(Copy, Clone)]
pub struct PositiveF32 {
    value: f32
}

impl PositiveF32 {
    pub const fn panicking_new_const(value: f32) -> Self {
        assert!(value > 0.0, "Value must be positive");
        Self { value }
    }
    
    pub fn new(value: f32) -> Result<Self, NonPositiveF32Error> {
        if value > 0.0 { Ok(Self { value }) }
        else { Err(NonPositiveF32Error { value }) }
    }

    pub fn value(&self) -> f32 {
        self.value
    }
}

impl std::ops::Deref for PositiveF32 {
    type Target = f32;
    
    fn deref(&self) -> &f32 {
        &self.value
    }
}

impl From<PositiveF32> for f32 {
    fn from(p: PositiveF32) -> f32 {
        p.value
    }
}