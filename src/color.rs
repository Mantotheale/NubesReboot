use crate::math::unit_f32::UnitF32;

#[derive(Copy, Clone)]
pub struct Color {
    r: UnitF32,
    g: UnitF32,
    b: UnitF32,
    a: UnitF32
}

impl Color {
    pub fn new(r: UnitF32, g: UnitF32, b: UnitF32, a: UnitF32) -> Self {
        Self { r, g, b, a }
    }
}

impl From<Color> for [f32; 4] {
    fn from(c: Color) -> Self {
        [*c.r, *c.g, *c.b, *c.a]
    }
}