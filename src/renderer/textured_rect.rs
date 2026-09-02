use crate::constants::MATH_EPSILON;
use crate::math::positive_f32::PositiveF32;
use crate::math::rect2f::Rect2f;
use crate::math::segment2f::Segment2f;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TexturedRectVertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

impl TexturedRectVertex {
    pub const VERTICES_PER_RECT: usize = 4;

    pub const INDICES_PER_RECT: usize = 6;

    pub const PRIMITIVE_INDICES: [usize; Self::INDICES_PER_RECT] =
        [0, 1, 3, 1, 2, 3];

    pub fn byte_size() -> usize {
        size_of::<Self>()
    }

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
                    format: wgpu::VertexFormat::Float32x2,
                }
            ],
        }
    }

    pub fn generate(rect: Rect2f) -> [Self; Self::VERTICES_PER_RECT] {
        [
            Self { position: rect.bottom_left().into(), tex_coords: [0.0, 0.0] },
            Self { position: rect.bottom_right().into(), tex_coords: [1.0, 0.0] },
            Self { position: rect.top_right().into(), tex_coords: [1.0, 1.0] },
            Self { position: rect.top_left().into(), tex_coords: [0.0, 1.0] }
        ]
    }
}