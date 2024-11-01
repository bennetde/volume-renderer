
use glam::{DVec3, UVec3};
use noise::{NoiseFn, Perlin};

use crate::voxel::grid::VoxelGrid;

/// Initiates the VoxelGrid to contain a Perlin Noise Model
#[allow(dead_code)]
pub fn init_grid_buffer_perlin(grid: &mut VoxelGrid) {
    //std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32
    let noise = Perlin::new(51);

    for x in 0..grid.dimensions.x {
        for y in 0..grid.dimensions.y {
            for z in 0..grid.dimensions.z {
                let mut pos = DVec3::new(x as f64, y as f64, z as f64);
                pos /= 100.0;
                let alpha = (noise.get(pos.to_array()) as f32 + 1.0) / 2.0;
                let mut alpha = (alpha * 255.0) as u8;
                if alpha < 128 {
                    alpha = 0;
                } else {
                    alpha = 255;
                }
                // let alpha = 255;
                // println!("{} is {}", UVec3::new(x,y,z), alpha);
                grid.set_color(UVec3::new(x,y,z), [alpha, alpha, alpha, alpha])
            }
        }
    }
}