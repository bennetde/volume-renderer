use std::rc::Rc;

use glam::UVec3;
use wgpu::{BindGroup, Color, Device, FragmentState, PrimitiveState, Queue, RenderPass, RenderPipeline, RenderPipelineDescriptor, SurfaceConfiguration, TextureView, VertexState};
use crate::{camera, model::{DrawModel, Model}, vertex::Vertex, voxel::{self, grid::VoxelGrid, voxel::Voxel}};


const VERTICES: &[Vertex] = &[
    Vertex { position: [-1.0, -1.0, 0.0], tex_coords: [0.0, 0.0], }, // A
    Vertex { position: [3.0, -1.0, 0.0], tex_coords: [2.0, 0.0], }, // B
    Vertex { position: [-1.0, 3.0, 0.0], tex_coords: [0.0, 2.0], }, // C
];

const INDICES: &[u16] = &[
    0, 1, 2
];

pub struct RayMarcher {
    render_pipeline: RenderPipeline,
    screen_model: Model,
    camera_bind_group: Rc<BindGroup>,
    voxel_grid: VoxelGrid
}

impl RayMarcher {
    pub fn new(device: &Device, config: &SurfaceConfiguration, camera_bind_group: Rc<BindGroup>) -> Self {
        
        let shader = device.create_shader_module(wgpu::include_wgsl!("raymarcher.wgsl"));
        let voxel_grid = VoxelGrid::new(UVec3::new(300,300,300), &device);

        // --- UNIFORMS ---
        let camera_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("raymarcher_camera_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
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

        // --- RENDER PIPELINE ---
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Raymarching Render Pipeline Layout"),
            bind_group_layouts: &[
                &camera_bind_group_layout,
                &voxel_grid.voxels_bind_group_layout
            ],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("RayMarching Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                entry_point: "vs_main",
                module: &shader,
                buffers: &[
                    Vertex::desc()
                ]
            },
            fragment: Some(FragmentState {
                entry_point: "fs_main",
                module: &shader,
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })]
            }),
            primitive: PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let screen_model = Model::new(&device, "ScreenOverlay", VERTICES, INDICES);
        
        RayMarcher {
            render_pipeline,
            screen_model,
            camera_bind_group,
            voxel_grid
        }
    }

    pub fn draw(&self, device: &Device, view: &TextureView, queue: &Queue) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Raymarching Render Encoder"),
        });
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {

            label: Some("Raymarching Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(Color::BLACK),
                    store: wgpu::StoreOp::Store
                }
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass.set_bind_group(1, &self.voxel_grid.voxels_bind_group, &[]);
        render_pass.draw_model(&self.screen_model);

        drop(render_pass);
        queue.submit(std::iter::once(encoder.finish()));



    }
}