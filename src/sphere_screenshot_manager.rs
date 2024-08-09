use std::{fs::File, io::{BufWriter, Write}};

use glam::Vec3;
use anyhow::Result;
use serde::Serialize;
use crate::{camera::Camera, camera_sphere_controller::{self, CameraSphereController}};

pub struct SphereScreenshotManager {
    is_screenshotting: bool,
    positions: Vec<CameraPositions>

}

impl SphereScreenshotManager {
    pub fn new(csp: &CameraSphereController) -> Self {
        Self {
            is_screenshotting: false,
            positions: Vec::with_capacity(csp.x_divisions() as usize * csp.y_divisions() as usize)
        }
    }

    pub fn start_screenshotting(&mut self, csp: &mut CameraSphereController) {
        self.is_screenshotting = true;
        csp.current_index_x = 0;
        csp.current_index_y = 1;
        self.positions.clear();
    }


    pub fn update_camera(&mut self, csp: &mut CameraSphereController, camera: &mut Camera) -> bool {
        // println!("{}", self.current_index_x);
        if self.is_screenshotting {
            if csp.current_index_x < csp.x_divisions() {
                csp.current_index_x += 1;
            } else if csp.current_index_y < csp.y_divisions() - 1 {
                csp.current_index_y += 1;
                csp.current_index_x = 0;
            } else {
                self.is_screenshotting = false;
                self.save_positions_to_json("screenshots/camera.json").unwrap();
                return false;
            }
        }

        csp.update_position(camera);
        self.positions.push(CameraPositions {
            position: camera.transform.position.to_array(),
            right: camera.transform.right().to_array(),
            up: camera.transform.up().to_array()
        });
        self.is_screenshotting
    }

    pub fn save_positions_to_json(&self, path: &str) -> Result<()> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self.positions)?;
        Ok(())
    }

}


#[derive(Serialize)]
struct CameraPositions {
    pub position: [f32; 3],
    pub right: [f32; 3],
    pub up: [f32;3],
}