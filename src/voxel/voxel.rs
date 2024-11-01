#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Voxel {
    pub color: [u8; 4]
}

impl Voxel {
    pub fn set_color(&mut self, new_color: [u8; 4]) {
        self.color = new_color;
    }
}

impl Default for Voxel {
    fn default() -> Self {
        Self { color: [0, 0, 0, 255]}
    }
}