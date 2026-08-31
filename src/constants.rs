use crate::math::positive_f32::PositiveF32;

pub const MATH_EPSILON: PositiveF32 = PositiveF32::panicking_new_const(1e-5);

pub const MAX_SEGMENTS_BATCH: usize = 100;