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
            _ => continue
        }
    }

    let raw_file_name = raw_file_name.unwrap();
    let resolution = resolution.unwrap();

    let mut path = Path::new(path);
    path = path.parent().unwrap();
    let path = path.join(raw_file_name);


    let mut raw_file = File::open(path)?;
    let mut byte = [0u8; 1];
    *grid = VoxelGrid::new(resolution, &device, &queue);
    for x in 0..resolution.x {
        for y in 0..resolution.y {
            for z in 0..resolution.z {
                raw_file.read(&mut byte)?;
                grid.set_color(UVec3::new(x,y,z), [byte[0], byte[0], byte[0], byte[0]])
            }
        }
    }

    grid.update_buffer(&queue);
    Ok(())
}