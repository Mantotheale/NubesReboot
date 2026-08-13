use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};
use winit::window::WindowId;

#[derive(Debug)]
pub struct InitializationError { }

pub struct State {
    window: Window
}

impl State {
    pub async fn new(window: Window) -> Result<Self, InitializationError> {
        Ok(Self { window })
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
    event_loop.run_app(&mut App::new()).expect("Error running the app");
}