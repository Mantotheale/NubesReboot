use wgpu::RenderPass;
use crate::constants;
use crate::math::rect2f::Rect2f;
use crate::renderer::colored_rect::ColoredRectVertex;
use crate::renderer::colored_segment::ColoredSegmentVertex;
use crate::renderer::Fill;

pub struct RectBatch {
    batch: Vec<(Rect2f, Fill)>,
    queue: wgpu::Queue,
    vertex_buffer: wgpu::Buffer,
    cpu_buffer: [u8; constants::RECTS_MAX_BATCH_SIZE * ColoredRectVertex::byte_size()],
    index_buffer: wgpu::Buffer,
    texture_bind_group: Option<wgpu::BindGroup>,
    pipeline: wgpu::RenderPipeline,
}

impl RectBatch {
    pub fn new(device: wgpu::Device, queue: wgpu::Queue, surface_format: wgpu::TextureFormat) -> Self {
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Rect batch vertex buffer"),
            size: (constants::RECTS_MAX_BATCH_SIZE * ColoredRectVertex::byte_size() * ColoredRectVertex::VERTICES_PER_RECT) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Rect batch index buffer"),
            size: (constants::RECTS_MAX_BATCH_SIZE * ColoredRectVertex::INDICES_PER_RECT * size_of::<u32>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut indices = Vec::new();
        for n in 0..constants::RECTS_MAX_BATCH_SIZE {
            for idx in ColoredRectVertex::PRIMITIVE_INDICES {
                let base_idx = n * ColoredRectVertex::VERTICES_PER_RECT;
                indices.push((base_idx + idx) as u32);
            }
        }

        queue.write_buffer(&index_buffer, 0, bytemuck::cast_slice(&indices));

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Rect batch shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../resources/shaders/rect_shader.wgsl").into()),
        });

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Rect batch pipeline layout"),
                bind_group_layouts: &[],
                immediate_size: 0,
            });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Rect batch pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(ColoredRectVertex::desc())],
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
            batch: Vec::new(),
            queue,
            vertex_buffer,
            cpu_buffer: [0; constants::RECTS_MAX_BATCH_SIZE * ColoredRectVertex::byte_size()],
            index_buffer,
            texture_bind_group: None,
            pipeline
        }
    }

    pub fn push(&mut self, rect: Rect2f, fill: Fill) {
        self.batch.push((rect, fill))
    }

    pub fn draw(&mut self, render_pass: &mut RenderPass) {
        let mut cpu_buffer_idx = 0;
        for (rect, fill) in &self.batch {
            match fill {
                Fill::Color(color) => {
                    let vertex_data = ColoredRectVertex::generate(*rect, *color);
                    self.cpu_buffer[cpu_buffer_idx..cpu_buffer_idx + ColoredRectVertex::VERTICES_PER_RECT * ColoredRectVertex::byte_size()].copy_from_slice(bytemuck::cast_slice(&vertex_data));
                }
                Fill::TextureView(_) => unimplemented!()
            }

            cpu_buffer_idx += ColoredRectVertex::VERTICES_PER_RECT * ColoredRectVertex::byte_size();
        }

        self.queue.write_buffer(&self.vertex_buffer, 0, &self.cpu_buffer);

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        render_pass.draw_indexed(
            0..(self.batch.len() * ColoredRectVertex::INDICES_PER_RECT) as u32,
            0,
            0..1
        );
    }

    pub fn clear(&mut self) {
        self.batch.clear();
    }
}