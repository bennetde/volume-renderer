use std::fmt::Display;
use glam::{Mat4, Quat, Vec3};

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

#[allow(dead_code)]
impl Transform {
    pub fn new(pos: Vec3, rot: Quat, sca: Vec3) -> Self {
        Transform {
            position: pos,
            rotation: rot,
            scale: sca,
        }
    }

    pub fn to_model_matrix(&self) -> Mat4 {
        Mat4::from_scale(self.scale) * Mat4::from_quat(self.rotation) * Mat4::from_translation(self.position)
    }

    pub fn euler_angles(&self) -> Vec3 {
        self.rotation.to_euler(glam::EulerRot::XYZ).into()
    }

    pub fn forward(&self) -> Vec3 {
        self.rotation * Vec3::Z
    }

    pub fn right(&self) -> Vec3 {
        self.rotation * Vec3::X
    }

    pub fn up(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }

    pub fn back(&self) -> Vec3 {
        self.rotation * Vec3::NEG_Z
    }

    pub fn left(&self) -> Vec3 {
        self.rotation * Vec3::NEG_X
    }

    pub fn down(&self) -> Vec3 {
        self.rotation * Vec3::NEG_Y
    }

    pub fn move_pos(mut self, pos: Vec3) -> Self {
        self.position += pos;
        self
    }

    pub fn look_to(&mut self, pos: Vec3, up: Vec3) {
        let d = (pos - self.position).normalize();
        let quat = Quat::from_mat4(&Mat4::look_to_rh(self.position, d, up)).conjugate();
        self.rotation = quat;
    }

    pub fn with_rotation(mut self, rot: Quat) -> Self {
        self.rotation = rot;
        self
    }
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl Display for Transform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.position, self.forward(), self.scale)
    }
}