pub trait App {
    fn init() -> Self;
    fn process_inputs(&mut self);
    fn update(&mut self);
    fn one_sec_update(&mut self);
}

pub struct Game {

}

impl App for Game {
    fn init() -> Self {
        Self { }
    }

    fn process_inputs(&mut self) {
    }

    fn update(&mut self) {
    }

    fn one_sec_update(&mut self) {
    }
}