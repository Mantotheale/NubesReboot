use std::sync::Arc;
use wgpu::CurrentSurfaceTexture;

pub mod renderer;
pub mod texture;

#[derive(Debug)]
pub enum InitializationError {
    CreateSurfaceError(wgpu::CreateSurfaceError),
    AdapterError(wgpu::RequestAdapterError),
    RequestDeviceError(wgpu::RequestDeviceError),
    NoSRGBSurface,
}

impl std::fmt::Display for InitializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InitializationError::CreateSurfaceError(err) =>
                write!(f, "Error creating a surface: {err}"),
            InitializationError::AdapterError(err) =>
                write!(f, "Error creating an adapter: {err}"),
            InitializationError::RequestDeviceError(err) =>
                write!(f, "Error requesting the device: {err}"),
            InitializationError::NoSRGBSurface =>
                write!(f, "No SRGB surface found")
        }
    }
}

impl std::error::Error for InitializationError { }

#[derive(Debug)]
pub enum SkipRender {
    ValidationError,
    FrameTimeout,
    OccludedSurface,
    OutdatedConfig,
}

#[derive(Debug)]
pub struct LostSurfaceError;

impl std::fmt::Display for LostSurfaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "The render surface was lost")
    }
}

impl std::error::Error for LostSurfaceError { }

#[derive(Debug)]
pub enum FetchSurfaceResult {
    Success(wgpu::SurfaceTexture),
    SkipRender(SkipRender),
    LostSurface(LostSurfaceError)
}

pub struct GpuContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration
}

impl GpuContext {
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

        Ok(Self {
            device,
            queue,
            surface,
            config
        })
    }

    pub fn fetch_render_surface(&self) -> FetchSurfaceResult {
        match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(t) | CurrentSurfaceTexture::Suboptimal(t)
                => FetchSurfaceResult::Success(t),
            CurrentSurfaceTexture::Timeout => FetchSurfaceResult::SkipRender(SkipRender::FrameTimeout),
            CurrentSurfaceTexture::Occluded => FetchSurfaceResult::SkipRender(SkipRender::OccludedSurface),
            CurrentSurfaceTexture::Validation => FetchSurfaceResult::SkipRender(SkipRender::ValidationError),
            CurrentSurfaceTexture::Outdated => FetchSurfaceResult::SkipRender(SkipRender::OutdatedConfig),
            CurrentSurfaceTexture::Lost => FetchSurfaceResult::LostSurface(LostSurfaceError)
        }
    }

    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    pub fn device(&self) -> wgpu::Device {
        self.device.clone()
    }

    pub fn queue(&self) -> wgpu::Queue {
        self.queue.clone()
    }
}