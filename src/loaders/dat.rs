use std::{fs::File, io::{BufRead, BufReader, Read}, path::Path};

use glam::UVec3;
use wgpu::{Device, Queue};

use crate::voxel::grid::VoxelGrid;
use anyhow::Result;

pub fn open_voxel_grid(path: &str, grid: &mut VoxelGrid, device: &Device, queue: &Queue) -> Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut raw_file_name: Option<String> = None;
    let mut resolution: Option<UVec3> = None;
    let mut bytes_to_read = 1usize;
    for line in reader.lines() {
        let line = line?;
        let collection =  line.split(":").collect::<Vec<&str>>();
        let value = collection[1].trim();
        let key = collection[0];

        drop(collection);
        match key {
            "ObjectFileName" => {
                raw_file_name = Some(value.to_string());
            },
            "Resolution" => {
                let sizes = value.split(" ").collect::<Vec<&str>>();
                resolution = Some(UVec3::new(sizes[0].parse::<u32>()?, sizes[1].parse::<u32>()?, sizes[2].parse::<u32>()?));
            },
            "Format" => {
                println!("{}", value);
                if value.eq("USHORT") {
                    bytes_to_read = 2;
                }
            },
            _ => continue
        }
    }

    let raw_file_name = raw_file_name.unwrap();
    let resolution = resolution.unwrap();

    let mut path = Path::new(path);
    path = path.parent().unwrap();
    let path = path.join(raw_file_name);


    let mut raw_file = File::open(path)?;
    let mut bytes = vec![0; bytes_to_read];
    *grid = VoxelGrid::new(resolution, &device, &queue);
    for z in 0..resolution.z {
        for y in 0..resolution.y {
            for x in 0..resolution.x {
                raw_file.read(&mut bytes)?;
                 
                let mut val: usize = 0;
                for (i, byte) in (&bytes).into_iter().enumerate() {
                    val |= *byte as usize;
                    if i < bytes_to_read - 1 {
                    val <<= 8;
                    }
                }

                let byte: u8 = ((val as f32 / (usize::pow(2, bytes_to_read as u32 * 8) - 1) as f32) * 255.0) as u8;
                // println!("Read {} vs {}", val, byte);
                grid.set_color(UVec3::new(x,y,z), [byte, byte, byte, byte])
            }
        }
        println!("Loaded {} out of {} layers", z, resolution.z);
    }
    grid.update_buffer(&queue);
    Ok(())
}