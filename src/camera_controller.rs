use winit::{event::{ElementState, KeyEvent, WindowEvent}, keyboard::{KeyCode, PhysicalKey}};

use crate::camera::Camera;

pub struct CameraController {
    speed: f32,
    is_forward_pressed: bool,
    is_right_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_up_pressed: bool,
    is_down_pressed: bool,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        CameraController {
            speed,
            is_forward_pressed: false,
            is_right_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_down_pressed: false,
            is_up_pressed: false
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    state,
                    physical_key: PhysicalKey::Code(keycode),
                    ..
                },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    KeyCode::KeyS => {
                        self.is_forward_pressed = is_pressed;
                        true
                    },
                    KeyCode::KeyD => {
                        self.is_left_pressed = is_pressed;
                        true
                    },
                    KeyCode::KeyW => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyA => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    KeyCode::Space => {
                        self.is_up_pressed = is_pressed;
                        true
                    }
                    KeyCode::ControlLeft => {
                        self.is_down_pressed = is_pressed;
                        true
                    }
                _ => false
                }
            }
            _ => false
        }
    }

    #[allow(unused)]
    pub fn update_camera(&self, camera: &mut Camera, delta: f32) {
        if self.is_forward_pressed {
            camera.transform.position += camera.transform.forward() * self.speed * delta;
        }
        if self.is_right_pressed {
            camera.transform.position += camera.transform.right() * self.speed * delta;
        }
        if self.is_backward_pressed {
            camera.transform.position += camera.transform.back() * self.speed * delta;
        }
        if self.is_left_pressed {
            camera.transform.position += camera.transform.left() * self.speed * delta;
        }
        if self.is_up_pressed {
            camera.transform.position += camera.transform.up() * self.speed * delta;
        }
        if self.is_down_pressed {
            camera.transform.position += camera.transform.down() * self.speed * delta;
        }
    }
}