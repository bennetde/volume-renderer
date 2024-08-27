use glam::{DVec3, UVec3, Vec3};

use crate::voxel::grid::VoxelGrid;

pub fn init_grid_buffer_sphere(grid: &mut VoxelGrid, radius: f32) {
    let center = Vec3::new(grid.dimensions.x as f32, grid.dimensions.y as f32, grid.dimensions.z as f32) / 2.0;
    println!("{center}");
    for x in 0..grid.dimensions.x {
        for y in 0..grid.dimensions.y {
            for z in 0..grid.dimensions.z {
                let mut pos = Vec3::new(x as f32, y as f32, z as f32);
                // pos /= 10.0;
                let alpha = if center.distance(pos as Vec3) < radius {
                    255u8
                } else {
                    0u8
                };
                // let alpha = (center.distance(pos as Vec3)) as u8;
                // println!("{alpha}");
                // println!("{} is {}", UVec3::new(x,y,z), alpha);
                grid.set_color(UVec3::new(x,y,z), [alpha, alpha, alpha, alpha])
            }
        }
    }
}