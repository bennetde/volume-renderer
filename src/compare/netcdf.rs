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
    let mut amount_channels = 1;
    let data= var.get::<f32, _>((..,..,..,..))?;
    for x in 0..grid.dimensions.x {
        for y in 0..grid.dimensions.y {
            for z in 0..grid.dimensions.z {
                let pos = UVec3::new(x,y,z);
                let ground_truth = grid[pos].color;

                let data_0 = data[[0,z as usize,y as usize,x as usize]];
                let data_1 = data[[1,z as usize,y as usize,x as usize]];
                let data_2 = data[[2,z as usize,y as usize,x as usize]];
                let data_3 = data[[3,z as usize,y as usize,x as usize]];

                let alpha = (ground_truth[3] as f32) / 255.0;
                rmse += f32::powi(alpha - data_3, 2);
                if !density_only {
                    let tf_0r;
                    let tf_0g;
                    let tf_0b;
                    let tf_1r;
                    let tf_1g;
                    let tf_1b;
                    let alpha_remapped;
                    if alpha < 0.5 {
                       tf_0r = grid.transfer_function_colors.color_a[0];
                       tf_0g = grid.transfer_function_colors.color_a[1];
                       tf_0b = grid.transfer_function_colors.color_a[2];
                       tf_1r = grid.transfer_function_colors.color_b[0];
                       tf_1g = grid.transfer_function_colors.color_b[1];
                       tf_1b = grid.transfer_function_colors.color_b[2];
                       alpha_remapped = remap(alpha, 0.0, 0.5, 0.0, 1.0);
                    } else {
                        tf_0r = grid.transfer_function_colors.color_b[0];
                        tf_0g = grid.transfer_function_colors.color_b[1];
                        tf_0b = grid.transfer_function_colors.color_b[2];
                        tf_1r = grid.transfer_function_colors.color_c[0];
                        tf_1g = grid.transfer_function_colors.color_c[1];
                        tf_1b = grid.transfer_function_colors.color_c[2];
                        alpha_remapped = remap(alpha, 0.5, 1.0, 0.0, 1.0);
                    }

                    let r = mix(tf_0r, tf_1r, alpha_remapped);
                    let g = mix(tf_0g, tf_1g, alpha_remapped);
                    let b = mix(tf_0b, tf_1b, alpha_remapped);
                    rmse += f32::powi(r - data_0,2);
                    rmse += f32::powi(g - data_1,2);
                    rmse += f32::powi(b - data_2,2);
                    amount_channels = 4;
                }
            }
        }
    }
    

    println!("Finished comparing NetCDF Model | Density only: {}", density_only);
    let mean = rmse / (x * y * z * amount_channels) as f32;
    Ok(f32::sqrt(mean))
}

fn mix(x: f32, y: f32, a: f32) -> f32 {
    x * (1.0 - a) + y * a
}

fn remap(value: f32, min1: f32, max1: f32, min2: f32, max2: f32) -> f32 {
    min2 + (value - min1) * (max2 - min2) / (max1 - min1)
}