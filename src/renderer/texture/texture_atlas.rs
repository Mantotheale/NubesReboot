use std::collections::HashMap;
use crate::util::image_utils::ImageData;
use std::path::{Path, PathBuf};
use image::{EncodableLayout, ImageBuffer};
use crate::constants;
use crate::renderer::texture::Texture;
use crate::renderer::texture::texture_handle::TextureHandle;
use crate::renderer::texture::uv_rect::{TexCoord, UVOffset, UVRect};

pub struct TextureAtlas {
    tiles: HashMap<PathBuf, TextureHandle>
}

impl TextureAtlas {
    pub fn get_tile(&self, path: &Path) -> Option<&TextureHandle> {
        self.tiles.get(path)
    }

    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, tiles: &[(&Path, ImageData)]) -> Result<Self, ()> {
        let mut tiles = tiles.iter()
            .map(|(path, data)| (*path, data))
            .collect::<Vec<_>>();

        tiles.sort_by(
            |(_, image_a), (_, image_b)|
                u32::cmp(&image_b.height(), &image_a.height())
                    .then(u32::cmp(&image_b.width(), &image_a.width()))
        );

        let mut atlas_size = 32;
        let (placements, atlas_size) = loop {
            if atlas_size > constants::MAX_TEXTURE_SIZE as u32 { return Err(()) }

            if let Ok(placements) = Self::generate_placements(&tiles, atlas_size) {
                break (placements, atlas_size);
            }

            atlas_size *= 2;
        };

        let texture_handles = Self::generate_atlas(device, queue, &placements, &tiles, atlas_size);
        Ok(Self { tiles: texture_handles })
    }

    fn generate_placements<'a>(tiles: &[(&'a Path, &ImageData)], atlas_size: u32) -> Result<Vec<(&'a Path, Placement)>, ()> {
        let mut empty_spaces = vec![EmptySpace::starting_space(atlas_size, atlas_size)];
        let mut placements = Vec::new();

        for (path, image) in tiles {
            let mut placement = None;

            for i in (0..empty_spaces.len()).rev() {
                let empty_space = empty_spaces[i];
                if !empty_space.can_image_fit(image.width(), image.height()) { continue; }

                empty_spaces.remove(i);

                for s in empty_space.split_space(image.width(), image.height()) {
                    empty_spaces.push(s);
                }

                placement = Some(empty_space.placement);
                break;
            }

            match placement {
                None => return Err(()),
                Some(placement) => placements.push((*path, placement))
            };
        }

        Ok(placements)
    }

    fn generate_atlas(device: &wgpu::Device, queue: &wgpu::Queue, placements: &[(&Path, Placement)], images: &[(&Path, &ImageData)], atlas_size: u32)
        -> HashMap<PathBuf, TextureHandle> {
        let mut atlas = image::RgbaImage::new(atlas_size, atlas_size);
        let mut uv_rects = Vec::new();

        for i in 0..placements.len() {
            let placement = placements.get(i).unwrap().1;
            let image = images.get(i).unwrap().1;

            Self::insert_image(&mut atlas, placement, image);
            uv_rects.push(UVRect::from_corner_and_offset(
                TexCoord::new( placement.col as f32 / atlas_size as f32, placement.row as f32 / atlas_size as f32),
                UVOffset::new(image.width() as f32 / atlas_size as f32, image.height() as f32 / atlas_size as f32)
            ))
        }

        let texture = Texture::new(device, queue, atlas.as_bytes(), atlas_size, atlas_size);

        let mut texture_handles = HashMap::new();
        for i in 0..placements.len() {
            texture_handles.insert(
                PathBuf::from(placements.get(i).unwrap().0),
                TextureHandle::new(&texture, uv_rects[i])
            );
        }

        image::imageops::flip_vertical_in_place(&mut atlas);
        atlas.save("C:\\Users\\a-mantonico\\Downloads\\ciao.png").unwrap();
        
        texture_handles
    }

    fn insert_image(atlas: &mut ImageBuffer<image::Rgba<u8>, Vec<u8>>, placement: Placement, image_data: &ImageData) {
        for row in 0..image_data.height() {
            for col in 0..image_data.width() {
                let atlas_row = placement.row + row;
                let atlas_col = placement.col + col;

                atlas.put_pixel(atlas_col, atlas_row, image_data.get_pixel(row, col).unwrap());
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Placement {
    row: u32,
    col: u32
}

impl Placement {
    fn new(row: u32, col: u32) -> Self {
        Self { row, col }
    }

    fn offset(&self, row_offset: u32, col_offset: u32) -> Self {
        Self { row: self.row + row_offset, col: self.col + col_offset }
    }
}

#[derive(Debug, Copy, Clone)]
struct EmptySpace {
    placement: Placement,
    width: u32,
    height: u32
}

impl EmptySpace {
    fn starting_space(width: u32, height: u32) -> Self {
        Self { placement: Placement::new(0, 0), width, height }
    }

    fn can_image_fit(&self, width: u32, height: u32) -> bool {
        width <= self.width && height <= self.height
    }

    fn split_space(&self, width: u32, height: u32) -> Vec<Self> {
        if !self.can_image_fit(width, height) { return Vec::new(); }

        let remainder_width = self.width - width;
        let remainder_height = self.height - height;

        let mut res = Vec::new();
        if remainder_height > 0 {
            res.push(Self {
                placement: self.placement.offset(height, 0),
                width: self.width,
                height: remainder_height
            });
        }

        if remainder_width > 0 {
            res.push(Self {
                placement: self.placement.offset(0, width),
                width: remainder_width,
                height
            });
        }

        res
    }
}