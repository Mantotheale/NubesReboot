use std::fmt::{Debug, Formatter};
use std::ops::{Add, Sub};
use crate::math::positive_f32::PositiveF32;
use crate::math::vec2f::Vec2f;
use super::approx_eq::ApproxEq;

#[derive(Copy, Clone)]
pub struct Point2f {
    x: f32,
    y: f32
}

impl Point2f {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn x(&self) -> f32 {
        self.x
    }

    pub fn y(&self) -> f32 {
        self.y
    }
}

impl Add for Point2f {
    type Output = Vec2f;

    fn add(self, rhs: Self) -> Self::Output {
        Vec2f::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for Point2f {
    type Output = Vec2f;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec2f::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl ApproxEq for Point2f {
    fn approx_eq(&self, other: &Self, epsilon: PositiveF32) -> bool {
        self.x.approx_eq(&other.x, epsilon) && self.y.approx_eq(&other.y, epsilon)
    }
}

impl Debug for Point2f {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Point2f(x: {}, y: {})", self.x, self.y)
    }
}

impl From<Point2f> for [f32; 2] {
    fn from(p: Point2f) -> Self {
        [p.x, p.y]
    }
}

impl From<[f32; 2]> for Point2f {
    fn from(arr: [f32; 2]) -> Self {
        Self::new(arr[0], arr[1])
    }
}