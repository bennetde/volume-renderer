use crate::voxel::grid::VoxelGrid;
use anyhow::Result;
use glam::UVec3;


pub fn compare_to_netcdf_rmse(path: &str, grid: &mut VoxelGrid, density_only: bool) -> Result<f32> {
    let file = netcdf::open(path)?;

    let var = &file.variable("color").expect("Could not find 'color' variable");

    let x = &file.dimension("x").unwrap().len();
    let y = &file.dimension("y").unwrap().len();
    let z = &file.dimension("z").unwrap().len();

    let mut rmse: f32 = 0.0;

    let data= var.get::<f32, _>((..,..,..,..))?;
    for x in 0..grid.dimensions.x {
        for y in 0..grid.dimensions.y {
            for z in 0..grid.dimensions.z {
                let pos = UVec3::new(x,y,z);
                let ground_truth = grid[pos].color;

                let data_0 = data[[0,z as usize,y as usize,x as usize]] * 255.0;
                let data_1 = data[[1,z as usize,y as usize,x as usize]] * 255.0;
                let data_2 = data[[2,z as usize,y as usize,x as usize]] * 255.0;
                let data_3 = data[[3,z as usize,y as usize,x as usize]] * 255.0;

                rmse += f32::abs(ground_truth[3] as f32 - data_3);

                if !density_only {
                    rmse += f32::abs(ground_truth[0] as f32 - data_0);
                    rmse += f32::abs(ground_truth[1] as f32 - data_1);
                    rmse += f32::abs(ground_truth[2] as f32 - data_2);
                }
            }
        }
    }
    

    println!("Finished comparing NetCDF Model | Density only: {}", density_only);
    Ok(rmse / ((x * y * z) as f32))
}