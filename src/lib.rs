mod state;
mod vertex;
mod texture;
mod camera;
mod camera_controller;
mod transform;
mod model;
mod ray_marcher;
mod voxel;

use std::time::Instant;

use state::State;
use vertex::Vertex;
use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};

const VERTICES: &[Vertex] = &[
    Vertex { position: [-1.0, -1.0, 0.0], tex_coords: [0.0, 1.0], }, // A
    Vertex { position: [1.0, -1.0, 0.0], tex_coords: [1.0, 1.0], }, // B
    Vertex { position: [0.0, 1.0, 0.0], tex_coords: [0.5, 0.0], }, // C
];


const INDICES: &[u16] = &[
    0, 1, 2
];

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
                        // println!("{}ms", time.elapsed().as_millis() as f64);
                        state.window().set_title(format!("diffdvr-voxel {}ms", time.elapsed().as_millis()).as_str());
                    }

                    _ => {}
                }
            }
            _ => {}
        }
    }).unwrap();
}

 

 