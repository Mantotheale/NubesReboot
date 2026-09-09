use crate::{
    constants,
    color::Color,
    math::positive_f32::PositiveF32,
    math::segment2f::Segment2f,
    renderer::segment::SegmentVertex
};

pub struct SegmentBatch {
    inserted_segments: usize,
    queue: wgpu::Queue,
    vertex_buffer: wgpu::Buffer,
    cpu_buffer: [u8; constants::SEGMENTS_MAX_BATCH_SIZE * SegmentVertex::SEGMENT_BYTE_SIZE],
    index_buffer: wgpu::Buffer,
    screen_dimensions: (u32, u32),
    screen_dimensions_buffer: wgpu::Buffer,
    screen_dimensions_bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

impl SegmentBatch {
    pub fn new(device: wgpu::Device, queue: wgpu::Queue, surface_format: wgpu::TextureFormat) -> Self {
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Segment batch vertex buffer"),
            size: (constants::SEGMENTS_MAX_BATCH_SIZE * SegmentVertex::SEGMENT_BYTE_SIZE) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Segment batch index buffer"),
            size: (constants::SEGMENTS_MAX_BATCH_SIZE * SegmentVertex::INDICES_PER_SEGMENT * size_of::<u32>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut indices = Vec::new();
        for n in 0..constants::SEGMENTS_MAX_BATCH_SIZE {
            for idx in SegmentVertex::PRIMITIVE_INDICES {
                let base_idx = n * SegmentVertex::VERTICES_PER_SEGMENT;
                indices.push((base_idx + idx) as u32);
            }
        }

        queue.write_buffer(&index_buffer, 0, bytemuck::cast_slice(&indices));

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Segment batch shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../../resources/shaders/segment_shader.wgsl").into()),
        });

        let screen_dimensions_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (2 * size_of::<u32>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        queue.write_buffer(&screen_dimensions_buffer, 0, bytemuck::cast_slice(&[0, 0]));
        
        let screen_dimensions_binding_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: None,
            });

        let screen_dimensions_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &screen_dimensions_binding_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &screen_dimensions_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
            label: None,
        });
        
        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Segment batch pipeline layout"),
                bind_group_layouts: &[Some(&screen_dimensions_binding_layout)],
                immediate_size: 0,
            });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Segment batch pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(SegmentVertex::desc())],
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
            inserted_segments: 0,
            queue,
            vertex_buffer,
            cpu_buffer: [0; constants::SEGMENTS_MAX_BATCH_SIZE * SegmentVertex::SEGMENT_BYTE_SIZE],
            index_buffer,
            screen_dimensions: (0, 0),
            screen_dimensions_buffer,
            screen_dimensions_bind_group,
            pipeline
        }
    }

    pub fn push(&mut self, segment: Segment2f, color: Color, pixel_width: PositiveF32) -> Result<(), SegmentBatchFullError> {
        if self.inserted_segments == constants::SEGMENTS_MAX_BATCH_SIZE { return Err(SegmentBatchFullError) }

        let insertion_idx = self.inserted_segments * SegmentVertex::SEGMENT_BYTE_SIZE;

        let vertex_data = SegmentVertex::new(segment, color, pixel_width);
        self.cpu_buffer[insertion_idx..insertion_idx + SegmentVertex::SEGMENT_BYTE_SIZE]
            .copy_from_slice(bytemuck::cast_slice(&vertex_data));
        
        self.inserted_segments += 1;
        Ok(())
    }

    pub fn draw(&mut self, render_pass: &mut wgpu::RenderPass, screen_dimensions: (u32, u32)) {
        self.queue.write_buffer(&self.vertex_buffer, 0, &self.cpu_buffer);

        if screen_dimensions != self.screen_dimensions {
            self.queue.write_buffer(
                &self.screen_dimensions_buffer, 
                0, 
                bytemuck::cast_slice(&[screen_dimensions.0, screen_dimensions.1])
            );
        }

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.set_bind_group(0, &self.screen_dimensions_bind_group, &[]);

        render_pass.draw_indexed(
            0..(self.inserted_segments * SegmentVertex::INDICES_PER_SEGMENT) as u32,
            0,
            0..1
        );
    }

    pub fn clear(&mut self) {
        self.inserted_segments = 0;
    }
}

#[derive(Debug)]
pub struct SegmentBatchFullError;

impl std::fmt::Display for SegmentBatchFullError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "The segment batch is full")
    }
}

impl std::error::Error for SegmentBatchFullError { }