use glam::*;
use crate::transform::Transform;

/// Camera struct that stores all information necessary for perspective rendering.
pub struct Camera {
    pub transform: Transform,
    target: Vec3,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32
}

impl Camera {
    pub fn new(aspect_ratio: f32) -> Camera {
        Camera {
            transform: Transform::default().move_pos(Vec3::new(-5.0, -5.0, -5.0)),
            target: Vec3::ZERO,
            aspect: aspect_ratio,
            fovy: 45.0,
            znear: 0.1,
            zfar: 1000.0,
        }
    }

    pub fn build_view_projection_matrix(&mut self) -> Mat4 {
        let view = Mat4::look_at_rh(self.transform.position, -self.transform.forward(), Vec3::Y);

        let proj = Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar);

        return proj * view;
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    // Position has to be a Vec4 because of what I believe to be alignment issues on the GPU
    position: [f32; 4],
    view_proj: [f32; 16],
    inverse_view_proj: [f32; 16],

}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            position: [0.0,0.0,0.0, 0.0],
            view_proj: Mat4::IDENTITY.to_cols_array(),
            inverse_view_proj: Mat4::IDENTITY.to_cols_array()
        }
    }

    pub fn update_view_proj(&mut self, camera: &mut Camera) {
        let matrix = camera.build_view_projection_matrix();

        self.position = [camera.transform.position.x, camera.transform.position.y, camera.transform.position.z, 0.0];
        self.view_proj = matrix.to_cols_array();
        self.inverse_view_proj = matrix.inverse().to_cols_array();
    }
}