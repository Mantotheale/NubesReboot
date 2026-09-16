use crate::app::App;
use crate::color::Color;
use crate::graphics::renderer::{BeginSceneResult, IdleRenderer};
use crate::graphics::{GpuContext, InitializationError, LostSurfaceError, SkipRender};
use crate::math::unit_f32::UnitF32;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct Engine<T: App> {
    proxy: winit::event_loop::EventLoopProxy<()>,
    window: Arc<winit::window::Window>,
    next_update: Instant,
    next_one_sec_update: Instant,
    update_duration: Duration,
    one_sec_duration: Duration,
    renderer: IdleRenderer,
    app: T
}

impl<T: App> Engine<T> {
    pub async fn new(proxy: winit::event_loop::EventLoopProxy<()>, event_loop: &winit::event_loop::ActiveEventLoop) -> Result<Self, InitializationError> {
        let window_attributes = winit::window::Window::default_attributes();
        let window = event_loop.create_window(window_attributes)
            .expect("Couldn't create a window");
        let window = Arc::new(window);
        
        let gpu_context = GpuContext::new(window.clone()).await?;

        let mut renderer = IdleRenderer::new(gpu_context).await;
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
            app: T::init(),
            renderer,
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

        match self.renderer.begin_scene() {
            BeginSceneResult::Success(scene_renderer) => scene_renderer.end_scene(),
            BeginSceneResult::SkipRender(skip_render) => {
                if let SkipRender::OutdatedConfig = skip_render {
                    let screen_dimensions = self.window.inner_size();
                    self.renderer.resize(screen_dimensions.width, screen_dimensions.height);
                }
            }
            BeginSceneResult::LostSurface(err) => return Err(err)
        }

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