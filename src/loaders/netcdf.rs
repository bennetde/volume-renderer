use anyhow::Result;
use glam::UVec3;
use wgpu::{Device, Queue};

use crate::voxel::grid::VoxelGrid;

/// Exports a Voxel Grid into a NetCDF-File
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

/// Loads a NetCDF-File into a Voxel Grid
pub fn open_voxel_grid(path: &str, grid: &mut VoxelGrid, device: &Device, queue: &Queue) -> Result<()> {
    let file = netcdf::open(path)?;

    let var = &file.variable("color").expect("Could not find 'color' variable");

    let x = &file.dimension("x").unwrap().len();
    let y = &file.dimension("y").unwrap().len();
    let z = &file.dimension("z").unwrap().len();
    
    println!("{} {} {}", x, y, z);

    let data= var.get::<f32, _>((..,..,..,..))?;

    *grid = VoxelGrid::new(UVec3::new(*x as u32,*y as u32,*z as u32), device, queue);

    for x in 0..grid.dimensions.x {
        for y in 0..grid.dimensions.y {
            for z in 0..grid.dimensions.z {


                let data_0 = (data[[0,z as usize,y as usize,x as usize]] * 255.0) as u8;
                let data_1 = (data[[1,z as usize,y as usize,x as usize]] * 255.0) as u8;
                let data_2 = (data[[2,z as usize,y as usize,x as usize]] * 255.0) as u8;
                let data_3 = (data[[3,z as usize,y as usize,x as usize]] * 255.0) as u8;

                grid.set_color(UVec3::new(x,y,z), [data_0, data_1, data_2, data_3])
            }
        }
    }



    grid.update_buffer(&queue);
    println!("Finished loading NetCDF Model");
    Ok(())
}