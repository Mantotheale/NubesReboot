use std::sync::atomic::{AtomicU32, Ordering};
use crate::renderer::tex_coords::RectTexCoords;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct TextureId {
    id: u32
}

impl TextureId {
    pub fn new() -> Self {
        static NEXT_ID: AtomicU32 = AtomicU32::new(0);
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        Self { id }
    }
}

#[derive(Clone, Debug)]
pub struct Texture {
    id: TextureId,
    texture: wgpu::TextureView,
    tex_coords: RectTexCoords,
}

impl Texture {
    pub fn new(id: TextureId, texture: wgpu::TextureView, tex_coords: RectTexCoords) -> Self {
        Self { id, texture, tex_coords }
    }

    pub fn id(&self) -> TextureId {
        self.id
    }
    
    pub fn tex_coords(&self) -> RectTexCoords {
        self.tex_coords
    }

    pub fn wgpu_texture(&self) -> &wgpu::TextureView {
        &self.texture
    }
}