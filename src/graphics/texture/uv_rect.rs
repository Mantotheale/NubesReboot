use std::ops::Add;

#[derive(Copy, Clone, Debug)]
pub struct TexCoord {
    u: f32,
    v: f32
}

impl TexCoord {
    pub const fn new(u: f32, v: f32) -> Self {
        Self { u, v }
    }

    pub const fn u(&self) -> f32 {
        self.u
    }

    pub const fn v(&self) -> f32 {
        self.v
    }
}

impl Into<[f32; 2]> for TexCoord {
    fn into(self) -> [f32; 2] {
        [self.u, self.v]
    }
}

#[derive(Copy, Clone)]
pub struct UVOffset {
    du: f32,
    dv: f32
}

impl UVOffset {
    pub const fn new(du: f32, dv: f32) -> Self {
        Self { du, dv }
    }

    pub const fn du(&self) -> f32 {
        self.du
    }

    pub const fn dv(&self) -> f32 {
        self.dv
    }
}

impl Add<UVOffset> for TexCoord {
    type Output = Self;

    fn add(self, rhs: UVOffset) -> Self::Output {
        Self { u: self.u + rhs.du, v: self.v + rhs.dv }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct UVRect {
    bottom_left: TexCoord,
    top_right: TexCoord
}

impl UVRect {
    pub fn from_corners(bottom_left: TexCoord, top_right: TexCoord) -> Self {
        Self { bottom_left, top_right }
    }

    pub fn from_corner_and_offset(bottom_left: TexCoord, offset: UVOffset) -> Self {
        Self { bottom_left, top_right: bottom_left + offset }
    }

    pub fn bottom_left(&self) -> TexCoord {
        self.bottom_left
    }

    pub fn bottom_right(&self) -> TexCoord {
        TexCoord::new(self.top_right.u, self.bottom_left.v)
    }

    pub fn top_right(&self) -> TexCoord {
        self.top_right
    }

    pub fn top_left(&self) -> TexCoord {
        TexCoord::new(self.bottom_left.u, self.top_right.v)
    }
}

impl Default for UVRect {
    fn default() -> Self {
        Self {
            bottom_left: TexCoord::new(0.0, 0.0),
            top_right: TexCoord::new(1.0, 1.0)
        }
    }
}