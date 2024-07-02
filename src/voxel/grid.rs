use glam::{UVec3, Vec3};
use wgpu::{util::{BufferInitDescriptor, DeviceExt}, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BufferUsages, Device, ShaderStages};

use super::voxel::Voxel;
pub struct VoxelGrid {
    voxels: Vec<Voxel>,
    dimensions: UVec3,
    voxels_buffer: wgpu::Buffer,
    pub voxels_bind_group_layout: BindGroupLayout,
    pub voxels_bind_group: BindGroup,
    voxel_grid_buffer: wgpu::Buffer
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VoxelGridUniform {
    dimensions: [u32; 3]
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
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false, 
                        min_binding_size: None 
                    },
                    count: None
                },

            ]
        });

        let mut voxels: Vec<Voxel> = vec![Voxel::default(); dimensions.x as usize * dimensions.y as usize * dimensions.z as usize];
        voxels[1].set_position(Vec3::new(1.0, 0.0, 0.0));
        voxels[2].set_position(Vec3::new(-2.0, 2.0, 0.0));


        let voxels_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("voxel_grid_buffer_init_descriptor_voxels"),
            contents: bytemuck::cast_slice(&voxels),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST
        });

        let voxel_grid_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("voxel_grid_buffer_init_descriptor_voxel_grid"),
            contents: bytemuck::cast_slice(&[VoxelGridUniform::new(dimensions)]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST
        });
        
        let voxels_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("voxel_grid_bind_group_descriptor_voxels"),
            layout: &layout,
            entries: &[
                BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(
                        wgpu::BufferBinding { 
                            buffer: &voxels_buffer, 
                            offset: 0, 
                            size: None 
                        }
                    )
                },

                BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(
                        wgpu::BufferBinding {
                            buffer: &voxel_grid_buffer,
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
            voxels_bind_group_layout: layout,
            voxels_buffer,
            voxels_bind_group,
            voxel_grid_buffer
        }
    }
}

impl VoxelGridUniform {
    pub fn new(dimensions: UVec3) -> Self {
        Self {
            dimensions: dimensions.to_array()
        }
    }
}