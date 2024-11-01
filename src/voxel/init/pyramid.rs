use glam::{UVec3, Vec3};

use crate::voxel::grid::VoxelGrid;

/// Initiates the VoxelGrid to contain a Pyramid Model
#[allow(dead_code)]
pub fn init_grid_buffer_pyramid(grid: &mut VoxelGrid) {
    let center = Vec3::new(grid.dimensions.x as f32, grid.dimensions.y as f32, grid.dimensions.z as f32) / 2.0;
    println!("{center}");
    for y in 0..grid.dimensions.y {
        for x in y..grid.dimensions.x-y {
            for z in y..grid.dimensions.y-y {
                // let pos = Vec3::new(x as f32, y as f32, z as f32);
                let alpha = 255u8;
                // let alpha = (center.distance(pos as Vec3)) as u8;
                // println!("{alpha}");
                // println!("{} is {}", UVec3::new(x,y,z), alpha);
                grid.set_color(UVec3::new(x,y,z), [alpha, alpha, alpha, alpha])
            }
        }
    }
}