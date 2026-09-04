use std::collections::HashMap;
use std::fmt::Pointer;
use crate::constants;
use crate::math::rect2f::Rect2f;
use crate::renderer::Fill;
use crate::renderer::rect::RectVertex;
use crate::renderer::texture::{Texture, TextureId};

pub struct RectBatch {
    inserted_rects: usize,
    queue: wgpu::Queue,
    vertex_buffer: wgpu::Buffer,
    cpu_buffer: [u8; constants::RECTS_MAX_BATCH_SIZE * RectVertex::RECT_BYTE_SIZE],
    index_buffer: wgpu::Buffer,
    texture_pool: TexturePool,
    pipeline: wgpu::RenderPipeline,
}

impl RectBatch {
    pub fn new(device: wgpu::Device, queue: wgpu::Queue, surface_format: wgpu::TextureFormat) -> Self {
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Rect batch vertex buffer"),
            size: (constants::RECTS_MAX_BATCH_SIZE * RectVertex::RECT_BYTE_SIZE) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Rect batch index buffer"),
            size: (constants::RECTS_MAX_BATCH_SIZE * RectVertex::INDICES_PER_RECT * size_of::<u32>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut indices = Vec::new();
        for n in 0..constants::RECTS_MAX_BATCH_SIZE {
            for idx in RectVertex::PRIMITIVE_INDICES {
                let base_idx = n * RectVertex::VERTICES_PER_RECT;
                indices.push((base_idx + idx) as u32);
            }
        }

        queue.write_buffer(&index_buffer, 0, bytemuck::cast_slice(&indices));

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Rect batch shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../resources/shaders/rect_shader.wgsl").into()),
        });

        let texture_pool = TexturePool::new(device.clone());

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Rect batch pipeline layout"),
                bind_group_layouts: &[Some(texture_pool.get_group_layout())],
                immediate_size: 0,
            });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Rect batch pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(RectVertex::desc())],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        Self {
            inserted_rects: 0,
            queue,
            vertex_buffer,
            cpu_buffer: [0; constants::RECTS_MAX_BATCH_SIZE * RectVertex::RECT_BYTE_SIZE],
            index_buffer,
            texture_pool,
            pipeline
        }
    }

    pub fn push(&mut self, rect: Rect2f, fill: Fill) -> Result<(), RectBatchPushError> {
        if self.inserted_rects == constants::RECTS_MAX_BATCH_SIZE { return Err(RectBatchPushError::BatchFull) }
        
        let insertion_idx = self.inserted_rects * RectVertex::RECT_BYTE_SIZE;

        match fill {
            Fill::Color(color) => {
                let vertex_data = RectVertex::from_colored_rect(rect, color);
                self.cpu_buffer[insertion_idx..insertion_idx + RectVertex::RECT_BYTE_SIZE]
                    .copy_from_slice(bytemuck::cast_slice(&vertex_data));
            }
            Fill::TextureView(texture) => {
                match self.texture_pool.push(texture.clone()) {
                    Ok(tex_slot) => {
                        let vertex_data = RectVertex::from_textured_rect(rect, tex_slot, texture.tex_coords());
                        self.cpu_buffer[insertion_idx..insertion_idx + RectVertex::RECT_BYTE_SIZE]
                            .copy_from_slice(bytemuck::cast_slice(&vertex_data));
                    }
                    Err(err) => return Err(RectBatchPushError::TexturePoolFull { err })
                }
            }
        }

        self.inserted_rects += 1;
        Ok(())
    }

    pub fn draw(&mut self, render_pass: &mut wgpu::RenderPass) {
        self.queue.write_buffer(&self.vertex_buffer, 0, &self.cpu_buffer);

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.set_bind_group(0, self.texture_pool.get_bind_group(), &[]);

        render_pass.draw_indexed(
            0..(self.inserted_rects * RectVertex::INDICES_PER_RECT) as u32,
            0,
            0..1
        );
    }

    pub fn clear(&mut self) {
        self.inserted_rects = 0;
        self.texture_pool.reset();
    }
}

struct TexturePool {
    device: wgpu::Device,
    bound_textures: HashMap<TextureId, usize>,
    texture_pool: [(TextureId, wgpu::TextureView); constants::TEXTURE_SLOTS],
    bind_group: wgpu::BindGroup,
    group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    has_pool_changed: bool,
    null_texture: (TextureId, wgpu::TextureView)
}

impl TexturePool {
    pub fn new(device: wgpu::Device) -> Self {
        let group_layout = Self::gen_group_layout(&device);

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Texture pool sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let null_texture = device.create_texture(&wgpu::TextureDescriptor {
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
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let null_texture = null_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let null_id = TextureId::new();

        let texture_pool = [
            (null_id, null_texture.clone()),
            (null_id, null_texture.clone()),
            (null_id, null_texture.clone()),
            (null_id, null_texture.clone()),
            (null_id, null_texture.clone()),
            (null_id, null_texture.clone()),
            (null_id, null_texture.clone()),
            (null_id, null_texture.clone()),
            (null_id, null_texture.clone()),
            (null_id, null_texture.clone()),
            (null_id, null_texture.clone()),
            (null_id, null_texture.clone()),
            (null_id, null_texture.clone()),
            (null_id, null_texture.clone()),
            (null_id, null_texture.clone()),
            (null_id, null_texture.clone()),
        ];

        let bind_group = Self::gen_bind_group(&device, &group_layout, &texture_pool, &sampler);

        Self {
            device,
            bound_textures: HashMap::new(),
            texture_pool,
            bind_group,
            group_layout,
            sampler,
            has_pool_changed: false,
            null_texture: (null_id, null_texture)
        }
    }

    fn push(&mut self, texture: Texture) -> Result<usize, TexturePoolFullError> {
        if self.bound_textures.len() == constants::TEXTURE_SLOTS { Err(TexturePoolFullError { texture }) }
        else {
            match self.bound_textures.get(&texture.id()) {
                Some(index) => Ok(*index),
                None => {
                    let index = self.bound_textures.len();
                    self.texture_pool[index] = (texture.id(), texture.wgpu_texture().clone());
                    self.bound_textures.insert(texture.id(), index);
                    self.has_pool_changed = true;
                    Ok(index)
                }
            }
        }
    }

    fn reset(&mut self) {
        self.bound_textures.clear();
        for i in 0..constants::TEXTURE_SLOTS {
            self.texture_pool[i] = self.null_texture.clone();
        }
        self.has_pool_changed = true;
    }

    fn get_bind_group(&mut self) -> &wgpu::BindGroup {
        if self.has_pool_changed {
            self.bind_group = Self::gen_bind_group(&self.device, &self.group_layout, &self.texture_pool, &self.sampler);
        }

        &self.bind_group
    }

    fn get_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.group_layout
    }

    const fn layout_entry(index: usize) -> wgpu::BindGroupLayoutEntry {
        if index > constants::TEXTURE_SLOTS { panic!("Entry indices are bound by the texture slot") }

        let binding_type = if index == constants::TEXTURE_SLOTS {
            wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering)
        } else {
            wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false
            }
        };

        wgpu::BindGroupLayoutEntry {
            binding: index as u32,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: binding_type,
            count: None,
        }
    }

    fn gen_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Texture pool bind group layout"),
                entries: &(0..=constants::TEXTURE_SLOTS)
                    .map(|index| Self::layout_entry(index))
                    .collect::<Vec<wgpu::BindGroupLayoutEntry>>()
            })
    }

    fn gen_bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        textures: &[(TextureId, wgpu::TextureView)],
        sampler: &wgpu::Sampler
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Texture Pool bind group"),
            layout,
            entries: &textures.iter().enumerate()
                .map(|(index, (_, tex))|
                    wgpu::BindGroupEntry {
                        binding: index as u32,
                        resource: wgpu::BindingResource::TextureView(tex)
                    }
                ).chain(std::iter::once(
                    wgpu::BindGroupEntry {
                        binding: constants::TEXTURE_SLOTS as u32,
                        resource: wgpu::BindingResource::Sampler(sampler)
                    }
                )).collect::<Vec<wgpu::BindGroupEntry>>()
        })
    }
}

#[derive(Debug)]
pub enum RectBatchPushError {
    BatchFull,
    TexturePoolFull { err: TexturePoolFullError }
}

impl std::fmt::Display for RectBatchPushError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RectBatchPushError::BatchFull => write!(f, "The rect batch is full"),
            RectBatchPushError::TexturePoolFull { err } => err.fmt(f)
        }
    }
}

impl std::error::Error for RectBatchPushError { }

#[derive(Debug)]
struct TexturePoolFullError {
    texture: Texture
}

impl std::fmt::Display for TexturePoolFullError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "The texture pool is full. Texture with id {:?} could not be added", self.texture.id())
    }
}

impl std::error::Error for TexturePoolFullError { }