mod segment;

use std::num::NonZeroU8;
use std::sync::Arc;
use wgpu::CurrentSurfaceTexture;
use crate::color::Color;
use crate::constants;
use crate::engine::{InitializationError};
use crate::math::unit_f32::UnitF32;
use crate::renderer::segment::ColoredSegmentVertex;

#[derive(Debug)]
pub enum BeginSceneError {
    ValidationError,
    FrameTimeoutError,
    OccludedSurfaceError,
    OutdatedConfigError,
    LostSurfaceError
}

impl std::fmt::Display for BeginSceneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            BeginSceneError::ValidationError => "Validation error occurred while retrieving the render texture, wait until validation occurs",
            BeginSceneError::FrameTimeoutError => "Timeout occurred while trying to retrieve the next texture, skip it and retry later",
            BeginSceneError::OccludedSurfaceError => "The window is occluded, so skip the rendering",
            BeginSceneError::OutdatedConfigError => "The surface changed, update the config and retry again this frame",
            BeginSceneError::LostSurfaceError => "The surface is lost, the render system should be reset"
        };

        write!(f, "{}", msg)
    }
}

impl std::error::Error for BeginSceneError { }

struct Line {

}

struct Triangle {

}

struct Square {

}

struct Circle {

}

struct TextureView {

}

enum Fill {
    Color(Color),
    TextureView(TextureView)
}

pub struct IdleRenderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    screen_dimensions_bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
    clear_color: Color
}

impl IdleRenderer {
    pub async fn new(window: Arc<winit::window::Window>) -> Result<Self, InitializationError> {
        let instance = wgpu::Instance::new(
            wgpu::InstanceDescriptor::new_with_display_handle(
                Box::new(window.clone())
            )
        );
        println!("{:?}", instance);

        let surface = instance.create_surface(window.clone())
            .map_err(|err| InitializationError::CreateSurfaceError(err))?;
        println!("{:?}", surface);

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
            apply_limit_buckets: true,
        }).await.map_err(|err| InitializationError::AdapterError(err))?;
        println!("{:?}", adapter);
        println!("CIAO");

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::empty(),
            required_limits: Default::default(),
            experimental_features: Default::default(),
            memory_hints: Default::default(),
            trace: Default::default(),
        }).await.map_err(|err| InitializationError::RequestDeviceError(err))?;

        let surface_caps = surface.get_capabilities(&adapter);
        println!("{:?}", surface_caps);

        let surface_format = *surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .ok_or(InitializationError::NoSRGBSurface)?;
        println!("Surface format\n{:?}", surface_format);

        let window_size = window.inner_size();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            color_space: wgpu::SurfaceColorSpace::Auto,
            width: window_size.width,
            height: window_size.height,
            present_mode: wgpu::PresentMode::AutoNoVsync,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (constants::MAX_SEGMENTS_BATCH * ColoredSegmentVertex::byte_size() * ColoredSegmentVertex::VERTICES_PER_SEGMENT) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (constants::MAX_SEGMENTS_BATCH * ColoredSegmentVertex::INDICES_PER_SEGMENT * size_of::<u32>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut indices = Vec::new();
        for n in 0..constants::MAX_SEGMENTS_BATCH {
            for idx in ColoredSegmentVertex::PRIMITIVE_INDICES {
                let base_idx = n * ColoredSegmentVertex::INDICES_PER_SEGMENT;
                indices.push((base_idx + idx) as u32);
            }
        }

        queue.write_buffer(&index_buffer, 0, bytemuck::cast_slice(&indices));

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../resources/shaders/simple_shader.wgsl").into()),
        });

        let screen_dimensions_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (2 * size_of::<u32>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let screen_dimensions_binding_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
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

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[Some(&screen_dimensions_binding_layout)],
                immediate_size: 0,
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(ColoredSegmentVertex::desc())],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
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

        Ok(Self {
            surface,
            device,
            queue,
            vertex_buffer,
            index_buffer,
            screen_dimensions_bind_group,
            pipeline: render_pipeline,
            clear_color: Color::SOLID_BLACK
        })
    }

    pub fn set_clear_color(&mut self, color: Color) {
        self.clear_color = color;
    }

    fn begin_scene(self) -> Result<InProgressRenderer, BeginSceneError> {
        let output = match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(t) | CurrentSurfaceTexture::Suboptimal(t) => t,
            CurrentSurfaceTexture::Timeout => return Err(BeginSceneError::FrameTimeoutError),
            CurrentSurfaceTexture::Occluded => return Err(BeginSceneError::OccludedSurfaceError),
            CurrentSurfaceTexture::Validation => return Err(BeginSceneError::ValidationError),
            CurrentSurfaceTexture::Outdated => return Err(BeginSceneError::OutdatedConfigError),
            CurrentSurfaceTexture::Lost => return Err(BeginSceneError::LostSurfaceError)
        };

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("render pass"),
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color.into()),
                        store: wgpu::StoreOp::Store
                    },
                })
            ],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        Ok(InProgressRenderer {
            device: self.device,
            queue: self.queue,
            vertex_buffer: self.vertex_buffer,
            index_buffer: self.index_buffer,
            screen_dimensions_bind_group: self.screen_dimensions_bind_group,
            pipeline: self.pipeline,
            render_pass
        })
    }

    fn resize(&mut self) {

    }

    fn swap_buffers(&self) {

    }
}

struct InProgressRenderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    screen_dimensions_bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
    clear_color: Color,

}

impl InProgressRenderer {
    fn add_line(&mut self, line: Line, fill: Fill, pixel_width: NonZeroU8) {

    }

    fn add_triangle(&mut self, triangle: Triangle, fill: Fill) {

    }

    fn add_square(&mut self, square: Square, fill: Fill) {

    }

    fn add_circle(&mut self, circle: Circle, fill: Fill) {

    }

    fn end_scene(self) -> IdleRenderer {

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.as_wgpu_buffer().slice(..));
        render_pass.set_index_buffer(self.index_buffer.as_wgpu_buffer().slice(..), self.index_buffer.format().wgpu_format());
        render_pass.set_bind_group(0, self.texture_sampler.wgpu_bind_group(), &[]);
        render_pass.set_bind_group(1, self.texture.wgpu_bind_group(), &[]);
        render_pass.draw_indexed(0..6, 0, 0..1);
        drop(render_pass);

        let command_buffer = encoder.finish();
        self.queue.submit(std::iter::once(command_buffer));
        self.queue.present(output);

        IdleRenderer {
            device: self.device,
            queue: self.queue,
            vertex_buffer: self.vertex_buffer,
            index_buffer: self.index_buffer,
            screen_dimensions_bind_group: self.screen_dimensions_bind_group,
            pipeline: self.pipeline
        }
    }
}