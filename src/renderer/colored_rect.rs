use crate::color::Color;
use crate::math::rect2f::Rect2f;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColoredRectVertex {
    position: [f32; 2],
    color: [f32; 4],
}

impl ColoredRectVertex {
    pub const VERTICES_PER_RECT: usize = 4;

    pub const INDICES_PER_RECT: usize = 6;

    pub const BYTE_SIZE: usize = size_of::<Self>();

    pub const RECT_BYTE_SIZE: usize = Self::VERTICES_PER_RECT * Self::BYTE_SIZE;

    pub const PRIMITIVE_INDICES: [usize; Self::INDICES_PER_RECT] =
        [0, 1, 3, 1, 2, 3];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }

    pub fn generate(rect: Rect2f, color: Color) -> [Self; Self::VERTICES_PER_RECT] {
        [
            Self { position: rect.bottom_left().into(), color: color.into() },
            Self { position: rect.bottom_right().into(), color: color.into() },
            Self { position: rect.top_right().into(), color: color.into() },
            Self { position: rect.top_left().into(), color: color.into() }
        ]
    }
}