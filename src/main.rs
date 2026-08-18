use std::sync::Arc;
use wgpu::{CurrentSurfaceTexture, Features, TextureViewDescriptor};
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};
use winit::window::WindowId;

#[derive(Debug)]
pub enum InitializationError {
    CreateSurfaceError(wgpu::CreateSurfaceError),
    AdapterError(wgpu::RequestAdapterError),
    RequestDeviceError(wgpu::RequestDeviceError),
    NoSRGBSurface
}

#[derive(Debug)]
pub struct LostSurfaceError { }

pub struct State {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration
}

impl State {
    pub async fn new(window: Window) -> Result<Self, InitializationError> {
        let window = Arc::new(window);

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
            required_features: Features::empty(),
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
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        Ok(Self { window, surface, device, queue, config })
    }

    pub fn resize(&mut self) {
        let size = self.window.inner_size();
        if size.width > 0 && size.height > 0 {
            self.config.width = size.width;
            self.config.height = size.height;
            self.surface.configure(&self.device, &self.config);
            println!("Size: {:?}", size)
        }
    }

    pub fn render(&mut self) -> Result<(), LostSurfaceError> {
        self.window.request_redraw();

        let output = match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(t) | CurrentSurfaceTexture::Suboptimal(t) => t,
            CurrentSurfaceTexture::Timeout |
            CurrentSurfaceTexture::Occluded |
            CurrentSurfaceTexture::Validation => return Ok(()),
            CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return Ok(());
            }
            CurrentSurfaceTexture::Lost => return Err(LostSurfaceError { })
        };

        let view = output.texture.create_view(&TextureViewDescriptor::default());

        Ok(())
    }
}

pub struct App {
    state: Option<State>,
}

impl App {
    pub fn new() -> Self {
        Self { state: None }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes();
        let window = event_loop.create_window(window_attributes)
            .expect("Couldn't create a window");

        self.state = Some(pollster::block_on(State::new(window)).expect("Couldn't initialize the app state"));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        let Some(_) = &self.state else { return; };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(_) => if let Some(s) = &mut self.state { s.resize() },
            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    physical_key: PhysicalKey::Code(code),
                    state, ..
                }, ..
            } => match (code, state.is_pressed()) {
                (KeyCode::Escape, true) => event_loop.exit(),
                _ => {}
            },
            _ => {}
        }
    }
}

pub fn main() {
    let event_loop = EventLoop::new().expect("Couldn't build event loop");
    let mut app = App::new();
    event_loop.run_app(&mut app).expect("Error running the app");
}