use std::{fs::File, io::BufWriter};

use glam::Vec3;
use anyhow::Result;
use serde::Serialize;
use crate::{camera::Camera, camera_sphere_controller::CameraSphereController};

/// Helper struct to create screenshots with the camera placed around the sphere
pub struct SphereScreenshotManager {
    is_screenshotting: bool,
    screenshot_info: ScreenshotInformation

}

impl SphereScreenshotManager {
    pub fn new(csp: &CameraSphereController) -> Self {
        Self {
            is_screenshotting: false,
            screenshot_info: ScreenshotInformation::new(csp.x_divisions() as usize * csp.y_divisions() as usize, csp.origin)
        }
    }

    pub fn start_screenshotting(&mut self, csp: &mut CameraSphereController, camera: &mut Camera) {
        self.is_screenshotting = true;
        csp.current_index_x = 0;
        csp.current_index_y = 1;
        self.screenshot_info.positions.clear();
        csp.update_position(camera);
        self.add_position(csp, camera);
    }


    pub fn update_camera(&mut self, csp: &mut CameraSphereController, camera: &mut Camera) -> bool {
        // println!("{}", self.current_index_x);
        if self.is_screenshotting {
            if csp.current_index_x < csp.x_divisions() - 1 {
                csp.current_index_x += 1;
            } else if csp.current_index_y < csp.y_divisions() - 1 {
                csp.current_index_y += 1;
                csp.current_index_x = 0;
                // println!("Increasing y")
            } else {
                self.is_screenshotting = false;
                self.save_positions_to_json("screenshots/cameras.json").unwrap();
                return false;
            }
        }

        csp.update_position(camera);
        if self.is_screenshotting {
            self.add_position(csp, camera)
        }
        self.is_screenshotting
    }

    fn add_position(&mut self, csp: &CameraSphereController, camera: &Camera) {
        self.screenshot_info.positions.push(CameraPositions {
            position: camera.transform.position.to_array(),
            right: camera.transform.right().to_array(),
            up: camera.transform.up().to_array(),
            front: camera.transform.forward().to_array(),
            img: format!("{}.png", csp.get_position_as_string().to_string()),
            fovy: camera.fovy()
        });
        // println!("Add pos: {}", csp.get_position_as_string());
    }

    pub fn save_positions_to_json(&self, path: &str) -> Result<()> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self.screenshot_info)?;
        Ok(())
    }

}


#[derive(Serialize)]
struct CameraPositions {
    pub position: [f32; 3],
    pub right: [f32; 3],
    pub up: [f32;3],
    pub front: [f32;3],
    pub img: String,
    pub fovy: f32,
}

#[derive(Serialize)]
struct ScreenshotInformation {
    look_at: [f32;3],
    positions: Vec<CameraPositions>,

}

impl ScreenshotInformation {
    pub fn new(size: usize, center: Vec3) -> Self {
        Self {
            look_at: center.to_array(),
            positions: Vec::with_capacity(size)
        }
    }
}