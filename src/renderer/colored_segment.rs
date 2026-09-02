use crate::color::Color;
use crate::constants::MATH_EPSILON;
use crate::math::positive_f32::PositiveF32;
use crate::math::segment2f::Segment2f;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColoredSegmentVertex {
    position: [f32; 2],
    normal: [f32; 2],
    pixel_width: f32,
    color: [f32; 4]
}

impl ColoredSegmentVertex {
    pub const VERTICES_PER_SEGMENT: usize = 4;

    pub const INDICES_PER_SEGMENT: usize = 6;
    
    pub const PRIMITIVE_INDICES: [usize; Self::INDICES_PER_SEGMENT] =
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
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }

    pub fn generate(segment: Segment2f, color: Color, pixel_width: PositiveF32) -> [Self; Self::VERTICES_PER_SEGMENT] {
        let origin = segment.origin().into();
        let destination = segment.destination().into();
        let left_normal = segment.left_normal()
            .normalized(MATH_EPSILON)
            .expect("A segment length is always positive")
            .into();
        let right_normal = segment.right_normal()
            .normalized(MATH_EPSILON)
            .expect("A segment length is always positive")
            .into();
        let color = color.into();

        let bottom_left = Self {
            position: origin,
            normal: right_normal,
            pixel_width: pixel_width.value(),
            color
        };

        let bottom_right = Self {
            position: destination,
            normal: right_normal,
            pixel_width: pixel_width.value(),
            color
        };

        let top_right = Self {
            position: destination,
            normal: left_normal,
            pixel_width: pixel_width.value(),
            color
        };

        let top_left = Self {
            position: origin,
            normal: left_normal,
            pixel_width: pixel_width.value(),
            color
        };

        [bottom_left, bottom_right, top_right, top_left]
    }
}