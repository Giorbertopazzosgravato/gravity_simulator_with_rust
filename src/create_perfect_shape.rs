use crate::vertex::Vertex;

pub struct CreatePerfectShape;
impl CreatePerfectShape {
    pub fn create_shape_optimized(num_vertices: u32, color: [f32; 3]) -> (Vec<Vertex>, Vec<u16>){
        let mut vertices: Vec<Vertex> = Vec::new();

        for i in 1..= num_vertices {
            vertices.push(Vertex{
                position: [
                    f32::cos(f32::to_radians(i as f32*(90./(num_vertices as f32/4.)))) / 64.0,
                    f32::sin(f32::to_radians(i as f32*(90./(num_vertices as f32/4.)))) / 64.0,
                    0.0
                ],
                color,
            });
        }
        let mut indices: Vec<u16> = Vec::new();
        for i in 1..vertices.len() - 1 {
            indices.push(0);
            indices.push(i as u16);
            indices.push((i + 1) as u16);
        }

        (vertices, indices)
    }
}