mod structures;

use crate::constants;
use crate::graphics::tex::atlas::structures::{EmptySpace, TileEntry};
use crate::graphics::tex::Tex;
use crate::graphics::tex::texture_handle::TexHandle;
use crate::util::image_utils::ImageData;

pub use crate::graphics::tex::atlas::structures::PixelCoord;
use crate::graphics::texture::uv_rect::UVRect;

#[derive(Debug)]
pub struct TileCantFitInAtlasError;

impl std::fmt::Display for TileCantFitInAtlasError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "The tile can't fit in the atlas")
    }
}

impl std::error::Error for TileCantFitInAtlasError { }


pub struct TexAtl {
    device: wgpu::Device,
    queue: wgpu::Queue,
    texture: Tex,
    tiles: Vec<TileEntry>,
    empty_spaces: Vec<EmptySpace>,
    size: u32
}

impl TexAtl {
    fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        const SIZE: u32 = constants::ATLAS_MIN_SIZE as u32;
        const STARTING_BUFFER_SIZE: u32 = SIZE.pow(2) * 4;

        let texture = Tex::new(
            device,
            queue,
            &ImageData::from_data(&vec![0; STARTING_BUFFER_SIZE as usize], SIZE, SIZE).expect("Image has RGBA channels")
        ).expect("Valid size");

        Self {
            device: device.clone(),
            queue: queue.clone(),
            texture,
            tiles: Vec::new(),
            empty_spaces: vec![EmptySpace::starting_space(SIZE, SIZE)],
            size: SIZE
        }
    }

    pub fn add_tile(&mut self, tile: &ImageData) -> Result<TexHandle, TileCantFitInAtlasError> {
        Ok(self.add_tiles(&[tile])?[0].clone())
    }

    pub fn add_tiles(&mut self, tiles: &[&ImageData]) -> Result<Vec<TexHandle>, TileCantFitInAtlasError> {
        let (placements, empty_spaces) = Self::place_tiles(tiles, &self.empty_spaces)?;

        let atlas_image = self.texture.texture_content(&self.device, &self.queue);
        let mut atlas_image = ImageData::from_data(&atlas_image, self.size, self.size)
            .expect("The image supports RGBA");

        let mut tex_handles = Vec::new();
        for i in 0..placements.len() {
            let placement = placements[i];
            let tile_data = tiles[i];

            atlas_image.copy_with_offset(placement.row(), placement.col(), tile_data)
                .expect("Correct pixel bounds");

            let entry = TileEntry::new_from_placement(
                placement,
                tile_data.width(),
                tile_data.height(),
                self.texture.clone()
            );

            tex_handles.push(entry.handle());
            self.tiles.push(entry);
        }

        self.empty_spaces = empty_spaces;
        self.texture.replace_content(&self.device, &self.queue, &atlas_image).expect("Matching texture size");
        Ok(tex_handles)
    }

    fn remove_tile(&mut self, tex_handle: TexHandle) {
        if let Some(index) = self.tiles.iter()
            .position(|e| e.handle().id() == tex_handle.id()) {
            self.tiles.remove(index).handle().invalidate();
        }
    }

    fn remove_tiles(&mut self, tex_handles: &[TexHandle]) {
        tex_handles.iter().for_each(|t| self.remove_tile(t.clone()));
    }

    fn pack(&mut self) -> Result<(), TileCantFitInAtlasError> {
        let contained_tiles = self.contained_tiles();

        let (placements, empty_spaces) = Self::place_tiles(
            &contained_tiles.iter().collect::<Vec<_>>(),
            &vec![EmptySpace::starting_space(self.size, self.size)]
        )?;

        let mut new_tiles = Vec::new();
        let mut atlas_image = ImageData::blank_image(self.size, self.size);

        for i in 0..placements.len() {
            let placement = placements[i];
            let tile = contained_tiles.get(i).expect("Valid index");
            let old_entry = self.tiles.get(i).expect("Valid index");

            atlas_image.copy_with_offset(placement.row(), placement.col(), tile).expect("Valid location");

            let tex_handle = old_entry.handle();
            tex_handle.update(
                self.texture.clone(),
                UVRect::sub_texture(
                    placement,
                    (tile.width(), tile.height()),
                    &self.texture)
            );

            new_tiles.push(TileEntry::new(placement, tile.width(), tile.height(), tex_handle));
        }

        self.texture.replace_content(&self.device, &self.queue, &atlas_image).expect("Matching texture size");
        self.empty_spaces = empty_spaces;
        self.tiles = new_tiles;
        Ok(())
    }

    fn contained_tiles(&self) -> Vec<ImageData> {
        let atlas_data = self.texture.texture_content(&self.device, &self.queue);
        let atlas_image = ImageData::from_data(&atlas_data, self.size, self.size).expect("The image supports RGBA");

        self.tiles.iter()
            .map(|entry| {
                let location = entry.location();
                let sub_image = atlas_image.sub_image(location.row(), location.col(), entry.width(), entry.height())
                    .expect("Valid pixel bounds");
                sub_image
            })
            .collect::<Vec<_>>()
    }

    fn tile_ordering(tile_a: &ImageData, tile_b: &ImageData) -> std::cmp::Ordering {
        u32::cmp(&tile_b.height(), &tile_a.height())
            .then(u32::cmp(&tile_b.width(), &tile_a.width()))
    }

    fn place_tiles(tiles: &[&ImageData], empty_spaces: &Vec<EmptySpace>)
                   -> Result<(Vec<PixelCoord>, Vec<EmptySpace>), TileCantFitInAtlasError> {
        let mut indexed_tiles = tiles.iter().enumerate().collect::<Vec<(usize, &&ImageData)>>();
        indexed_tiles.sort_by(
            |(_, image_a), (_, image_b)| Self::tile_ordering(image_a, image_b)
        );

        let mut placements = Vec::new();
        let mut empty_spaces = empty_spaces.clone();

        for (_, tile) in &indexed_tiles {
            let mut inserted = false;

            for i in (0..empty_spaces.len()).rev() {
                if empty_spaces.get(i).expect("Valid index").can_image_fit(tile.width(), tile.height()) {
                    let empty_space = empty_spaces.remove(i);

                    for s in empty_space.split_space(tile.width(), tile.height()) {
                        empty_spaces.push(s);
                    }

                    placements.push(empty_space.corner());
                    inserted = true;
                    break;
                }
            }

            if !inserted {
                return Err(TileCantFitInAtlasError);
            }
        }

        let mut placements_original_order = vec![None; placements.len()];
        for i in 0..placements.len() {
            let (index, _) = indexed_tiles.get(i).expect("Valid index");
            placements_original_order[*index] = Some(placements[i]);
        }

        Ok((placements_original_order.into_iter().flatten().collect::<Vec<_>>(), empty_spaces))
    }
}