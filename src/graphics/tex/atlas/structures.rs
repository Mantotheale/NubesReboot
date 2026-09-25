use crate::graphics::tex::Tex;
use crate::graphics::tex::texture_handle::TexHandle;
use crate::graphics::texture::uv_rect::UVRect;

#[derive(Debug)]
pub struct TileCantFitInEmptySpaceError;

impl std::fmt::Display for TileCantFitInEmptySpaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "The tile can't fit in the empty space")
    }
}

impl std::error::Error for TileCantFitInEmptySpaceError { }

#[derive(Debug, Copy, Clone)]
pub struct PixelCoord {
    row: u32,
    col: u32
}

impl PixelCoord {
    pub fn new(row: u32, col: u32) -> Self {
        Self { row, col }
    }

    pub fn offset(&self, row_offset: u32, col_offset: u32) -> Self {
        Self { row: self.row + row_offset, col: self.col + col_offset }
    }
    
    pub const fn row(&self) -> u32 {
        self.row
    }

    pub const fn col(&self) -> u32 {
        self.col
    }
}

#[derive(Debug, Copy, Clone)]
pub struct EmptySpace {
    corner: PixelCoord,
    width: u32,
    height: u32
}

impl EmptySpace {
    pub fn starting_space(width: u32, height: u32) -> Self {
        Self { corner: PixelCoord::new(0, 0), width, height }
    }

    pub fn can_image_fit(&self, width: u32, height: u32) -> bool {
        width <= self.width && height <= self.height
    }

    pub fn split_space(&self, width: u32, height: u32) -> Vec<Self> {
        if !self.can_image_fit(width, height) { return Vec::new(); }

        let remainder_width = self.width - width;
        let remainder_height = self.height - height;

        let mut res = Vec::new();
        if remainder_height > 0 {
            res.push(Self {
                corner: self.corner.offset(height, 0),
                width: self.width,
                height: remainder_height
            });
        }

        if remainder_width > 0 {
            res.push(Self {
                corner: self.corner.offset(0, width),
                width: remainder_width,
                height
            });
        }

        res
    }

    pub fn corner(&self) -> PixelCoord {
        self.corner
    }
}

pub struct TileEntry {
    location: PixelCoord,
    width: u32,
    height: u32,
    texture_handle: TexHandle
}

impl TileEntry {
    pub fn new(location: PixelCoord, width: u32, height: u32, texture_handle: TexHandle) -> Self {
        Self { location, width, height, texture_handle }
    }
    
    pub fn new_from_placement(location: PixelCoord, width: u32, height: u32, tex: Tex) -> Self {
        let uv_rect = UVRect::sub_texture(location, (width, height), &tex);
        
        Self {
            location,
            width,
            height,
            texture_handle: TexHandle::new(tex, uv_rect)
        }
    }

    pub fn location(&self) -> PixelCoord {
        self.location
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
    
    pub fn handle(&self) -> TexHandle {
        self.texture_handle.clone()
    }
}