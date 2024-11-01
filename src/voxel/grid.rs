use std::ops::{Index, IndexMut};

use glam::{UVec3, Vec3Swizzles};
use wgpu::{util::{BufferInitDescriptor, DeviceExt}, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BufferUsages, Device, Queue, ShaderStages};

use crate::texture_3d::Texture3D;

use super::voxel::Voxel;
pub struct VoxelGrid {
    voxels: Vec<Voxel>,
    pub dimensions: UVec3,
    // voxels_buffer: wgpu::Buffer,
    pub voxels_bind_group_layout: BindGroupLayout,
    pub voxels_bind_group: BindGroup,
    pub voxel_texture: Texture3D,
    pub voxel_texture_bind_group_layout: BindGroupLayout,
    pub voxel_texture_bind_group: BindGroup,
    voxel_grid_buffer: wgpu::Buffer,
    raymarch_color_buffer: wgpu::Buffer,
    pub attenuation: f32,
    pub transfer_function_colors: RaymarchTransferFunctionColors,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VoxelGridUniform {
    dimensions: [u32; 4],
    box_min: [f32; 4],
    box_size: [f32; 4],
    // Buffer is needed for byte alignment in wgsl and has no further use
    buffer: [f32; 4],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RaymarchTransferFunctionColors {
    pub color_a: [f32; 4],
    pub color_b: [f32; 4],
    pub color_c: [f32; 4],
    pub use_transfer_function: [u32; 4],
}

impl VoxelGrid {
    pub fn new(dimensions: UVec3, device: &Device, queue: &Queue) -> Self {
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
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None
                    },
                    count: None
                }
                // BindGroupLayoutEntry {
                //     binding: 1,
                //     visibility: ShaderStages::FRAGMENT,
                //     ty: wgpu::BindingType::Buffer { 
                //         ty: wgpu::BufferBindingType::Storage { read_only: true },
                //         has_dynamic_offset: false, 
                //         min_binding_size: None 
                //     },
                //     count: None
                // },

            ]
        });

        let voxels: Vec<Voxel> = vec![Voxel::default(); dimensions.x as usize * dimensions.y as usize * dimensions.z as usize];


        // let voxels_buffer = device.create_buffer_init(&BufferInitDescriptor {
        //     label: Some("voxel_grid_buffer_init_descriptor_voxels"),
        //     contents: bytemuck::cast_slice(&voxels),
        //     usage: BufferUsages::STORAGE | BufferUsages::COPY_DST
        // });

        let raymarch_color_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("raymarch_color_buffer"),
            contents: bytemuck::cast_slice(&[RaymarchTransferFunctionColors::new()]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST
        });

        let voxel_grid_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("voxel_grid_buffer_init_descriptor_voxel_grid"),
            contents: bytemuck::cast_slice(&[VoxelGridUniform::new(dimensions, 1.0)]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST
        });
        
        let voxels_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("voxel_grid_bind_group_descriptor_voxels"),
            layout: &layout,
            entries: &[
                // BindGroupEntry {
                //     binding: 1,
                //     resource: wgpu::BindingResource::Buffer(
                //         wgpu::BufferBinding { 
                //             buffer: &voxels_buffer, 
                //             offset: 0, 
                //             size: None 
                //         }
                //     )
                // },

                BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(
                        wgpu::BufferBinding {
                            buffer: &voxel_grid_buffer,
                            offset: 0,
                            size: None
                        }
                    )
                },

                BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(
                        wgpu::BufferBinding {
                            buffer: &raymarch_color_buffer,
                            offset: 0,
                            size: None
                        }
                    )
                }
            ]
        });

        let texture = Texture3D::from_image(&device, &queue, bytemuck::cast_slice(&voxels), dimensions, Some("Voxel 3DTexture")).unwrap();

        let voxel_texture_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("voxel_texture_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture { 
                            sample_type: wgpu::TextureSampleType::Float { filterable: true }, 
                            view_dimension: wgpu::TextureViewDimension::D3, 
                            multisampled: false 
                        },
                        count: None
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None
                    }
                ]
            }
        );

        let voxel_texture_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &voxel_texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&texture.sampler)
                    }
                ],
                label: Some("voxel_texture_bind_group")
            }
        );

        Self {
            voxels,
            dimensions,
            voxels_bind_group_layout: layout,
            // voxels_buffer,
            voxels_bind_group,
            voxel_texture: texture,
            voxel_texture_bind_group_layout,
            voxel_texture_bind_group,
            voxel_grid_buffer,
            raymarch_color_buffer,
            attenuation: 1.0,
            transfer_function_colors: RaymarchTransferFunctionColors::new()
        }
    }

    pub fn set_color(&mut self, position: UVec3, color: [u8; 4]) {
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

        let size = wgpu::Extent3d {
            width: self.dimensions.x,
            height: self.dimensions.y,
            depth_or_array_layers: self.dimensions.z,
        };

        // queue.write_buffer(&self.voxels_buffer, 0, bytemuck::cast_slice(&self.voxels));
        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &self.voxel_texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO
            },
            bytemuck::cast_slice(&self.voxels),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * self.dimensions.x),
                rows_per_image: Some(self.dimensions.y),
            },
            size
        );
    }

    pub fn update_voxel_grid_buffer(&mut self, queue: &Queue) {
        queue.write_buffer(&self.voxel_grid_buffer, 0, bytemuck::cast_slice(&[VoxelGridUniform::new(self.dimensions, self.attenuation)]));
    }

    pub fn update_transfer_function_buffer(&mut self, queue: &Queue) {
        queue.write_buffer(&self.raymarch_color_buffer, 0, bytemuck::cast_slice(&[self.transfer_function_colors]));
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
    pub fn new(dimensions: UVec3, attenuation: f32) -> Self {
        println!("Has dimensions {}", dimensions);
        let min_dimension = u32::min(dimensions.x, u32::min(dimensions.y, dimensions.z));

        let box_size = dimensions.as_vec3() / min_dimension as f32;
        let box_min = -box_size / 2.0;

        Self {
            dimensions: dimensions.xyzx().to_array(),
            box_min: [box_min.x, box_min.y, box_min.z, 0.0],
            box_size: [box_size.x, box_size.y, box_size.z, 0.0],
            buffer: [attenuation; 4],
        }
    }
}

impl RaymarchTransferFunctionColors {
    pub fn new() -> Self {
        Self {
            color_a: [0.117, 0.188, 0.62, 1.0],
            color_b: [0.7294, 0.7294, 0.7294, 1.0],
            color_c: [0.5725, 0.0, 0.039, 1.0],
            use_transfer_function: [0; 4],
        }
    }

    pub fn use_transfer_function_active(&self) -> bool {
        self.use_transfer_function[0] != 0
    }

    pub fn set_transfer_function_active(&mut self, val: bool) {
        self.use_transfer_function[0] = val as u32;
    }
}