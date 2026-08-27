#[derive(Copy, Clone)]
pub enum IndexFormat {
    Uint16 = 0,
    Uint32 = 1,
}

impl IndexFormat {
    pub fn wgpu_format(&self) -> wgpu::IndexFormat {
        match self {
            IndexFormat::Uint16 => wgpu::IndexFormat::Uint16,
            IndexFormat::Uint32 => wgpu::IndexFormat::Uint32,
        }
    }
}

pub struct IndexBuffer {
    buffer: wgpu::Buffer,
    format: IndexFormat
}

impl IndexBuffer {
    pub fn new_u16(device: &wgpu::Device, data: &[u16]) -> Self {
        use wgpu::util::DeviceExt;

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(data),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            }
        );

        Self { buffer: index_buffer, format: IndexFormat::Uint16 }
    }

    pub fn new_u32(device: &wgpu::Device, data: &[u32]) -> Self {
        use wgpu::util::DeviceExt;

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(data),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            }
        );

        Self { buffer: index_buffer, format: IndexFormat::Uint32 }
    }

    pub fn format(&self) -> IndexFormat {
        self.format
    }
    
    pub fn as_wgpu_buffer(&self) -> wgpu::Buffer {
        self.buffer.clone()
    }
}