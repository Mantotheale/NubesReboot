mod rect;
mod texture;
mod segment;

use crate::{
    renderer::{
        texture::{
            texture_handle::TextureHandle,
            texture_atlas::TextureAtlas
        },
    },
    math::{
        unit_f32::UnitF32,
        segment2f::Segment2f,
        rect2f::Rect2f,
        positive_f32::PositiveF32,
        point2f::Point2f
    },
    engine::InitializationError,
    constants,
    color::Color,
    util::image_utils
};
use rect::rect_batch::RectBatch;
use std::cmp::Ordering;
use std::path::Path;
use std::sync::Arc;
use wgpu::CurrentSurfaceTexture;
use crate::renderer::segment::segment_batch::SegmentBatch;

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

enum Primitive {
    Rect { rect: Rect2f, fill: Fill },
    Segment { segment: Segment2f, pixel_width: PositiveF32, color: Color }
}

impl Primitive {
    fn draw_order(a: &Self, b: &Self) -> Ordering {
        match a {
            Primitive::Rect { fill: fill_a, .. } => match b {
                Primitive::Rect { fill: fill_b, .. } => Fill::draw_order(fill_a, fill_b),
                Primitive::Segment { .. } => Ordering::Greater
            }
            Primitive::Segment { .. } => match b {
                Primitive::Rect { .. } => Ordering::Less,
                Primitive::Segment { .. } => Ordering::Equal
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
    primitive: Primitive,
    z_index: i32
}

impl RenderCommand {
    fn draw_order(a: &Self, b: &Self) -> Ordering {
        Ord::cmp(&a.z_index, &b.z_index)
            .then_with(|| Primitive::draw_order(&a.primitive, &b.primitive))
    }
}

pub struct IdleRenderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
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
    mewtwo_texture: TextureHandle,
    segment_batch: SegmentBatch,
    segment_1: Segment2f,
    segment_2: Segment2f,
    segment_3: Segment2f,
}

impl IdleRenderer {
    pub async fn new(window: Arc<winit::window::Window>) -> Result<Self, InitializationError> {
        let instance = wgpu::Instance::new(
            wgpu::InstanceDescriptor::new_with_display_handle(
                Box::new(window.clone())
            )
        );

        let surface = instance.create_surface(window.clone())
            .map_err(|err| InitializationError::CreateSurfaceError(err))?;

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
            apply_limit_buckets: true,
        }).await.map_err(|err| InitializationError::AdapterError(err))?;

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::empty(),
            required_limits: Default::default(),
            experimental_features: Default::default(),
            memory_hints: Default::default(),
            trace: Default::default(),
        }).await.map_err(|err| InitializationError::RequestDeviceError(err))?;

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = *surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .ok_or(InitializationError::NoSRGBSurface)?;

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

        let segment_batch = SegmentBatch::new(device.clone(), queue.clone(), config.format);

        let segment_1 = Segment2f::new(
            Point2f::new(-0.6, -0.6),
            Point2f::new(-0.6, 0.6),
            constants::MATH_EPSILON
        ).unwrap();

        let segment_2 = Segment2f::new(
            Point2f::new(0.6, -0.6),
            Point2f::new(0.6, 0.6),
            constants::MATH_EPSILON
        ).unwrap();

        let segment_3 = Segment2f::new(
            Point2f::new(-0.2, 0.2),
            Point2f::new(0.2, -0.2),
            constants::MATH_EPSILON
        ).unwrap();

        Ok(Self {
            surface,
            device,
            queue,
            config,
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
            color_1,
            segment_batch,
            segment_1,
            segment_2,
            segment_3
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
    }
}

pub struct InProgressRenderer<'a> {
    renderer: &'a mut IdleRenderer,
    surface_texture: wgpu::SurfaceTexture,
    render_commands: Vec<RenderCommand>
}

impl<'a> InProgressRenderer<'a> {
    pub fn add_segment(&mut self, segment: Segment2f, color: Color, pixel_width: PositiveF32, z_index: i32) {
        self.render_commands.push(RenderCommand {
            primitive: Primitive::Segment { segment, pixel_width, color },
            z_index,
        });
    }

    pub fn add_rect(&mut self, rect: Rect2f, fill: Fill, z_index: i32) {
        self.render_commands.push(RenderCommand {
            primitive: Primitive::Rect { rect, fill },
            z_index,
        });
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

        let (width, height) = (self.renderer.config.width, self.renderer.config.height);

        self.add_rect(self.renderer.rect_1, Fill::Color(self.renderer.yellow), 1);
        self.add_rect(self.renderer.rect_2, Fill::Color(self.renderer.white), 1);
        self.add_rect(self.renderer.rect_3, Fill::Color(self.renderer.yellow), 1);
        self.add_rect(self.renderer.rect_4, Fill::Color(self.renderer.white), 1);
        self.add_rect(self.renderer.rect_5, Fill::TextureHandle(self.renderer.reshiram_texture.clone()), 1);
        self.add_rect(self.renderer.rect_6, Fill::TextureHandle(self.renderer.mewtwo_texture.clone()), 1);
        self.add_rect(self.renderer.rect_7, Fill::TextureHandle(self.renderer.rock_texture.clone()), 1);
        self.add_rect(self.renderer.rect_8, Fill::Color(self.renderer.color_1), 1);
        self.add_rect(self.renderer.rect_9, Fill::TextureHandle(self.renderer.reshiram_texture.clone()), 1);

        self.add_segment(self.renderer.segment_1, self.renderer.color_1, PositiveF32::new(2.0).unwrap(), 1);
        self.add_segment(self.renderer.segment_2, self.renderer.yellow, PositiveF32::new(3.0).unwrap(), 1);
        self.add_segment(self.renderer.segment_3, self.renderer.white, PositiveF32::new(4.0).unwrap(), 1);

        self.render_commands.sort_by(|a, b| RenderCommand::draw_order(a, b));
        for command in self.render_commands {
            match command.primitive {
                Primitive::Rect { rect, fill, .. } => {
                    let res = self.renderer.rect_batch.push(rect, fill.clone());
                    if res.is_err() {
                        self.renderer.rect_batch.draw(&mut render_pass);
                        self.renderer.rect_batch.clear();
                        self.renderer.rect_batch.push(rect, fill).expect("The batch is now empty");
                    }
                }
                Primitive::Segment { segment, color, pixel_width } => {
                    let res = self.renderer.segment_batch.push(segment, color, pixel_width);
                    if res.is_err() {
                        self.renderer.segment_batch.draw(&mut render_pass, (width, height));
                        self.renderer.segment_batch.clear();
                        self.renderer.segment_batch.push(segment, color, pixel_width).expect("The batch is now empty");
                    }
                }
            }
        }
        self.renderer.rect_batch.draw(&mut render_pass);
        self.renderer.rect_batch.clear();
        self.renderer.segment_batch.draw(&mut render_pass, (width, height));
        self.renderer.segment_batch.clear();

        drop(render_pass);

        let command_buffer = encoder.finish();
        self.renderer.queue.submit(std::iter::once(command_buffer));
        self.renderer.queue.present(self.surface_texture);
    }
}