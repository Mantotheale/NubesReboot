use winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use crate::engine::Engine;

pub struct EntryPoint {
    proxy: EventLoopProxy<()>,
    engine: Option<Engine>
}

impl EntryPoint {
    pub fn new(proxy: EventLoopProxy<()>) -> Self {
        Self { proxy, engine: None }
    }
}

impl winit::application::ApplicationHandler for EntryPoint {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let engine = pollster::block_on(Engine::new(self.proxy.clone(), event_loop))
            .expect("Couldn't initialize the engine");
        self.engine = Some(engine);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, _: ()) {
        event_loop.exit();
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _: winit::window::WindowId,
        event: winit::event::WindowEvent
    ) {
        match &mut self.engine {
            None => if let winit::event::WindowEvent::CloseRequested = event { event_loop.exit() }
            Some(engine) => engine.window_event(event)
        }
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let Some(engine) = &mut self.engine else { return; };
        if let Err(_) = engine.hook() {
            event_loop.exit();
        }
    }
}