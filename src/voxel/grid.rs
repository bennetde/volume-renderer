use glam::UVec3;
use wgpu::{util::{BufferInitDescriptor, DeviceExt}, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BufferUsages, Device, ShaderStages};

use super::voxel::Voxel;
pub struct VoxelGrid {
    voxels: Vec<Voxel>,
    dimensions: UVec3,
    buffer: wgpu::Buffer,
    pub bind_group_layout: BindGroupLayout,
    pub bind_group: BindGroup,
}

impl VoxelGrid {
    pub fn new(dimensions: UVec3, device: &Device) -> Self {
        let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("voxel_grid_bind_group_layout_descriptor"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false, 
                        min_binding_size: None 
                    },
                    count: None
                }
            ]
        });

        let voxels: Vec<Voxel> = vec![Voxel(0); dimensions.x as usize * dimensions.y as usize * dimensions.z as usize];

        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("voxel_grid_buffer_init_descriptor"),
            contents: bytemuck::cast_slice(&voxels),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST
        });
        
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("voxel_grid_bind_group_descriptor"),
            layout: &layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(
                        wgpu::BufferBinding { 
                            buffer: &buffer, 
                            offset: 0, 
                            size: None 
                        }
                    )
                }
            ]
        });

        Self {
            voxels,
            dimensions,
            bind_group_layout: layout,
            buffer,
            bind_group
        }
    }
}