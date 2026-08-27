use crate::vertex_buffer::VertexBufferLayout;

pub struct Shader {
    shader: wgpu::ShaderModule,
    vertex_entry_point: String,
    fragment_entry_point: String,
    vertex_layout: VertexBufferLayout
}

impl Shader {
    pub fn new(
        device: &wgpu::Device,
        source: &str,
        vertex_entry_point: &str,
        fragment_entry_point: &str,
        vertex_layout: &VertexBufferLayout
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(source.into())
        });
        
        Self {
            shader,
            vertex_entry_point: vertex_entry_point.into(),
            fragment_entry_point: fragment_entry_point.into(),
            vertex_layout: vertex_layout.clone(),
        }
    }

    pub fn module(&self) -> &wgpu::ShaderModule {
        &self.shader
    }

    pub fn vertex_entry_point(&self) -> &str {
        &self.vertex_entry_point
    }

    pub fn fragment_entry_point(&self) -> &str {
        &self.fragment_entry_point
    }
    
    pub fn vertex_layout(&self) -> &VertexBufferLayout {
        &self.vertex_layout
    }
}