use std::ops::{Div, Mul};
use crate::math::non_zero_f32::{NonZeroF32, ZeroF32Error};
use crate::math::normalized_vec2f::NormalizedVec2f;
use crate::math::point2f::Point2f;
use crate::math::positive_f32::PositiveF32;

#[derive(Copy, Clone)]
pub struct Vec2f {
    x: f32,
    y: f32
}

impl Vec2f {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y}
    }

    pub fn dot(&self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y
    }

    pub fn squared_len(&self) -> f32 {
        self.dot(*self)
    }

    pub fn len(&self) -> f32 {
        f32::sqrt(self.squared_len())
    }

    pub fn normalized(&self, epsilon: PositiveF32) -> Result<NormalizedVec2f, ZeroF32Error> {
        NormalizedVec2f::from_unnormalized(*self, epsilon)
    }

    pub fn left_normal(&self) -> Self {
        Self { x: -self.y, y: self.x }
    }

    pub fn right_normal(&self) -> Self {
        Self { x: self.y, y: -self.x }
    }
}

impl Mul<f32> for Vec2f {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl Mul<Vec2f> for f32 {
    type Output = Vec2f;

    fn mul(self, rhs: Vec2f) -> Self::Output {
        rhs * self
    }
}

impl Div<NonZeroF32> for Vec2f {
    type Output = Self;

    fn div(self, rhs: NonZeroF32) -> Self::Output {
        let inv = 1.0 / rhs;
        self * inv
    }
}

impl From<Vec2f> for [f32; 2] {
    fn from(v: Vec2f) -> Self {
        [v.x, v.y]
    }
}

impl From<[f32; 2]> for Vec2f {
    fn from(arr: [f32; 2]) -> Self {
        Self::new(arr[0], arr[1])
    }
}