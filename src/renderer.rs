mod colored_segment;
mod textured_segment;
mod colored_rect;
mod textured_rect;

use std::num::NonZeroU8;
use std::sync::Arc;
use wgpu::CurrentSurfaceTexture;
use crate::color::Color;
use crate::constants;
use crate::engine::{InitializationError};
use crate::math::point2f::Point2f;
use crate::math::positive_f32::PositiveF32;
use crate::math::rect2f::Rect2f;
use crate::math::segment2f::Segment2f;
use crate::math::unit_f32::UnitF32;
use crate::renderer::colored_rect::ColoredRectVertex;
use crate::renderer::colored_segment::ColoredSegmentVertex;
use crate::renderer::textured_rect::TexturedRectVertex;
use crate::renderer::textured_segment::TexturedSegmentVertex;

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
    config: wgpu::SurfaceConfiguration,
    colored_segment_vertex_buffer: wgpu::Buffer,
    colored_segment_index_buffer: wgpu::Buffer,
    textured_segment_vertex_buffer: wgpu::Buffer,
    textured_segment_index_buffer: wgpu::Buffer,
    colored_rect_vertex_buffer: wgpu::Buffer,
    colored_rect_index_buffer: wgpu::Buffer,
    textured_rect_vertex_buffer: wgpu::Buffer,
    textured_rect_index_buffer: wgpu::Buffer,
    screen_dimensions_buffer: wgpu::Buffer,
    screen_dimensions_bind_group: wgpu::BindGroup,
    textured_segment_bind_group: wgpu::BindGroup,
    textured_rect_bind_group: wgpu::BindGroup,
    colored_segment_pipeline: wgpu::RenderPipeline,
    textured_segment_pipeline: wgpu::RenderPipeline,
    colored_rect_pipeline: wgpu::RenderPipeline,
    textured_rect_pipeline: wgpu::RenderPipeline,
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

        let colored_segment_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (constants::SEGMENTS_MAX_BATCH_SIZE * ColoredSegmentVertex::byte_size() * ColoredSegmentVertex::VERTICES_PER_SEGMENT) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let colored_segment_index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (constants::SEGMENTS_MAX_BATCH_SIZE * ColoredSegmentVertex::INDICES_PER_SEGMENT * size_of::<u32>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut indices = Vec::new();
        for n in 0..constants::SEGMENTS_MAX_BATCH_SIZE {
            for idx in ColoredSegmentVertex::PRIMITIVE_INDICES {
                let base_idx = n * ColoredSegmentVertex::VERTICES_PER_SEGMENT;
                indices.push((base_idx + idx) as u32);
            }
        }

        queue.write_buffer(&colored_segment_index_buffer, 0, bytemuck::cast_slice(&indices));

        let colored_segment_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("../resources/shaders/colored_segment_shader.wgsl").into()),
        });

        let textured_segment_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (constants::SEGMENTS_MAX_BATCH_SIZE * TexturedSegmentVertex::byte_size() * TexturedSegmentVertex::VERTICES_PER_SEGMENT) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let colored_rect_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (constants::SEGMENTS_MAX_BATCH_SIZE * ColoredRectVertex::byte_size() * ColoredRectVertex::VERTICES_PER_RECT) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let colored_rect_index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
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

        queue.write_buffer(&colored_rect_index_buffer, 0, bytemuck::cast_slice(&indices));

        let textured_rect_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (constants::RECTS_MAX_BATCH_SIZE * TexturedRectVertex::byte_size() * TexturedRectVertex::VERTICES_PER_RECT) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let textured_segment_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("../resources/shaders/textured_segment_shader.wgsl").into()),
        });

        let colored_rect_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("../resources/shaders/colored_rect_shader.wgsl").into()),
        });

        let textured_rect_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("../resources/shaders/textured_rect_shader.wgsl").into()),
        });

        let screen_dimensions_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (2 * size_of::<u32>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        queue.write_buffer(&screen_dimensions_buffer, 0, bytemuck::cast_slice(&[window_size.width, window_size.height]));

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

        let texture_binding_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
                label: None,
            });

        let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let image_bytes = include_bytes!("../resources/tiles/dotted_line.png");
        let image = image::load_from_memory(image_bytes)
            .expect("The image is fine")
            .flipv();
        let image_rgba = image.as_rgba8().expect("The image contains rgba channels");

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

        let textured_segment_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_binding_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&texture_sampler)
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view)
                },
            ],
            label: None,
        });

        let image_bytes = include_bytes!("../resources/tiles/reshiram.png");
        let image = image::load_from_memory(image_bytes)
            .expect("The image is fine")
            .flipv();
        let image_rgba = image.as_rgba8().expect("The image contains rgba channels");

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

        let textured_rect_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_binding_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&texture_sampler)
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view)
                },
            ],
            label: None,
        });

        let colored_segment_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[Some(&screen_dimensions_binding_layout)],
                immediate_size: 0,
            });

        let colored_segment_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&colored_segment_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &colored_segment_shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(ColoredSegmentVertex::desc())],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &colored_segment_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
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

        let textured_segment_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[
                    Some(&screen_dimensions_binding_layout),
                    Some(&texture_binding_layout)
                ],
                immediate_size: 0,
            });

        let textured_segment_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&textured_segment_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &textured_segment_shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(TexturedSegmentVertex::desc())],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &textured_segment_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
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

        let colored_rect_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[],
                immediate_size: 0,
            });

        let colored_rect_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&colored_rect_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &colored_rect_shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(ColoredRectVertex::desc())],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &colored_rect_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
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

        let textured_rect_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[Some(&texture_binding_layout)],
                immediate_size: 0,
            });

        let textured_rect_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&textured_rect_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &textured_rect_shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(TexturedRectVertex::desc())],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &textured_rect_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
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

        Ok(Self {
            surface,
            device,
            queue,
            config,
            colored_segment_vertex_buffer,
            colored_segment_index_buffer: colored_segment_index_buffer.clone(),
            textured_segment_vertex_buffer,
            textured_segment_index_buffer: colored_segment_index_buffer,
            colored_rect_vertex_buffer,
            colored_rect_index_buffer: colored_rect_index_buffer.clone(),
            textured_rect_vertex_buffer,
            textured_rect_index_buffer: colored_rect_index_buffer,
            screen_dimensions_buffer,
            screen_dimensions_bind_group,
            textured_segment_bind_group,
            textured_rect_bind_group,
            colored_segment_pipeline,
            textured_segment_pipeline,
            colored_rect_pipeline,
            textured_rect_pipeline,
            clear_color: Color::SOLID_BLACK
        })
    }

    pub fn set_clear_color(&mut self, color: Color) {
        self.clear_color = color;
    }

    pub fn begin_scene(&mut self) -> Result<InProgressRenderer<'_>, BeginSceneError> {
        let surface_texture = match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(t) | CurrentSurfaceTexture::Suboptimal(t) => t,
            CurrentSurfaceTexture::Timeout => return Err(BeginSceneError::FrameTimeoutError),
            CurrentSurfaceTexture::Occluded => return Err(BeginSceneError::OccludedSurfaceError),
            CurrentSurfaceTexture::Validation => return Err(BeginSceneError::ValidationError),
            CurrentSurfaceTexture::Outdated => return Err(BeginSceneError::OutdatedConfigError),
            CurrentSurfaceTexture::Lost => return Err(BeginSceneError::LostSurfaceError)
        };

        Ok(InProgressRenderer {
            renderer: self,
            surface_texture
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }

        self.queue.write_buffer(
            &self.screen_dimensions_buffer, 0, bytemuck::cast_slice(&[width, height])
        );
    }
}

pub struct InProgressRenderer<'a> {
    renderer: &'a IdleRenderer,
    surface_texture: wgpu::SurfaceTexture
}

impl<'a> InProgressRenderer<'a> {
    pub fn add_line(&mut self, line: Line, fill: Fill, pixel_width: NonZeroU8) {

    }

    pub fn add_triangle(&mut self, triangle: Triangle, fill: Fill) {

    }

    pub fn add_square(&mut self, square: Square, fill: Fill) {

    }

    pub fn add_circle(&mut self, circle: Circle, fill: Fill) {

    }

    pub fn end_scene(self) {
        let view = self.surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.renderer.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("render pass"),
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.renderer.clear_color.into()),
                        store: wgpu::StoreOp::Store
                    },
                })
            ],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        let segment = Segment2f::new(
            Point2f::new(-0.5, -0.5), Point2f::new(0.0, 0.5), constants::MATH_EPSILON
        ).expect("Valid segment");

        let color = Color::new(
            UnitF32::new(0.3).expect("Valid color channel"),
            UnitF32::new(0.7).expect("Valid color channel"),
            UnitF32::new(0.1).expect("Valid color channel"),
            UnitF32::ONE
        );

        let pixel_width = PositiveF32::new(5.0).expect("Positive number");

        let colored_segment_vertices_1 = ColoredSegmentVertex::generate(segment, color, pixel_width);

        let segment = Segment2f::new(
            Point2f::new(0.0, 0.5), Point2f::new(0.5, -0.5), constants::MATH_EPSILON
        ).expect("Valid segment");

        let color = Color::new(
            UnitF32::new(0.7).expect("Valid color channel"),
            UnitF32::new(0.2).expect("Valid color channel"),
            UnitF32::new(0.3).expect("Valid color channel"),
            UnitF32::ONE
        );

        let pixel_width = PositiveF32::new(7.0).expect("Positive number");

        let colored_segment_vertices_2 = ColoredSegmentVertex::generate(segment, color, pixel_width);

        self.renderer.queue.write_buffer(
            &self.renderer.colored_segment_vertex_buffer,
            0,
            bytemuck::cast_slice(&[colored_segment_vertices_1, colored_segment_vertices_2].concat())
        );

        render_pass.set_pipeline(&self.renderer.colored_segment_pipeline);
        render_pass.set_vertex_buffer(0, self.renderer.colored_segment_vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.renderer.colored_segment_index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.set_bind_group(0, &self.renderer.screen_dimensions_bind_group, &[]);
        render_pass.draw_indexed(0..12, 0, 0..1);

        let segment = Segment2f::new(
            Point2f::new(-0.5, -0.5), Point2f::new(0.5, -0.5), constants::MATH_EPSILON
        ).expect("Valid segment");

        let pixel_width = PositiveF32::new(2.0).expect("Positive number");

        let textured_segment_vertices = TexturedSegmentVertex::generate(segment, pixel_width);

        self.renderer.queue.write_buffer(
            &self.renderer.textured_segment_vertex_buffer,
            0,
            bytemuck::cast_slice(&textured_segment_vertices)
        );

        render_pass.set_pipeline(&self.renderer.textured_segment_pipeline);
        render_pass.set_vertex_buffer(0, self.renderer.textured_segment_vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.renderer.textured_segment_index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.set_bind_group(0, &self.renderer.screen_dimensions_bind_group, &[]);
        render_pass.set_bind_group(1, &self.renderer.textured_segment_bind_group, &[]);
        render_pass.draw_indexed(0..6, 0, 0..1);

        let rect = Rect2f::new(
            Point2f::new(-0.5, 0.5),
            PositiveF32::new(0.25).expect("Positive number"),
            PositiveF32::new(0.25).expect("Positive number")
        );

        let color = Color::new(
            UnitF32::new(0.4).expect("Valid color channel"),
            UnitF32::new(0.1).expect("Valid color channel"),
            UnitF32::new(0.7).expect("Valid color channel"),
            UnitF32::ONE
        );

        let colored_rect_vertices = ColoredRectVertex::generate(rect, color);

        self.renderer.queue.write_buffer(
            &self.renderer.colored_rect_vertex_buffer,
            0,
            bytemuck::cast_slice(&colored_rect_vertices)
        );

        render_pass.set_pipeline(&self.renderer.colored_rect_pipeline);
        render_pass.set_vertex_buffer(0, self.renderer.colored_rect_vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.renderer.colored_rect_index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..6, 0, 0..1);

        let rect = Rect2f::new(
            Point2f::new(0.5, 0.5),
            PositiveF32::new(0.25).expect("Positive number"),
            PositiveF32::new(0.25).expect("Positive number")
        );

        let textured_rect_vertices = TexturedRectVertex::generate(rect);

        self.renderer.queue.write_buffer(
            &self.renderer.textured_rect_vertex_buffer,
            0,
            bytemuck::cast_slice(&textured_rect_vertices)
        );

        render_pass.set_pipeline(&self.renderer.textured_rect_pipeline);
        render_pass.set_vertex_buffer(0, self.renderer.textured_rect_vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.renderer.textured_rect_index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.set_bind_group(0, &self.renderer.textured_rect_bind_group, &[]);
        render_pass.draw_indexed(0..6, 0, 0..1);

        drop(render_pass);

        let command_buffer = encoder.finish();
        self.renderer.queue.submit(std::iter::once(command_buffer));
        self.renderer.queue.present(self.surface_texture);
    }
}