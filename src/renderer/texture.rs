use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};
use crate::constants;
use crate::renderer::tex_coords::RectTexCoords;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Ord, PartialOrd)]
pub struct TextureId {
    id: u32
}

impl TextureId {
    pub const NULL: Self = Self { id: 0 };

    pub fn new() -> Self {
        static NEXT_ID: AtomicU32 = AtomicU32::new(1);
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
    pub fn null(device: &wgpu::Device) -> Self {
        static NULL_VIEW: std::sync::OnceLock<wgpu::TextureView> = std::sync::OnceLock::new();

        let view = NULL_VIEW.get_or_init(|| {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });

            texture.create_view(&wgpu::TextureViewDescriptor::default())
        });

        Self {
            id: TextureId::NULL,
            texture: view.clone(),
            tex_coords: RectTexCoords::DEFAULT_COORDS
        }
    }

    pub fn new(id: TextureId, texture: wgpu::TextureView, tex_coords: RectTexCoords) -> Self {
        Self { id, texture, tex_coords }
    }

    pub fn from_path(path: &Path, device: &wgpu::Device, queue: &wgpu::Queue) -> Result<Self, ()> {
        let file = constants::RESOURCE_DIR.get_file(path).ok_or(())?;
        let image_bytes = file.contents();
        let image = image::load_from_memory(image_bytes).map_err(|_| ())?.flipv();
        let image_rgba = image.as_rgba8().ok_or(())?;


        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: image.width(),
                height: image.height(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            image_rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(image.width() * 4),
                rows_per_image: Some(image.height()),
            },
            wgpu::Extent3d {
                width: image.width(),
                height: image.height(),
                depth_or_array_layers: 1,
            }
        );

        let texture_view: wgpu::TextureView = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Ok(Self {
            id: TextureId::new(),
            texture: texture_view,
            tex_coords: RectTexCoords::DEFAULT_COORDS,
        })
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