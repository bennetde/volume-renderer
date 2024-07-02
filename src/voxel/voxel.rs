use glam::Vec3;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Voxel {
    position: [f32; 3],
    radius: f32,
}

impl Voxel {
    pub fn set_position(&mut self, new_pos: Vec3) {
        self.position = new_pos.to_array();
    }
}

impl Default for Voxel {
    fn default() -> Self {
        Self { position: [0.0,0.0,0.0], radius: 1.0 }
    }
}