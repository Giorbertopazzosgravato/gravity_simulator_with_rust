use cgmath::num_traits::{Float, pow};
use cgmath::Vector3;
use wgpu::VertexBufferLayout;

pub const GRAVITATIONAL_CONSTANT: f64 = 6.67430e-11;
pub const PARTICLE_MASS: f64 = 2.0;
pub struct Instance {
    pub position: Vector3<f32>,
    pub color:    Vector3<f32>,
    pub forces:   Vector3<f32>,
}
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw{
    position: [f32 ; 3],
    color:    [f32 ; 3],
}
impl Instance {
    pub fn to_raw(&self) -> InstanceRaw{
        InstanceRaw{
            position: *self.position.as_ref(),
            color: *self.color.as_ref(),
        }
    }
    pub fn update_forces(&mut self, forces: Vector3<f32>){
        self.forces.x += forces.x;
        self.forces.y += forces.y;
        self.forces.z += forces.z;
        Self::update_position(self);
    }
    pub fn calculate_gravitational_pull(&self) -> Vector3<f32> {
        let BLACK_HOLE_MASS: f64 = - 10_000_000_000.0; // idfk why the mass has to be negative please help lmao its so fucking fun
        let x1 = 0.0;
        let y1 = 0.0;
        let x2 = self.position[0] as f64 * 100.0;
        let y2 = self.position[1] as f64 * 100.0;
        let delta_x = x2 - x1;
        let delta_y = y2 - y1;
        let distance = f64::sqrt(pow(x2 - x1, 2) + pow(y2 - y1, 2));
        let gravitational_pull_x = (GRAVITATIONAL_CONSTANT * BLACK_HOLE_MASS * PARTICLE_MASS * delta_x) / pow(distance, 2);
        let gravitational_pull_y = (GRAVITATIONAL_CONSTANT * BLACK_HOLE_MASS * PARTICLE_MASS * delta_y) / pow(distance, 2);
        Vector3::new(gravitational_pull_x as f32, gravitational_pull_y as f32, 0.0)
    }
    fn update_position(&mut self){
        self.position += self.forces;
    }
}

impl InstanceRaw{
    pub const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![3 => Float32x3, 4 => Float32x3];
    pub fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout{
            array_stride: std::mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}