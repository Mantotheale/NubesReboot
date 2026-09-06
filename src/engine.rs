use std::sync::Arc;
use std::time::{Duration, Instant};
use crate::color::Color;
use crate::math::unit_f32::UnitF32;
use crate::renderer::IdleRenderer;

#[derive(Debug)]
pub struct LostSurfaceError { }

#[derive(Debug)]
pub enum InitializationError {
    CreateSurfaceError(wgpu::CreateSurfaceError),
    AdapterError(wgpu::RequestAdapterError),
    RequestDeviceError(wgpu::RequestDeviceError),
    NoSRGBSurface,
}

pub struct Engine {
    proxy: winit::event_loop::EventLoopProxy<()>,
    window: Arc<winit::window::Window>,
    next_update: Instant,
    next_one_sec_update: Instant,
    update_duration: Duration,
    one_sec_duration: Duration,
    renderer: IdleRenderer
}

impl Engine {
    pub async fn new(proxy: winit::event_loop::EventLoopProxy<()>, event_loop: &winit::event_loop::ActiveEventLoop) -> Result<Self, InitializationError> {
        let window_attributes = winit::window::Window::default_attributes();
        let window = event_loop.create_window(window_attributes)
            .expect("Couldn't create a window");

        let window = Arc::new(window);

        let mut renderer = IdleRenderer::new(window.clone()).await.expect("Should not panic");
        renderer.set_clear_color(Color::new(
            UnitF32::new(0.2).expect("Valid color channel"),
            UnitF32::new(0.2).expect("Valid color channel"),
            UnitF32::new(0.2).expect("Valid color channel"),
            UnitF32::ONE
        ));

        let now = Instant::now();
        let update_duration = Duration::from_nanos(1_000_000_000 / 60);
        let one_sec_duration = Duration::from_secs(1);

        Ok(Self {
            proxy,
            window,
            next_update: now + update_duration,
            next_one_sec_update: now + one_sec_duration,
            update_duration,
            one_sec_duration,
            renderer
        })
    }

    pub fn window_event(&mut self, event: winit::event::WindowEvent) {
        match event {
            winit::event::WindowEvent::CloseRequested => _ = self.proxy.send_event(()),
            winit::event::WindowEvent::RedrawRequested => _ = self.render(),
            winit::event::WindowEvent::Resized(size) => self.renderer.resize(size.width, size.height),
            winit::event::WindowEvent::KeyboardInput {
                event: winit::event::KeyEvent {
                    physical_key: winit::keyboard::PhysicalKey::Code(code),
                    state, ..
                }, ..
            } => match (code, state.is_pressed()) {
                (winit::keyboard::KeyCode::Escape, true) => _ = self.proxy.send_event(()),
                _ => {}
            },
            _ => {}
        }
    }

    fn process_inputs(&mut self) {

    }

    fn update(&mut self) {

    }

    fn one_sec_update(&mut self) {

    }

    fn render(&mut self) -> Result<(), LostSurfaceError> {
        self.window.request_redraw();

        let scene_renderer = self.renderer.begin_scene().expect("Should not panic");
        scene_renderer.end_scene();

        Ok(())
    }

    pub fn hook(&mut self) -> Result<(), LostSurfaceError>{
        self.process_inputs();

        let now = Instant::now();

        while now >= self.next_update {
            self.update();
            self.next_update += self.update_duration;
        }

        self.render()?;

        while now >= self.next_one_sec_update {
            self.one_sec_update();
            self.next_one_sec_update += self.one_sec_duration;
        }

        Ok(())
    }
}