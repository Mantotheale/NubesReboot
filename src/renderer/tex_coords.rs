use std::ops::Add;

#[derive(Copy, Clone, Debug)]
pub struct TexCoords {
    u: f32,
    v: f32
}

impl TexCoords {
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

impl Into<[f32; 2]> for TexCoords {
    fn into(self) -> [f32; 2] {
        [self.u, self.v]
    }
}

#[derive(Copy, Clone)]
pub struct TexCoordsOffset {
    du: f32,
    dv: f32
}

impl TexCoordsOffset {
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

impl Add<TexCoordsOffset> for TexCoords {
    type Output = Self;

    fn add(self, rhs: TexCoordsOffset) -> Self::Output {
        Self { u: self.u + rhs.du, v: self.v + rhs.dv }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct RectTexCoords {
    bottom_left: TexCoords,
    top_right: TexCoords
}

impl RectTexCoords {
    pub const DEFAULT_COORDS: Self = 
        Self { 
            bottom_left: TexCoords::new(0.0, 0.0),
            top_right: TexCoords::new(1.0, 1.0)
        };
    
    pub fn from_bottom_left_corner_and_dimensions(bottom_left: TexCoords, offset: TexCoordsOffset) -> Self {
        Self { bottom_left, top_right: bottom_left + offset}
    }
    
    pub fn bottom_left(&self) -> TexCoords {
        self.bottom_left
    }

    pub fn bottom_right(&self) -> TexCoords {
        TexCoords::new(self.top_right.u, self.bottom_left.v)
    }

    pub fn top_right(&self) -> TexCoords {
        self.top_right
    }

    pub fn top_left(&self) -> TexCoords {
        TexCoords::new(self.bottom_left.u, self.top_right.v)
    }
}