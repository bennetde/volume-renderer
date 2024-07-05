use std::ops::{Index, IndexMut};

use glam::{UVec3, Vec3, Vec4};
use wgpu::{util::{BufferInitDescriptor, DeviceExt}, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BufferAddress, BufferUsages, Device, Queue, ShaderStages};

use super::{init::perlin::init_grid_buffer_perlin, voxel::Voxel};
pub struct VoxelGrid {
    voxels: Vec<Voxel>,
    pub dimensions: UVec3,
    voxels_buffer: wgpu::Buffer,
    pub voxels_bind_group_layout: BindGroupLayout,
    pub voxels_bind_group: BindGroup,
    voxel_grid_buffer: wgpu::Buffer
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VoxelGridUniform {
    dimensions: [u32; 3],
    // Buffer is needed for byte alignment in wgsl and has no further use
    buffer: u32,
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

    pub fn set_color(&mut self, position: UVec3, color: Vec4) {
        let index = self.get_index(position);
        self.voxels[index].set_color(color);
    }
    
    fn get_index(&self, position: UVec3) -> usize {
        if position.x >= self.dimensions.x || position.y >= self.dimensions.y || position.z >= self.dimensions.z {
            panic!("Tried to access grid outside array")
        }
        return (position.x + self.dimensions.x * (position.y + (self.dimensions.y) * position.z)) as usize;
    }

    pub fn update_buffer(&self, queue: &Queue) {
        queue.write_buffer(&self.voxels_buffer, 0, bytemuck::cast_slice(&self.voxels));
    }
}

impl Index<UVec3> for VoxelGrid {
    type Output = Voxel;

    fn index(&self, index: UVec3) -> &Self::Output {
        let index = self.get_index(index);
        &self.voxels[index]
    }
}

impl IndexMut<UVec3> for VoxelGrid { 
    fn index_mut(&mut self, index: UVec3) -> &mut Self::Output {
        let index = self.get_index(index);
        &mut self.voxels[index]
    }
}

impl VoxelGridUniform {
    pub fn new(dimensions: UVec3) -> Self {
        Self {
            dimensions: dimensions.to_array(),
            buffer: 0,
        }
    }
}