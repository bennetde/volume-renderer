use glam::{DVec3, UVec3};
use noise::{NoiseFn, Perlin};

use crate::voxel::grid::VoxelGrid;

pub fn init_grid_buffer_perlin(grid: &mut VoxelGrid) {
    let noise = Perlin::new(0);

    for x in 0..grid.dimensions.x {
        for y in 0..grid.dimensions.y {
            for z in 0..grid.dimensions.z {
                let mut pos = DVec3::new(x as f64, y as f64, z as f64);
                pos /= 10.0;
                let alpha = (noise.get(pos.to_array()) as f32 + 1.0) / 2.0;
                let alpha = (alpha * 255.0) as u8;
                // println!("{} is {}", UVec3::new(x,y,z), alpha);
                grid.set_color(UVec3::new(x,y,z), [alpha, alpha, alpha, alpha])
            }
        }
    }
}