use std::{fs::{File, OpenOptions}, io::BufWriter};

use image::{flat::SampleLayout, imageops::thumbnail, FlatSamples, Rgba, SubImage};
use png::chunk::{ChunkType, IDAT};
use wgpu::{CommandBuffer, CommandEncoderDescriptor, Device, Queue, SurfaceConfiguration, SurfaceTexture};

pub struct Screenshotter {
    buffer: wgpu::Buffer
}


impl Screenshotter {
    pub fn new(device: &Device, config: &SurfaceConfiguration) -> Self {
        let buffer_size = (std::mem::size_of::<u32>() as u32 * 832 * config.height) as wgpu::BufferAddress;

        let buffer_descriptor = wgpu::BufferDescriptor {
            size: buffer_size,
            label: Some("screenshot_buffer"),
            mapped_at_creation: false,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ
        };
        let buffer = device.create_buffer(&buffer_descriptor);

        Self {
            buffer
        }
    }

    pub fn screenshot(&self, surface: &SurfaceTexture, config: &SurfaceConfiguration, device: &Device, queue: &Queue) -> CommandBuffer {
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: Some("screenshot_command_encoder") });

        let texture = &surface.texture;
        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO
            },
            wgpu::ImageCopyBuffer {
                buffer: &self.buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(3328),
                    rows_per_image: Some(config.height),
                }
            },
            texture.size()
        );

        encoder.finish()
    }

    pub async fn save_screenshot_to_disk(&self, device: &Device, config: &SurfaceConfiguration, file_name: &str) {
        let buffer_slice = self.buffer.slice(..);
        
        let (tx, rx) = flume::bounded(1);
        buffer_slice.map_async(wgpu::MapMode::Read, move |r| tx.send(r).unwrap());
        device.poll(wgpu::Maintain::wait()).panic_on_timeout();
        rx.recv_async().await.unwrap().unwrap();
        {
            let data = buffer_slice.get_mapped_range();

            // Create image layout and set the height_stride to the actual with of the image including its padding in bytes
            // This means if the image has a width of 800 pixels => 800 pixels * 4 bytes (rgba) = 3200 pixels
            // Because WGPU requires a 256 byte row alignment the image would have a padding of 3328 bytes - 3200 bytes = 128 bytes => 32 pixel
            // We set the height_stride to 3328 bytes
            let mut sample_layout = SampleLayout::row_major_packed(4, 800, config.height);
            sample_layout.height_stride = 3328;

            // Create a flat sample as well as a view 
            let image_buffer = FlatSamples {
                color_hint: None,
                samples: data,
                layout: sample_layout
            };
            let view = match image_buffer.as_view::<Rgba<u8>>() {
                Err(_) => panic!("Invalid image format"),
                Ok(view) => view
            };

            // TODO: Find a better way to save the image.
            thumbnail(&view, config.width, config.height).save(file_name).unwrap();
            println!("Saved screenshot to file");
        }

        // Unmap buffer so it can be used again for the next screenshot
        self.buffer.unmap();
    }
}