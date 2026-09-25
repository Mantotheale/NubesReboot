use std::ops::Add;
use crate::graphics::tex::atlas::PixelCoord;
use crate::graphics::tex::Tex;

#[derive(Copy, Clone, Debug)]
pub struct TexCoord {
    u: f32,
    v: f32
}

impl TexCoord {
    pub const fn new(u: f32, v: f32) -> Self {
        Self { u, v }
    }

    pub const fn from_cell_in_texture(cell: PixelCoord, tex_size: (u32, u32)) -> Self {
        Self {
            u: cell.col() as f32 / tex_size.1 as f32,
            v: cell.row() as f32 / tex_size.0 as f32
        }
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

    pub const fn from_pixels_in_texture(pixel_offset: (u32, u32), tex_size: (u32, u32)) -> Self {
        Self {
            du: pixel_offset.1 as f32 / tex_size.1 as f32,
            dv: pixel_offset.0 as f32 / tex_size.0 as f32
        }
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

    pub fn sub_texture_from_sizes(bottom_left: PixelCoord, sub_tex_size: (u32, u32), tex_size: (u32, u32)) -> Self {
        let bottom_left = TexCoord::from_cell_in_texture(bottom_left, tex_size);
        let offset = UVOffset::from_pixels_in_texture(sub_tex_size, tex_size);

        Self::from_corner_and_offset(bottom_left, offset)
    }

    pub fn sub_texture(bottom_left: PixelCoord, sub_tex_size: (u32, u32), tex: &Tex) -> Self {
        let bottom_left = TexCoord::from_cell_in_texture(bottom_left, (tex.width(), tex.height()));
        let offset = UVOffset::from_pixels_in_texture(sub_tex_size, (tex.width(), tex.height()));

        Self::from_corner_and_offset(bottom_left, offset)
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