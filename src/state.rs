use std::{rc::Rc, time::Duration};
use egui::menu;
use egui_wgpu::ScreenDescriptor;
use glam::Vec3;
use rfd::AsyncFileDialog;
use wgpu::{util::DeviceExt, Color};
use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};
use crate::{camera::{Camera, CameraUniform}, camera_controller::CameraController, camera_sphere_controller::CameraSphereController, gui::EguiRenderer, ray_marcher::RayMarcher, screenshot::Screenshotter, sphere_screenshot_manager::SphereScreenshotManager};

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
    sphere_screenshot_manager: SphereScreenshotManager,
    frametime: Duration,
    should_screenshot: bool,
    free_move: bool,
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
        limits.max_buffer_size = 18446744073709551615;
        // limits.max_storage_buffer_binding_size = 2147483648;
        limits.max_texture_dimension_3d = 4096;
        

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

        let surface_format = wgpu::TextureFormat::Rgba8Unorm;

        println!("{:?}", surface_caps.present_modes);

        // If possible disable VSync
        // let present_mode = if surface_caps.present_modes.contains(&wgpu::PresentMode::Immediate) {
        //     wgpu::PresentMode::Immediate
        //     // surface_caps.present_modes[0]
        // } else {
        //     surface_caps.present_modes[0]
        // };
        let present_mode = surface_caps.present_modes[0];

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
        let camera_controller = CameraController::new(20.0);

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

        let camera_sphere_controller = CameraSphereController::new(8, 8, Vec3::ZERO, 100.0);

        let sphere_screenshot_manager = SphereScreenshotManager::new(&camera_sphere_controller);

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
            sphere_screenshot_manager,
            should_screenshot: false,
            frametime: Duration::ZERO,
            free_move: true,
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
        if self.free_move {
            self.camera_controller.update_camera(&mut self.camera, self.frametime.as_secs_f32() as f32);
            self.camera.transform.look_to(Vec3::ZERO, Vec3::Y);
        } else {
            self.should_screenshot = self.sphere_screenshot_manager.update_camera(&mut self.camera_sphere_controller,&mut self.camera);
        }
        // self.camera.transform.look_to(Vec3::ONE * 16.0, Vec3::NEG_Y);
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
                    // Draw Top Bar Menu
                    egui::TopBottomPanel::top("my_panel").show(&ctx, |ui| {
                        menu::bar(ui, |ui| {
                            ui.menu_button("File", |ui| {
                                if ui.button("Open NetCDF").clicked() {
                                    let file_path = open_file_menu("NetCDF", &["nc"]).unwrap();
                                    if let Some(file_path) = file_path {
                                        crate::loaders::netcdf::open_voxel_grid(&file_path, &mut self.ray_marcher.voxel_grid, &self.device, &self.queue).unwrap();
                                        self.window.set_title(&file_path);
                                    }
                                }

                                if ui.button("Open DAT").clicked() {
                                    let file_path = open_file_menu("DAT", &["dat"]).unwrap();
                                    if let Some(file_path) = file_path {
                                        crate::loaders::dat::open_voxel_grid(&file_path, &mut self.ray_marcher.voxel_grid, &self.device, &self.queue).unwrap();
                                        self.window.set_title(&file_path);
                                    }

                                }

                                if ui.button("Export NetCDF").clicked() {
                                    crate::loaders::netcdf::write_voxel_grid("test.nc", &self.ray_marcher.voxel_grid).unwrap();
                                }
                            });

                            ui.menu_button("Compare", |ui| {
                                if ui.button("Compare NetCDF to Ground-Truth").clicked() {
                                    let file_path = open_file_menu("NetCDF", &["nc"]).unwrap();
                                    if let Some(path) = file_path {
                                        let color_function_active = self.ray_marcher.voxel_grid.transfer_function_colors.use_transfer_function_active();
                                        let result = crate::compare::netcdf::compare_to_netcdf_rmse(&path, &mut self.ray_marcher.voxel_grid, !color_function_active);
                                        println!("{:?}", result);
                                    }
                                }
                            });
                        });
                        
                        });

                    // Draw Main Window UI
                    egui::Window::new("").default_open(true)
                    .show(&ctx, |ui| {

                        ui.label(format!("Frametime: {}ms", self.frametime.as_millis()));

                        // Screenshotting 

                        if ui.button("Screenshot").clicked() {
                            self.should_screenshot = true;
                        }
                        
                        if ui.button("Screenshot All").clicked() {
                            self.free_move = false;
                            self.sphere_screenshot_manager.start_screenshotting(&mut self.camera_sphere_controller, &mut self.camera);
                            self.should_screenshot = true;
                        }
                        

                        // Camera Sphere Controller
                        // X Divisions
                        let mut val = self.camera_sphere_controller.x_divisions();
                        let slider = egui::Slider::new(&mut val, 0..=100).text("X Divisions");
                        ui.add(slider);
                        if val != self.camera_sphere_controller.x_divisions() {
                            self.camera_sphere_controller.set_horizontal_divisions(val);
                        }

                        // Y Divisions
                        let mut val = self.camera_sphere_controller.y_divisions();
                        let slider = egui::Slider::new(&mut val, 0..=100).text("Y Divisions");
                        ui.add(slider);
                        if val != self.camera_sphere_controller.y_divisions() {
                            self.camera_sphere_controller.set_vertical_divisions(val);
                        }

                        // Current Positions
                        let max_x = self.camera_sphere_controller.x_divisions();
                        let slider = egui::Slider::new(&mut self.camera_sphere_controller.current_index_x, 0..=max_x-1).text("X Arc");
                        ui.add(slider);

                        let max_y = self.camera_sphere_controller.y_divisions();
                        let slider = egui::Slider::new(&mut self.camera_sphere_controller.current_index_y, 1..=max_y-1).text("Y Arc");
                        ui.add(slider);

                        let slider = egui::Slider::new(&mut self.camera_sphere_controller.radius, 1.0..=1000.0).text("Radius");
                        ui.add(slider);

                        // Debug Info

                        ui.label(format!("Position: {:.2}", self.camera.transform.position));
                        ui.label(format!("Right: {:.2}", self.camera.transform.right()));
                        ui.label(format!("Up: {:.2}", self.camera.transform.up()));
                        ui.label(format!("Look dir: {:.2}", self.camera.transform.forward()));
                        ui.label(format!("Size: {:?}", self.window.inner_size()));
                        ui.checkbox(&mut self.free_move, "Free-Move");

                        // Attenuation + Transfer Function

                        let slider = egui::Slider::new(&mut self.ray_marcher.voxel_grid.attenuation, 0.0..=100.0).text("Attenuation");
                        if ui.add(slider).changed() {
                            self.ray_marcher.voxel_grid.update_voxel_grid_buffer(&self.queue);
                        }

                        let mut is_checked = self.ray_marcher.voxel_grid.transfer_function_colors.use_transfer_function[0] != 0;
                        if ui.checkbox(&mut is_checked, "Use Transfer Function Colors").changed() {
                            self.ray_marcher.voxel_grid.transfer_function_colors.set_transfer_function_active(is_checked);
                            self.ray_marcher.voxel_grid.update_transfer_function_buffer(&self.queue);
                        }
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
            let fut = self.screenshotter.save_screenshot_to_disk(&self.device, &self.config, filename.as_str());
            pollster::block_on(fut);
            self.should_screenshot = false;
        }
        Ok(())
    }

    pub fn set_frametime(&mut self, frametime: Duration) {
        self.frametime = frametime;
    }
}

/// Helper Function to easily open a File Dialog
fn open_file_menu(filter_name: &str, extensions: &[&str]) -> anyhow::Result<Option<String>> {
    let mut file_menu = None;

    let future = async {
        let file: Option<rfd::FileHandle> = AsyncFileDialog::new()
            .add_filter(filter_name, extensions)
            .set_directory(std::env::current_dir().unwrap())
            .pick_file()
            .await;

        if let Some(file_handle) = file {
            file_menu = Some(file_handle.path().to_str().unwrap().to_string());
        }
    };
    pollster::block_on(future);

    Ok(file_menu)
}