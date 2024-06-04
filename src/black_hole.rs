use wgpu::{Buffer, BufferUsages, Device};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use crate::create_perfect_shape::CreatePerfectShape;

pub struct BlackHole{
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    num_indices: u32,
}
impl BlackHole {
    pub fn new(device: &Device) -> Self {
        let (vertices, indices ) = CreatePerfectShape::create_shape_optimized(30, [0.0, 0.0, 0.0]);
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor{
            label: Some("black hole vertex"),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&BufferInitDescriptor{
            label: Some("black hole index"),
            contents: bytemuck::cast_slice(&indices),
            usage: BufferUsages::INDEX,
        });
        Self{
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
        }
    }
    pub fn get_buffers(&self) -> (&Buffer, &Buffer, u32){
        (&self.vertex_buffer, &self.index_buffer, self.num_indices)
    }
}