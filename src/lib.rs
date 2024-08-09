mod state;
mod vertex;
mod texture;
mod texture_3d;
mod camera;
mod camera_controller;
mod camera_sphere_controller;
mod transform;
mod model;
mod ray_marcher;
mod voxel;
mod gui;
mod screenshot;
mod sphere_screenshot_manager;

use std::time::Instant;

use state::State;
use winit::{
    event::*, event_loop::EventLoop, keyboard::{KeyCode, PhysicalKey}, window::WindowBuilder
};

/// Main function for this application.
/// 
/// Creates the window and runs it in an event loop until the application is exited.<br>
/// +X = Right, +Y = Forward, +Z = Up
pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = State::new(&window).await;

    event_loop.run(move |event,control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                state: ElementState::Pressed,
                                physical_key: PhysicalKey::Code(KeyCode::Escape),
                                ..
                            },
                        ..
                    } => control_flow.exit(),

                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                state: ElementState::Pressed,
                                physical_key: PhysicalKey::Code(KeyCode::F11),
                                ..
                            },
                        ..
                    } => {
                        // Toggle Fullscreen
                        if state.window().fullscreen().is_some() {
                            state.window().set_fullscreen(None);
                        } else {
                            state.window().set_fullscreen(Some(winit::window::Fullscreen::Borderless(state.window().current_monitor())));
                        } 
                    }

                    WindowEvent::Resized(physycal_size) => {
                        state.resize(*physycal_size);
                    }

                    WindowEvent::RedrawRequested => {
                        state.window().request_redraw();
                        let time = Instant::now();
                        state.update();
                        match state.render() {
                            Ok(_) => {}
                            Err(wgpu::SurfaceError::Lost) => state.resize(state.size()),
                            Err(wgpu::SurfaceError::OutOfMemory) => control_flow.exit(),
                            Err(e) => eprintln!("{:?}", e),
                        }
                        
                        state.set_frametime(time.elapsed().as_secs_f64());
                    }

                    _ => {}
                }
            }
            _ => {}
        }
    }).unwrap();
}

 

 