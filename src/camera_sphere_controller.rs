use std::f32::consts::PI;

use glam::Vec3;

use crate::{camera::Camera, screenshot::Screenshotter};

/// Defines an amount of position on a sphere where the camera can sit on.
/// The sphere is divided on x horizontal divisions and y vertical divisions, giving x*y possible positions where the camera can sit.
pub struct CameraSphereController {
    vertical_divisions: u32,
    horizontal_divisons: u32,
    pub current_index_x: u32,
    pub current_index_y: u32,
    pub radius: f32,
    pub origin: Vec3,
    is_screenshotting: bool,
}

impl CameraSphereController {
    pub fn new(y_divisions: u32, x_divisions: u32, origin: Vec3, radius: f32) -> Self {
        CameraSphereController {
            vertical_divisions: y_divisions,
            horizontal_divisons: x_divisions,
            current_index_x: 0,
            current_index_y: 4,
            radius: radius,
            origin,
            is_screenshotting: false
        }
    }

    pub fn inc_x_index(&mut self) {
        self.current_index_x += 1;
    }

    pub fn sub_x_index(&mut self) {
        self.current_index_x -= 1;
    }

    pub fn inc_y_index(&mut self) {
        self.current_index_y += 1;
    }

    pub fn sub_y_index(&mut self) {
        self.current_index_y -= 1;
    }

    pub fn set_x_index(&mut self, new: u32) {
        self.current_index_x = new;
    }

    pub fn set_y_index(&mut self, new: u32) {
        self.current_index_y = new;
    }

    pub fn update_camera(&self, camera: &mut Camera) {
        // println!("{}", self.current_index_x);
        camera.transform.position = self.get_position_on_sphere();
    }

    pub fn x_divisions(&self) -> u32 {
        self.horizontal_divisons
    }

    pub fn y_divisions(&self) -> u32 {
        self.vertical_divisions
    }

    /// Using the current indices, returns the according position on the sphere
    fn get_position_on_sphere(&self) -> Vec3 {
        let theta: f32 = self.current_index_x as f32 / self.horizontal_divisons as f32 * PI * 2.0;

        let phi: f32 = self.current_index_y as f32 / self.vertical_divisions as f32 * PI;
        let x = f32::sin(phi) * f32::cos(theta) * self.radius;
        let y = f32::sin(phi) * f32::sin(theta) * self.radius;
        let z = f32::cos(phi) * self.radius;
        let pos = Vec3::new(x, z,-y);

        pos + self.origin
    }

    pub fn start_screenshotting(&mut self) {
        self.is_screenshotting = true;
        self.current_index_x = 0;
        self.current_index_y = 1;
    }
}