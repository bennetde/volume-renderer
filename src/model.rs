use std::ops::Range;

use wgpu::{util::DeviceExt, Buffer, Device};
use crate::vertex::Vertex;

#[allow(dead_code)]
pub struct Model {
    name: String,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    num_elements: u32,
}

impl Model {
    pub fn new(device: &Device, name: &str, vertices: &[Vertex], indices: &[u16]) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{:?} Vertex Buffer", name)),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{:?} Index Buffer", name)),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX
        });
        
        Self {
            name: name.to_string(),
            vertex_buffer,
            index_buffer,
            num_elements: indices.len() as u32,
        }
    }
}

// model.rs
pub trait DrawModel<'a> {
    fn draw_model(&mut self, model: &'a Model);
    fn draw_model_instanced(
        &mut self,
        model: &'a Model,
        instances: Range<u32>,
    );
}
impl<'a, 'b> DrawModel<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_model(&mut self, mesh: &'b Model) {
        self.draw_model_instanced(mesh, 0..1);
    }

    fn draw_model_instanced(
        &mut self,
        mesh: &'b Model,
        instances: Range<u32>,
    ){
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }
}

 