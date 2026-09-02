use crate::math::point2f::Point2f;
use crate::math::positive_f32::PositiveF32;

#[derive(Copy, Clone)]
pub struct Rect2f {
    center: Point2f,
    width: f32,
    height: f32
}

impl Rect2f {
    pub fn new(center: Point2f, width: PositiveF32, height: PositiveF32) -> Self {
        Self { center, width: width.value(), height: height.value() }
    }

    pub fn center(&self) -> Point2f {
        self.center
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }

    pub fn left(&self) -> f32 {
        self.center.x() - (0.5 * self.width)
    }

    pub fn right(&self) -> f32 {
        self.center.x() + (0.5 * self.width)
    }

    pub fn top(&self) -> f32 {
        self.center.y() + (0.5 * self.height)
    }

    pub fn bottom(&self) -> f32 {
        self.center.y() - (0.5 * self.height)
    }

    pub fn bottom_left(&self) -> Point2f {
        Point2f::new(self.left(), self.bottom())
    }

    pub fn bottom_right(&self) -> Point2f {
        Point2f::new(self.right(), self.bottom())
    }

    pub fn top_right(&self) -> Point2f {
        Point2f::new(self.right(), self.top())
    }

    pub fn top_left(&self) -> Point2f {
        Point2f::new(self.left(), self.top())
    }
}