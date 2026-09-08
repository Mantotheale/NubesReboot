mod colored_segment;
mod textured_segment;
mod rect_batch;
mod rect;
mod texture;

use std::cmp::Ordering;
use std::path::{Path,};
use std::sync::Arc;
use wgpu::CurrentSurfaceTexture;
use crate::color::Color;
use crate::constants;
use crate::engine::InitializationError;
use crate::math::point2f::Point2f;
use crate::math::positive_f32::PositiveF32;
use crate::math::rect2f::Rect2f;
use crate::math::segment2f::Segment2f;
use crate::math::unit_f32::UnitF32;
use crate::renderer::colored_segment::ColoredSegmentVertex;
use crate::renderer::rect_batch::RectBatch;
use crate::renderer::texture::texture_atlas::TextureAtlas;
use crate::renderer::texture::texture_handle::TextureHandle;
use crate::renderer::textured_segment::TexturedSegmentVertex;
use crate::util::image_utils;

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

enum Shape {
    Rect(Rect2f),
    Segment { segment: Segment2f, pixel_width: PositiveF32 }
}

impl Shape {
    fn draw_order(a: &Self, b: &Self) -> Ordering {
        match a {
            Shape::Rect(_) => match b {
                Shape::Rect(_) => Ordering::Equal,
                Shape::Segment { .. } => Ordering::Less
            }
            Shape::Segment { .. } => match b {
                Shape::Rect(_) => Ordering::Greater,
                Shape::Segment { .. } => Ordering::Equal
            }
        }
    }
}

#[derive(Clone)]
pub enum Fill {
    Color(Color),
    TextureHandle(TextureHandle)
}

impl Fill {
    fn draw_order(a: &Self, b: &Self) -> Ordering {
        match a {
            Fill::Color(_) => match b {
                Fill::Color(_) => Ordering::Equal,
                Fill::TextureHandle(_) => Ordering::Less
            }
            Fill::TextureHandle(tex_a) => match b {
                Fill::Color(_) => Ordering::Greater,
                Fill::TextureHandle(tex_b) => Ord::cmp(&tex_a.id(), &tex_b.id())
            }
        }
    }
}

struct RenderCommand {
    shape: Shape,
    fill: Fill,
    z_index: i32
}

impl RenderCommand {
    fn draw_order(a: &Self, b: &Self) -> Ordering {
        Ord::cmp(&a.z_index, &b.z_index)
            .then_with(|| Shape::draw_order(&a.shape, &b.shape))
            .then_with(|| Fill::draw_order(&a.fill, &b.fill))
    }
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
    screen_dimensions_buffer: wgpu::Buffer,
    screen_dimensions_bind_group: wgpu::BindGroup,
    textured_segment_bind_group: wgpu::BindGroup,
    colored_segment_pipeline: wgpu::RenderPipeline,
    textured_segment_pipeline: wgpu::RenderPipeline,
    clear_color: Color,
    rect_batch: RectBatch,
    rect_1: Rect2f,
    rect_2: Rect2f,
    rect_3: Rect2f,
    rect_4: Rect2f,
    rect_5: Rect2f,
    rect_6: Rect2f,
    rect_7: Rect2f,
    rect_8: Rect2f,
    rect_9: Rect2f,
    white: Color,
    yellow: Color,
    color_1: Color,
    reshiram_texture: TextureHandle,
    rock_texture: TextureHandle,
    mewtwo_texture: TextureHandle
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

        let textured_segment_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("../resources/shaders/textured_segment_shader.wgsl").into()),
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

        let rect_batch = RectBatch::new(device.clone(), queue.clone(), config.format);

        let rect_1 = Rect2f::new(
            Point2f::new(-0.75, -0.75),
            PositiveF32::new(0.2).expect("Positive number"),
            PositiveF32::new(0.2).expect("Positive number")
        );

        let rect_2 = Rect2f::new(
            Point2f::new(-0.5, -0.75),
            PositiveF32::new(0.2).expect("Positive number"),
            PositiveF32::new(0.2).expect("Positive number")
        );

        let rect_3 = Rect2f::new(
            Point2f::new(-0.25, -0.75),
            PositiveF32::new(0.2).expect("Positive number"),
            PositiveF32::new(0.2).expect("Positive number")
        );

        let rect_4 = Rect2f::new(
            Point2f::new(0.0, -0.75),
            PositiveF32::new(0.2).expect("Positive number"),
            PositiveF32::new(0.2).expect("Positive number")
        );

        let rect_5 = Rect2f::new(
            Point2f::new(0.25, -0.75),
            PositiveF32::new(0.2).expect("Positive number"),
            PositiveF32::new(0.2).expect("Positive number")
        );

        let rect_6 = Rect2f::new(
            Point2f::new(0.5, -0.75),
            PositiveF32::new(0.2).expect("Positive number"),
            PositiveF32::new(0.2).expect("Positive number")
        );

        let rect_7 = Rect2f::new(
            Point2f::new(0.75, -0.75),
            PositiveF32::new(0.2).expect("Positive number"),
            PositiveF32::new(0.2).expect("Positive number")
        );

        let rect_8 = Rect2f::new(
            Point2f::new(-0.5, 0.5),
            PositiveF32::new(0.25).expect("Positive number"),
            PositiveF32::new(0.25).expect("Positive number")
        );

        let rect_9 = Rect2f::new(
            Point2f::new(0.5, 0.5),
            PositiveF32::new(0.25).expect("Positive number"),
            PositiveF32::new(0.25).expect("Positive number")
        );

        let white = Color::new(
            UnitF32::new(1.0).expect("Valid color channel"),
            UnitF32::new(1.0).expect("Valid color channel"),
            UnitF32::new(1.0).expect("Valid color channel"),
            UnitF32::ONE
        );

        let yellow = Color::new(
            UnitF32::new(0.5).expect("Valid color channel"),
            UnitF32::new(1.0).expect("Valid color channel"),
            UnitF32::new(0.0).expect("Valid color channel"),
            UnitF32::ONE
        );

        let color_1 = Color::new(
            UnitF32::new(0.4).expect("Valid color channel"),
            UnitF32::new(0.1).expect("Valid color channel"),
            UnitF32::new(0.7).expect("Valid color channel"),
            UnitF32::ONE
        );

        let reshiram_path = Path::new("tiles/reshiram.png");
        let mewtwo_path = Path::new("tiles/mewtwo.png");
        let rock_path = Path::new("tiles/rock.png");

        let reshiram_image = image_utils::read_image(reshiram_path).unwrap();
        let mewtwo_image = image_utils::read_image(mewtwo_path).unwrap();
        let rock_image = image_utils::read_image(rock_path).unwrap();
        let tiles = [
            (reshiram_path, reshiram_image),
            (mewtwo_path, mewtwo_image),
            (rock_path, rock_image),
        ];
        
        let atlas = TextureAtlas::new(&device, &queue, &tiles).unwrap();
        let reshiram_texture = atlas.get_tile(reshiram_path).unwrap().clone();
        let mewtwo_texture = atlas.get_tile(mewtwo_path).unwrap().clone();
        let rock_texture = atlas.get_tile(rock_path).unwrap().clone();

        Ok(Self {
            surface,
            device,
            queue,
            config,
            colored_segment_vertex_buffer,
            colored_segment_index_buffer: colored_segment_index_buffer.clone(),
            textured_segment_vertex_buffer,
            textured_segment_index_buffer: colored_segment_index_buffer,
            screen_dimensions_buffer,
            screen_dimensions_bind_group,
            textured_segment_bind_group,
            colored_segment_pipeline,
            textured_segment_pipeline,
            clear_color: Color::SOLID_BLACK,
            rect_batch,
            reshiram_texture,
            mewtwo_texture,
            rock_texture,
            rect_1,
            rect_2,
            rect_3,
            rect_4,
            rect_5,
            rect_6,
            rect_7,
            rect_8,
            rect_9,
            white,
            yellow,
            color_1
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
            surface_texture,
            render_commands: Vec::new()
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
    renderer: &'a mut IdleRenderer,
    surface_texture: wgpu::SurfaceTexture,
    render_commands: Vec<RenderCommand>
}

impl<'a> InProgressRenderer<'a> {
    pub fn add_segment(&mut self, segment: Segment2f, fill: Fill, pixel_width: PositiveF32, z_index: i32) {
        self.render_commands.push(RenderCommand {
            shape: Shape::Segment { segment, pixel_width },
            fill,
            z_index,
        })
    }

    pub fn add_rect(&mut self, rect: Rect2f, fill: Fill, z_index: i32) {
        self.render_commands.push(RenderCommand {
            shape: Shape::Rect(rect),
            fill,
            z_index,
        })
    }

    pub fn end_scene(mut self) {
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

        self.add_rect(self.renderer.rect_1, Fill::Color(self.renderer.yellow), 1);
        self.add_rect(self.renderer.rect_2, Fill::Color(self.renderer.white), 1);
        self.add_rect(self.renderer.rect_3, Fill::Color(self.renderer.yellow), 1);
        self.add_rect(self.renderer.rect_4, Fill::Color(self.renderer.white), 1);
        self.add_rect(self.renderer.rect_5, Fill::TextureHandle(self.renderer.reshiram_texture.clone()), 1);
        self.add_rect(self.renderer.rect_6, Fill::TextureHandle(self.renderer.mewtwo_texture.clone()), 1);
        self.add_rect(self.renderer.rect_7, Fill::TextureHandle(self.renderer.rock_texture.clone()), 1);
        self.add_rect(self.renderer.rect_8, Fill::Color(self.renderer.color_1), 1);
        self.add_rect(self.renderer.rect_9, Fill::TextureHandle(self.renderer.reshiram_texture.clone()), 1);

        self.render_commands.sort_by(|a, b| RenderCommand::draw_order(a, b));
        for command in self.render_commands {
            match command.shape {
                Shape::Rect(rect) => {
                    let res = self.renderer.rect_batch.push(rect, command.fill.clone());
                    if res.is_err() {
                        self.renderer.rect_batch.draw(&mut render_pass);
                        self.renderer.rect_batch.clear();
                        self.renderer.rect_batch.push(rect, command.fill).expect("The batch is now empty");
                    }
                }
                Shape::Segment { .. } => unimplemented!()
            }
        }
        self.renderer.rect_batch.draw(&mut render_pass);
        self.renderer.rect_batch.clear();

        drop(render_pass);

        let command_buffer = encoder.finish();
        self.renderer.queue.submit(std::iter::once(command_buffer));
        self.renderer.queue.present(self.surface_texture);
    }
}