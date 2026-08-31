use crate::math::unit_f32::UnitF32;

#[derive(Copy, Clone)]
pub struct Color {
    r: UnitF32,
    g: UnitF32,
    b: UnitF32,
    a: UnitF32
}

impl Color {
    pub const SOLID_BLACK: Self = Self {
        r: UnitF32::ZERO,
        g: UnitF32::ZERO,
        b: UnitF32::ZERO,
        a: UnitF32::ONE,
    };

    pub fn new(r: UnitF32, g: UnitF32, b: UnitF32, a: UnitF32) -> Self {
        Self { r, g, b, a }
    }
}

impl From<Color> for [f32; 4] {
    fn from(c: Color) -> Self {
        [*c.r, *c.g, *c.b, *c.a]
    }
}

impl From<Color> for wgpu::Color {
    fn from(value: Color) -> Self {
        wgpu::Color { 
            r: value.r.value() as f64,
            g: value.r.value() as f64,
            b: value.r.value() as f64,
            a: value.r.value() as f64
        }
    }
}