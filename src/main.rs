use crate::entry_point::EntryPoint;
use winit::event_loop::EventLoop;

mod engine;
mod entry_point;
mod renderer;
mod math;
mod color;
mod constants;
mod util;

pub fn main() {
    let event_loop = EventLoop::with_user_event().build().expect("Couldn't build event loop");
    let proxy = event_loop.create_proxy();
    let mut entry_point = EntryPoint::new(proxy);
    event_loop.run_app(&mut entry_point).expect("Error running the app");
}