#[derive(Copy, Clone)]
pub enum VertexAttribute {
    Float1,
    Float2,
    Float3,
    Float4,
}

impl VertexAttribute {
    const FLOAT_BYTE_SIZE: usize = size_of::<f32>();

    fn byte_size(&self) -> usize {
        match self {
            VertexAttribute::Float1 => Self::FLOAT_BYTE_SIZE,
            VertexAttribute::Float2 => Self::FLOAT_BYTE_SIZE * 2,
            VertexAttribute::Float3 => Self::FLOAT_BYTE_SIZE * 3,
            VertexAttribute::Float4 => Self::FLOAT_BYTE_SIZE * 4
        }
    }

    fn wgpu_type(&self) -> wgpu::VertexFormat {
        match self {
            VertexAttribute::Float1 => wgpu::VertexFormat::Float32,
            VertexAttribute::Float2 => wgpu::VertexFormat::Float32x2,
            VertexAttribute::Float3 => wgpu::VertexFormat::Float32x3,
            VertexAttribute::Float4 => wgpu::VertexFormat::Float32x4
        }
    }
}

pub struct VertexBufferLayout {
    attributes: Vec<wgpu::VertexAttribute>,
    byte_size: usize
}

impl VertexBufferLayout {
    pub fn new(attributes: &[VertexAttribute]) -> Self {
        let mut wgpu_attributes = Vec::new();
        let mut offset = 0;

        for (index, a) in attributes.iter().enumerate() {
            wgpu_attributes.push(wgpu::VertexAttribute {
                format: a.wgpu_type(),
                offset: offset as wgpu::BufferAddress,
                shader_location: index as wgpu::ShaderLocation,
            });
            offset += a.byte_size();
        }

        Self {
            attributes: wgpu_attributes,
            byte_size: offset
        }
    }

    pub fn wgpu_layout(&self) -> wgpu::VertexBufferLayout<'_> {
        wgpu::VertexBufferLayout {
            array_stride: self.byte_size as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &self.attributes
        }
    }
}

pub struct VertexBuffer {
    buffer: wgpu::Buffer,
    layout: VertexBufferLayout
}

impl VertexBuffer {
    pub fn new(device: wgpu::Device, layout: VertexBufferLayout, data: &[u8]) -> Self {
        use wgpu::util::DeviceExt;

        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: data,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );

        Self { buffer, layout }
    }

    pub fn as_wgpu_buffer(&self) -> wgpu::Buffer {
        self.buffer.clone()
    }

    pub fn layout(&self) -> &VertexBufferLayout {
        &self.layout
    }
}