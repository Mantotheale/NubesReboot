use std::num::NonZeroU8;

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

struct Color {

}

enum Fill {
    Color(Color),
    TextureView(TextureView)
}

struct IdleRenderer {

}

impl IdleRenderer {
    fn begin_scene(self) -> InProgressRenderer {
        InProgressRenderer { }
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