use crate::math::positive_f32::PositiveF32;

pub trait ApproxEq {
    fn approx_eq(&self, other: &Self, epsilon: PositiveF32) -> bool;
}

impl ApproxEq for f32 {
    fn approx_eq(&self, other: &Self, epsilon: PositiveF32) -> bool {
        (self - other).abs() <= *epsilon
    }
}