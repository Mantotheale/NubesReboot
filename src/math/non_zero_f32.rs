use std::ops::{Deref, Div};
use crate::math::approx_eq::ApproxEq;
use crate::math::positive_f32::PositiveF32;

#[derive(Debug)]
pub struct ZeroF32Error {
    value: f32
}

impl std::fmt::Display for ZeroF32Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "The specified value is zero: {}", self.value)
    }
}

impl std::error::Error for ZeroF32Error { }

#[derive(Debug, Copy, Clone)]
pub struct NonZeroF32 {
    value: f32
}

impl NonZeroF32 {
    pub fn new(value: f32, epsilon: PositiveF32) -> Result<Self, ZeroF32Error> {
        if value.approx_eq(&0f32, epsilon) { Err(ZeroF32Error { value }) }
        else { Ok(Self { value }) }
    }
}

impl Deref for NonZeroF32 {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl AsRef<f32> for NonZeroF32 {
    fn as_ref(&self) -> &f32 {
        self.deref()
    }
}

impl Div<NonZeroF32> for f32 {
    type Output = f32;

    fn div(self, rhs: NonZeroF32) -> Self::Output {
        self / rhs.value
    }
}