use std::cmp::Ordering;
use crate::color::Color;
use crate::math::positive_f32::PositiveF32;
use crate::math::rect2f::Rect2f;
use crate::math::segment2f::Segment2f;
use crate::renderer::texture::texture_handle::TextureHandle;

#[derive(Clone)]
pub enum Fill {
    Color(Color),
    TextureHandle(TextureHandle)
}

impl Fill {
    fn draw_order(a: &Self, b: &Self) -> Ordering {
        match a {
            Fill::Color(_) => match b {
                Fill::Color(_) => Ordering::Equal,
                Fill::TextureHandle(_) => Ordering::Less
            }
            Fill::TextureHandle(tex_a) => match b {
                Fill::Color(_) => Ordering::Greater,
                Fill::TextureHandle(tex_b) => Ord::cmp(&tex_a.id(), &tex_b.id())
            }
        }
    }
}

pub enum Shape {
    Rect { rect: Rect2f, fill: Fill },
    Segment { segment: Segment2f, pixel_width: PositiveF32, color: Color }
}

impl Shape {
    fn draw_order(a: &Self, b: &Self) -> Ordering {
        match a {
            Shape::Rect { fill: fill_a, .. } => match b {
                Shape::Rect { fill: fill_b, .. } => Fill::draw_order(fill_a, fill_b),
                Shape::Segment { .. } => Ordering::Greater
            }
            Shape::Segment { .. } => match b {
                Shape::Rect { .. } => Ordering::Less,
                Shape::Segment { .. } => Ordering::Equal
            }
        }
    }
}

pub struct RenderPrimitive {
    pub shape: Shape,
    pub z_index: i32
}

impl RenderPrimitive {
    pub fn draw_order(a: &Self, b: &Self) -> Ordering {
        Ord::cmp(&a.z_index, &b.z_index)
            .then_with(|| Shape::draw_order(&a.shape, &b.shape))
    }
}

pub trait Renderable {
    fn render_primitive(&self) -> RenderPrimitive;
}