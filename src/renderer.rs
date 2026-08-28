mod segment;

use std::num::NonZeroU8;
use std::sync::Arc;
use crate::color::Color;
use crate::engine::InitializationError;

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


        Ok(Self { })
    }

    fn begin_scene(self) -> InProgressRenderer {
        InProgressRenderer { }
    }

    fn resize(&mut self) {

    }

    fn swap_buffers(&self) {

    }
}

struct InProgressRenderer {

}

impl InProgressRenderer {
    pub fn clear_color(&mut self, color: Color) {

    }

    fn add_line(&mut self, line: Line, fill: Fill, pixel_width: NonZeroU8) {

    }

    fn add_triangle(&mut self, triangle: Triangle, fill: Fill) {

    }

    fn add_square(&mut self, square: Square, fill: Fill) {

    }

    fn add_circle(&mut self, circle: Circle, fill: Fill) {

    }

    fn end_scene(self) -> IdleRenderer {
        IdleRenderer { }
    }
}