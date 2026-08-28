use crate::math::non_zero_f32::{NonZeroF32, ZeroF32Error};
use crate::math::positive_f32::PositiveF32;
use crate::math::vec2f::Vec2f;

pub struct NormalizedVec2f {
    value: Vec2f
}

impl NormalizedVec2f {
    pub fn from_unnormalized(value: Vec2f, epsilon: PositiveF32) -> Result<Self, ZeroF32Error> {
        let len = NonZeroF32::new(value.len(), epsilon)?;
        Ok(Self { value: value / len })
    }
}

impl std::ops::Deref for NormalizedVec2f {
    type Target = Vec2f;

    fn deref(&self) -> &Vec2f {
        &self.value
    }
}

impl From<NormalizedVec2f> for Vec2f {
    fn from(v: NormalizedVec2f) -> Vec2f {
        v.value
    }
}

impl From<NormalizedVec2f> for [f32; 2] {
    fn from(v: NormalizedVec2f) -> Self {
        v.value.into()
    }
}