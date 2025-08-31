// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

#[repr(C, align(16))]
#[derive(Default)]
pub struct ComputeCamera {
    transform: Mat4,
    angle: f32,
}

impl ComputeCamera {
    pub fn new(trs: &Trs, angle: f32) -> Self {
        let transform = Mat4::from(trs).get_transpose();
        Self { transform, angle }
    }
}

pub struct ComputeStep {
    pub size: winit::dpi::PhysicalSize<u32>,

    compute_pipeline: wgpu::ComputePipeline,

    compute_storage_buffer: wgpu::Buffer,
    size_buffer: wgpu::Buffer,
    triangle_count_buffer: wgpu::Buffer,
    triangle_buffer: wgpu::Buffer,
    cam_buffer: wgpu::Buffer,

    // 1. Storage, size
    // 2. Camera
    compute_bind_groups: [wgpu::BindGroup; 2],
}

impl ComputeStep {
    pub fn new(device: &wgpu::Device, size: winit::dpi::PhysicalSize<u32>) -> Self {
        // Compute pipeline
        let compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ComputeShader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shader/compute.wgsl").into()),
        });

        let compute_buffer_size = (size.width * size.height * 4) as u64; // RGBA8

        let compute_storage_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("StorageBuffer"),
            size: compute_buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("ComputePipeline"),
            layout: None,
            module: &compute_shader,
            entry_point: Some("render"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        let size_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("SizeBuffer"),
            size: 2 * std::mem::size_of::<u32>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let triangle_count_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("TriangleCountBuffer"),
            size: std::mem::size_of::<u32>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let triangle_max = 8;
        let triangle_buffer_size = std::mem::size_of::<Triangle>() * triangle_max;
        let triangle_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("TriangleBuffer"),
            size: triangle_buffer_size as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let cam_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("CameraBuffer"),
            size: std::mem::size_of::<ComputeCamera>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Instantiates the bind group, once again specifying the binding of buffers.
        let compute_bind_group_layouts = [
            compute_pipeline.get_bind_group_layout(0),
            compute_pipeline.get_bind_group_layout(1),
        ];

        let compute_bind_groups = [
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ComputeBindGroup0"),
                layout: &compute_bind_group_layouts[0],
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: compute_storage_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: size_buffer.as_entire_binding(),
                    },
                ],
            }),
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ComputeBindGroup1"),
                layout: &compute_bind_group_layouts[1],
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: cam_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: triangle_count_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: triangle_buffer.as_entire_binding(),
                    },
                ],
            }),
        ];

        Self {
            size,
            compute_pipeline,
            compute_storage_buffer,
            size_buffer,
            triangle_count_buffer,
            triangle_buffer,
            cam_buffer,

            compute_bind_groups,
        }
    }

    pub fn pass(
        &self,
        queue: &wgpu::Queue,
        camera: &ComputeCamera,
        triangles: &[Triangle],
        encoder: &mut wgpu::CommandEncoder,
        texture: &wgpu::Texture,
    ) {
        // Update size
        let size = [self.size.width, self.size.height];
        let size_slice: &[u8; 8] = unsafe { std::mem::transmute(&size) };
        queue.write_buffer(&self.size_buffer, 0, size_slice);

        // Update triangle count
        let triangle_count = triangles.len() as u32;
        let triangle_count_slice = unsafe {
            std::slice::from_raw_parts::<u8>(
                &triangle_count as *const u32 as _,
                std::mem::size_of::<u32>(),
            )
        };
        queue.write_buffer(&self.triangle_count_buffer, 0, triangle_count_slice);

        // Update triangle
        let triangle_slice = unsafe {
            std::slice::from_raw_parts::<u8>(
                triangles.as_ptr() as _,
                std::mem::size_of_val(triangles),
            )
        };
        queue.write_buffer(&self.triangle_buffer, 0, triangle_slice);

        // Update camera
        let cam_slice = unsafe {
            std::mem::transmute::<&ComputeCamera, &[u8; std::mem::size_of::<ComputeCamera>()]>(
                camera,
            )
        };
        queue.write_buffer(&self.cam_buffer, 0, cam_slice);

        // Record commands for compute pass
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.compute_pipeline);
            cpass.set_bind_group(0, &self.compute_bind_groups[0], &[]);
            cpass.set_bind_group(1, &self.compute_bind_groups[1], &[]);
            cpass.insert_debug_marker("Compute Debug Marker");
            cpass.dispatch_workgroups(self.size.width, self.size.height, 1);
        }

        // Copy compute storage buffer to texture
        encoder.copy_buffer_to_texture(
            wgpu::TexelCopyBufferInfo {
                buffer: &self.compute_storage_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * self.size.width),
                    rows_per_image: Some(self.size.height),
                },
            },
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: self.size.width,
                height: self.size.height,
                depth_or_array_layers: 1,
            },
        );
    }
}
