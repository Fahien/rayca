// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, SmolStr},
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
    scene: Scene,
    image: Option<Image>,
    renderer: SoftRenderer,
}

impl App {
    fn soft_render(&mut self) {
        let image = self.image.as_mut().unwrap();
        image.fill(RGBA8::black());
        self.renderer.draw(&self.scene, image);
    }

    fn update_move(&mut self, key: SmolStr) {
        let mut rot = 0.0;
        let mut x = 0.0;
        let mut z = 0.0;

        match key.as_str() {
            "w" => z = -0.25,
            "s" => z = 0.25,
            "e" => x = 0.25,
            "q" => x = -0.25,
            "a" => rot = 0.125,
            "d" => rot = -0.125,
            _ => (),
        }

        let camera_node = self.scene.get_last_camera_node_mut().unwrap();
        camera_node.trs.translate(Vec3::new(x, 0.0, z));
        camera_node.trs.rotate(Quat::axis_angle(Vec3::Y_AXIS, rot));
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .expect("Failed to create window"),
        );

        let ctx = pollster::block_on(Ctx::new(window.clone()));

        let assets = Assets::new();
        self.scene
            .push_gltf_from_path(tests::get_model_path().join("box/box.gltf"), &assets)
            .expect("Failed to push glTF model to scene");
        self.scene.push_model(SoftRenderer::create_default_model());

        let image = Image::new(ctx.size.width, ctx.size.height, ColorType::RGBA8);

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
                self.soft_render();

                let ctx = self.ctx.as_mut().unwrap();

                ctx.update(self.image.as_ref().unwrap().bytes());
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
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Character(key),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                self.update_move(key);
            }
            _ => (),
        }
    }
}
