use crate::color::Color;
use crate::math::rect2f::Rect2f;
use crate::renderer::tex_coords::RectTexCoords;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RectVertex {
    position: [f32; 2],
    color: [f32; 4],
    tex_coords: [f32; 2],
    tex_index: i32
}

impl RectVertex {
    pub const VERTICES_PER_RECT: usize = 4;

    pub const INDICES_PER_RECT: usize = 6;

    pub const BYTE_SIZE: usize = size_of::<Self>();

    pub const RECT_BYTE_SIZE: usize = Self::VERTICES_PER_RECT * Self::BYTE_SIZE;

    pub const PRIMITIVE_INDICES: [usize; Self::INDICES_PER_RECT] =
        [0, 1, 3, 1, 2, 3];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: Self::BYTE_SIZE as wgpu::BufferAddress,
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
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Sint32,
                }
            ],
        }
    }

    pub fn from_colored_rect(rect: Rect2f, color: Color) -> [Self; Self::VERTICES_PER_RECT] {
        [
            Self {
                position: rect.bottom_left().into(),
                color: color.into(),
                tex_coords: [0.0, 0.0],
                tex_index: -1
            },
            Self {
                position: rect.bottom_right().into(),
                color: color.into(),
                tex_coords: [1.0, 0.0],
                tex_index: -1
            },
            Self {
                position: rect.top_right().into(),
                color: color.into(),
                tex_coords: [1.0, 1.0],
                tex_index: -1
            },
            Self {
                position: rect.top_left().into(),
                color: color.into(),
                tex_coords: [0.0, 1.0],
                tex_index: -1
            }
        ]
    }

    pub fn from_textured_rect(rect: Rect2f, tex_slot: usize, tex_coords: RectTexCoords) -> [Self; Self::VERTICES_PER_RECT] {
        [
            Self {
                position: rect.bottom_left().into(),
                color: [0.0; 4],
                tex_coords: tex_coords.bottom_left().into(),
                tex_index: tex_slot as i32
            },
            Self {
                position: rect.bottom_right().into(),
                color: [0.0; 4],
                tex_coords: tex_coords.bottom_right().into(),
                tex_index: tex_slot as i32
            },
            Self {
                position: rect.top_right().into(),
                color: [0.0; 4],
                tex_coords: tex_coords.top_right().into(),
                tex_index: tex_slot as i32
            },
            Self {
                position: rect.top_left().into(),
                color: [0.0; 4],
                tex_coords: tex_coords.top_left().into(),
                tex_index: tex_slot as i32
            }
        ]
    }
}