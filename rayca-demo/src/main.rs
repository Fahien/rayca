// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

mod ctx;
use ctx::*;

use rayca_soft::*;

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop
        .run_app(&mut app)
        .expect("Failed to run application");
}

#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    ctx: Option<Ctx>,
    image: Option<Image>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .expect("Failed to create window"),
        );

        let ctx = pollster::block_on(Ctx::new(window.clone()));

        let mut image = Image::new(ctx.size.width, ctx.size.height, ColorType::RGBA8);
        run_test(&mut image);

        self.window = Some(window);
        self.ctx = Some(ctx);
        self.image = Some(image);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                let ctx = self.ctx.as_mut().unwrap();
                ctx.resize(physical_size);
            }
            WindowEvent::RedrawRequested => {
                let ctx = self.ctx.as_mut().unwrap();
                let image = self.image.as_ref().unwrap();

                ctx.update(image.bytes());
                match ctx.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => ctx.resize(ctx.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }

                // Queue a RedrawRequested event.
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
    }
}

fn run_test(image: &mut Image) {
    image.fill(RGBA8::black());

    let mut scene = Scene::default();
    let assets = Assets::new();
    scene
        .push_gltf_from_path(tests::get_model_path().join("box/box.gltf"), &assets)
        .expect("Failed to push glTF model to scene");
    scene.push_model(SoftRenderer::create_default_model());

    let mut renderer = rayca_soft::SoftRenderer::default();
    renderer.draw(&scene, image);
}
