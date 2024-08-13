use anyhow::Result;
use glam::UVec3;

use crate::voxel::{grid::VoxelGrid, voxel::Voxel};

pub fn write_voxel_grid(path: &str, grid: &VoxelGrid) -> Result<()> {

    let mut file = netcdf::create(path)?;
    file.add_dimension("c", 4)?;
    file.add_dimension("z", grid.dimensions.z as usize)?;
    file.add_dimension("y", grid.dimensions.y as usize)?;
    file.add_dimension("x", grid.dimensions.x as usize)?;

    let mut color = file.add_variable::<f32>("color", &["c", "z", "y", "x"])?;

    for x in 0..grid.dimensions.x as usize {
        for y in 0..grid.dimensions.y as usize {
            for z in 0..grid.dimensions.z as usize {
                let voxel = grid[UVec3::new(x as u32,y as u32,z as u32)];
                for channel in 0..4  as usize{
                    let val = voxel.color[channel as usize] as f32 / 255.0;
                    let extents: netcdf::Extents = [channel..channel+1, z..z+1,y..y+1,x..x+1].try_into().unwrap();
                    color.put_value::<f32, _>(val, extents)?;
                }
            }
        }
    }
    Ok(())
}