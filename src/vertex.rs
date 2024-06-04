use std::fmt::{Display, Formatter};
use wgpu::VertexBufferLayout;
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex{
    pub position: [f32; 3],
    pub color: [f32; 3],
}
impl Vertex{
    pub const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];
    pub fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout{
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}
impl Display for Vertex{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "color: {:?}", self.color)
    }
}