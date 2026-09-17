use std::cell::RefCell;
use std::rc::Rc;
use crate::graphics::tex::Tex;
use crate::graphics::texture::uv_rect::UVRect;

#[derive(Clone)]
struct TexHandleData {
    texture: Tex,
    uv_rect: UVRect,
    is_valid: bool
}

#[derive(Clone)]
pub struct TexHandle {
    data: Rc<RefCell<TexHandleData>>
}

impl TexHandle {
    pub fn snapshot(&self) -> (Tex, UVRect) {
        let data = self.data.borrow();
        (data.texture.clone(), data.uv_rect)
    }
}

#[derive(Debug)]
pub struct TileCantFitInAtlasError;

impl std::fmt::Display for TileCantFitInAtlasError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "The tile can't fit in the atlas")
    }
}

impl std::error::Error for TileCantFitInAtlasError { }

/*
pub struct TexAtl {
    device: wgpu::Device,
    queue: wgpu::Queue,
    texture: Tex,
    texture_read_buffer: wgpu::Buffer,
    texture_write_buffer: wgpu::Buffer,
    tiles: Vec<TexHandle>
}

impl TexAtl {
    fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        const STARTING_BUFFER_SIZE: u64 = (constants::ATLAS_MIN_SIZE as u64).pow(2) * 4;

        let texture = Tex::new(device, queue, &[0u8], constants::ATLAS_MIN_SIZE, constants::ATLAS_MIN_SIZE);

        let texture_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Texture atlas buffer"),
            size: STARTING_BUFFER_SIZE,
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            device: device.clone(),
            queue: queue.clone(),
            texture,
            texture_buffer,
            tiles: Vec::new()
        }
    }

    fn add_tile(&mut self, tile: &ImageData) -> Result<TexHandle, TileCantFitInAtlasError> {
        self.texture.copy_to_buffer(&self.device, &self.queue, &self.texture_buffer);


        Ok(TexHandle {})
    }

    fn add_tiles(&mut self, tiles: &[&ImageData]) -> Result<Vec<TexHandle>, ()> {
        Ok(vec![TexHandle {}])
    }

    fn remove_tile(&mut self, tex_handle: TexHandle) {

    }

    fn remove_tiles(&mut self, tex_handles: &[TexHandle]) {

    }

    fn pack(&mut self) {

    }
}*/