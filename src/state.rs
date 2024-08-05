use std::{rc::Rc, thread::current, time::SystemTime};

use chrono::Utc;
use egui_wgpu::ScreenDescriptor;
use glam::Vec3;
use wgpu::{util::DeviceExt, Color};
use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};
use crate::{camera::{Camera, CameraUniform}, camera_controller::CameraController, camera_sphere_controller::{self, CameraSphereController}, gui::EguiRenderer, ray_marcher::RayMarcher, screenshot::Screenshotter};

/// Handles and stores the state of the application. 
/// Additionally holds data needed for rendering, but this should be moved into it's own struct in the future.
pub struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: &'a Window,
    clear_color: Color,
    camera: Camera,
    camera_controller: CameraController,
    camera_sphere_controller: CameraSphereController,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    ray_marcher: RayMarcher,
    egui_renderer: EguiRenderer,
    screenshotter: Screenshotter,

    frametime: f64,
    should_screenshot: bool,
}

impl<'a> State<'a> {

    /// Creates a new state and initializes a WebGPU Instance for the given window.
    pub async fn new(window: &'a Window) -> State<'a> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,

            },
        ).await.unwrap();

        let mut limits = wgpu::Limits::default();
        limits.max_buffer_size = 536870912;
        limits.max_storage_buffer_binding_size = 536870912;
        

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: limits,
                label: None
            },
            None
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        // let surface_format = surface_caps.formats.iter()
        //     .find(|f| f.is_srgb())
        //     .copied()
        //     .unwrap_or(surface_caps.formats[0]);

        let surface_format = wgpu::TextureFormat::Rgba8UnormSrgb;

        println!("{:?}", surface_caps.present_modes);

        // If possible disable VSync
        let present_mode = if surface_caps.present_modes.contains(&wgpu::PresentMode::Immediate) {
            wgpu::PresentMode::Immediate
            // surface_caps.present_modes[0]
        } else {
            surface_caps.present_modes[0]
        };

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: present_mode,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2
        };

        surface.configure(&device, &config);

        let mut camera = Camera::new(config.width as f32 / config.height as f32);
        let camera_controller = CameraController::new(100.0);

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&mut camera);

        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
            }
        );

        let camera_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("camera_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer { 
                            ty: wgpu::BufferBindingType::Uniform, 
                            has_dynamic_offset: false, 
                            min_binding_size: None 
                        },
                        count: None
                    }
                ]
            }
        );

        let camera_bind_group = Rc::new(device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding()
                }
            ],
            label: Some("camera_bind_group")
        }));

        let clear_color = wgpu::Color {
            r: 0.1,
            g: 0.2,
            b: 0.1,
            a: 1.0,
        };

        let ray_marcher = RayMarcher::new(&device, &queue, &config, Rc::clone(&camera_bind_group));

        let egui_renderer = EguiRenderer::new(&device, config.format, None, 1, &window);

        let screenshotter = Screenshotter::new(&device, &config);

        let camera_sphere_controller = CameraSphereController::new(4, 8, Vec3::ONE * 16.0, 100.0);

        Self {
            window,
            surface,
            config,
            device,
            queue,
            size,
            clear_color,
            camera,
            camera_controller,
            camera_sphere_controller,
            camera_uniform,
            camera_buffer,
            ray_marcher,
            egui_renderer,
            screenshotter,
            should_screenshot: false,
            frametime: 0.0,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    // Returns the size of the window
    pub fn size(&self) -> PhysicalSize<u32> {
        self.size
    }

    // Called on window resize
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.camera.set_aspect_ratio(new_size.width as f32 / new_size.height as f32);

            // Screenshotter has to be recreated after resizing the window
            self.screenshotter = Screenshotter::new(&self.device, &self.config);
        }
    }

    // Called for handling different input events
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.egui_renderer.handle_input(&self.window, &event);

        match event {
            WindowEvent::CursorMoved { device_id: _, position } => {
                self.clear_color.r = position.x / self.size.width as f64;
                self.clear_color.g = position.y / self.size.height as f64;

                return true;
            }
            _ => {}
        }

        self.camera_controller.process_events(event)
    }

    pub fn update(&mut self) {
        // self.camera_controller.update_camera(&mut self.camera, 1.0/60.0);
        self.should_screenshot = self.camera_sphere_controller.update_camera(&mut self.camera);
        self.camera.transform.look_to(Vec3::ONE * 16.0, Vec3::NEG_Y);
        // self.camera.look_dir = self.camera.transform.position - Vec3::ONE * 16.0;
        self.camera_uniform.update_view_proj(&mut self.camera);
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Draw Raymarch Render Pass
        let raymarch_command = self.ray_marcher.draw(&self.device, &view);

        // Draw GUI
        let mut gui_encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("GUI Render Encoder"),
        });

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: self.window.scale_factor() as f32,
        };

        self.egui_renderer.draw(
                &self.device,
                &self.queue,
                &mut gui_encoder,
                &self.window,
                &view,
                screen_descriptor,
                |ctx| {
                    egui::Window::new("Window Test").default_open(true)
                    .show(&ctx, |ui| {
                        ui.label(format!("Frametime: {}", self.frametime));
                        if ui.button("Screenshot").clicked() {
                            self.should_screenshot = true;
                        }
                        
                        if ui.button("Screenshot All").clicked() {
                            self.camera_sphere_controller.start_screenshotting();
                            self.should_screenshot = true;
                        }

                        let max_x = self.camera_sphere_controller.x_divisions();
                        let slider = egui::Slider::new(&mut self.camera_sphere_controller.current_index_x, 0..=max_x).text("X Arc");
                        ui.add(slider);

                        let max_y = self.camera_sphere_controller.y_divisions();
                        let slider = egui::Slider::new(&mut self.camera_sphere_controller.current_index_y, 1..=max_y-1).text("Y Arc");
                        ui.add(slider);

                        let slider = egui::Slider::new(&mut self.camera_sphere_controller.radius, 0.0..=1000.0).text("Radius");
                        ui.add(slider);
                    });
                }
        );

        let gui_command = gui_encoder.finish();

        // Ensure that the screenshot is taken before the GUI is rendered
        let mut commands = vec![raymarch_command];
        if self.should_screenshot {
            commands.push(self.screenshotter.screenshot(&output, &self.config, &self.device));
        }
        commands.push(gui_command);
        self.queue.submit(commands);
        
        output.present();

        if self.should_screenshot {
            // let current_time = Utc::now().timestamp();
            let filename = format!("screenshots/{}.png", self.camera_sphere_controller.get_position_as_string());
            println!("Screenshotting: {}", filename);
            let fut = self.screenshotter.save_screenshot_to_disk(&self.device, &self.config, filename.as_str());
            pollster::block_on(fut);
            self.should_screenshot = false;
        }
        Ok(())
    }

    pub fn set_frametime(&mut self, frametime: f64) {
        self.frametime = frametime;
    }
}