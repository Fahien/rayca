// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use winit::window::Window;

use crate::*;

pub struct Ctx {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,

    compute_step: ComputeStep,
    render_step: RenderStep,
}

impl Ctx {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::from_env_or_default());
        let surface = instance
            .create_surface(window)
            .expect("Failed to create surface");
        Self::new_with_surface(instance, surface, size).await
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn new_with_offscreen_canvas(offscreen_canvas: web_sys::OffscreenCanvas) -> Self {
        let size =
            winit::dpi::PhysicalSize::new(offscreen_canvas.width(), offscreen_canvas.height());
        rlog!("Size: {:?}", size);

        // The instance is a handle to our GPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::BROWSER_WEBGPU,
            ..Default::default()
        });
        let surface = instance
            .create_surface_from_offscreen_canvas(offscreen_canvas)
            .unwrap();
        Self::new_with_surface(instance, surface, size).await
    }

    pub async fn new_with_surface(
        instance: wgpu::Instance,
        surface: wgpu::Surface<'static>,
        size: winit::dpi::PhysicalSize<u32>,
    ) -> Self {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .unwrap();

        device.push_error_scope(wgpu::ErrorFilter::Validation);
        device.on_uncaptured_error(Box::new(|error| eprintln!("{}", error)));

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let compute_step = ComputeStep::new(&device, size);
        let render_step = RenderStep::new(&device, size, &config);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            compute_step,
            render_step,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn render(
        &mut self,
        camera: &ComputeCamera,
        triangle: &Triangle,
    ) -> Result<(), wgpu::SurfaceError> {
        // A command encoder executes one or many pipelines.
        // It is to WebGPU what a command buffer is to Vulkan.
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // We can not render directly onto the surface
        self.compute_step.pass(
            &self.queue,
            camera,
            triangle,
            &mut encoder,
            &self.render_step.texture,
        );

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.render_step.pass(&mut encoder, view);

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
