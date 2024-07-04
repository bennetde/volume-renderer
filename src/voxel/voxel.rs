use glam::{Vec3, Vec4};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Voxel {
    color: [f32; 4]
}

impl Voxel {
    pub fn set_color(&mut self, new_color: Vec4) {
        self.color = new_color.to_array();
    }
}

impl Default for Voxel {
    fn default() -> Self {
        Self { color: [0.0,0.0,0.0,1.0]}
    }
}