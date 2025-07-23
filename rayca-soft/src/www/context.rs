// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::f32::consts::FRAC_PI_8;

use js_sys::{ArrayBuffer, Uint8Array};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::JsFuture;
use web_sys::OffscreenCanvas;
use web_sys::WebGl2RenderingContext;
use web_sys::{Request, RequestInit, RequestMode, Response};
use web_sys::{WebGl2RenderingContext as gl, WebGlProgram, WebGlShader};

use crate::*;

pub use wasm_bindgen_rayon::init_thread_pool;

pub fn set_panic_hook() {
    console_error_panic_hook::set_once();
}

fn get_canvas(canvas: OffscreenCanvas) -> Result<WebGl2RenderingContext, JsValue> {
    let canvas = canvas.get_context("webgl2");
    let canvas = canvas.unwrap();
    let rendering = canvas.unwrap();
    let canvas = rendering.dyn_into::<WebGl2RenderingContext>()?;
    Ok(canvas)
}

fn set_canvas_size(canvas: &OffscreenCanvas, width: u32, height: u32) {
    canvas.set_width(width);
    canvas.set_height(height);
}

async fn _get_model_from_internet() -> Result<Model, JsValue> {
    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);

    let url = "https://raw.githubusercontent.com/KhronosGroup/glTF-Sample-Models/master/2.0/Box/glTF-Embedded/Box.gltf";

    let request = Request::new_with_str_and_init(&url, &opts).expect("Failed to request");

    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .unwrap();

    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().unwrap();

    let buffer = JsFuture::from(resp.array_buffer().expect("Failed to get array buffer"))
        .await
        .expect("Failed to wait for array buffer");

    assert!(buffer.is_instance_of::<ArrayBuffer>());
    let buffer: ArrayBuffer = buffer.dyn_into().unwrap();
    get_model_from_buffer(buffer).await
}

async fn get_model_from_buffer(buffer: ArrayBuffer) -> Result<Model, JsValue> {
    let array = Uint8Array::new(&buffer);
    let bytes = array.to_vec();
    let assets = Assets::new();
    let model = Model::load_gltf_data(&bytes, &assets).expect("Failed to build model");
    Ok(model)
}

fn add_camera(model: &mut Model) {
    model.root.children.push(
        model.nodes.push(
            Node::builder()
                .camera(model.cameras.push(Camera::default()))
                .trs(Trs::builder().translation(Vec3::new(0.0, 0.0, 8.0)).build())
                .build(),
        ),
    );
}

fn tweak_box_scene(model: &mut Model) {
    let blue_material = Material::builder()
        .color(Color::new(0.1, 0.2, 0.7, 1.0))
        .build();
    let blue_material_handle = model.materials.push(blue_material);

    let mut blue_primitive_handles = vec![];
    let primitive_clones: Vec<_> = model.primitives.iter().cloned().collect();
    for mut blue_primitive in primitive_clones {
        blue_primitive.material = blue_material_handle;
        let blue_primitive_handle = model.primitives.push(blue_primitive);
        blue_primitive_handles.push(blue_primitive_handle);
    }

    let blue_mesh = Mesh::builder().primitives(blue_primitive_handles).build();
    let blue_mesh_handle = model.meshes.push(blue_mesh);

    let mut box_node = model.nodes.get(1.into()).unwrap().clone();
    box_node.trs.scale = Vec3::new(16.0, 0.125, 16.0);
    box_node
        .trs
        .translation
        .set_y(box_node.trs.translation.get_y() - 0.75);
    box_node.mesh.replace(blue_mesh_handle);

    model.nodes.get_mut(0.into()).unwrap().trs.rotation =
        Quat::new(0.0, FRAC_PI_8.sin(), 0.0, FRAC_PI_8.cos());

    model.root.children.push(model.nodes.push(box_node.clone()));
}

#[wasm_bindgen]
pub struct Context {
    width: u32,
    height: u32,
    canvas: WebGl2RenderingContext,
    image: Image,
    renderer: SoftRenderer,
    scene: Scene,
    timer: Timer,
}

#[wasm_bindgen]
impl Context {
    pub async fn new(
        offscreen_canvas: OffscreenCanvas,
        mut width: u32,
        mut height: u32,
        buffer: ArrayBuffer,
    ) -> Result<Context, JsValue> {
        set_panic_hook();
        logging::init();

        width = 256;
        height = 256;
        set_canvas_size(&offscreen_canvas, width, height);
        let canvas = get_canvas(offscreen_canvas)?;

        let mut image = Image::new(width, height, ColorType::RGBA8);
        image.clear(RGBA8::black());

        let renderer = SoftRenderer::default();

        let mut model = get_model_from_buffer(buffer).await.unwrap();
        add_camera(&mut model);
        tweak_box_scene(&mut model);

        let mut scene = Scene::default();
        scene.push_model(model);
        scene.push_model(SoftRenderer::create_default_model());

        Ok(Self {
            width,
            height,
            canvas,
            image,
            renderer,
            scene,
            timer: Timer::new(),
        })
    }

    pub fn rotate_box(&mut self) {
        let delta = self.timer.get_delta().as_secs_f32();
        let angle = FRAC_PI_8 * delta;
        self.scene.nodes.get_mut(0.into()).unwrap().trs.rotation *=
            Quat::new(0.0, angle.sin(), 0.0, angle.cos());
    }

    fn create_shader(&self, shader_type: u32, source: &str) -> Option<WebGlShader> {
        let shader = self.canvas.create_shader(shader_type);
        self.canvas.shader_source(shader.as_ref().unwrap(), source);
        self.canvas.compile_shader(shader.as_ref().unwrap());
        shader
    }

    fn create_program(&self, vs: &str, fs: &str) -> Option<WebGlProgram> {
        let program = self.canvas.create_program();
        let vs = self.create_shader(gl::VERTEX_SHADER, vs);
        let fs = self.create_shader(gl::FRAGMENT_SHADER, fs);

        self.canvas
            .attach_shader(program.as_ref().unwrap(), vs.as_ref().unwrap());
        self.canvas.delete_shader(vs.as_ref());
        self.canvas
            .attach_shader(program.as_ref().unwrap(), fs.as_ref().unwrap());
        self.canvas.delete_shader(fs.as_ref());
        self.canvas.link_program(program.as_ref().unwrap());

        let log = self.canvas.get_program_info_log(program.as_ref().unwrap());
        if let Some(log) = log {
            if !log.is_empty() {
                log::info!("Link program: {}", log);
            }
        }

        let log = self.canvas.get_shader_info_log(vs.as_ref().unwrap());
        if let Some(log) = log {
            if !log.is_empty() {
                log::info!("Vertex Shader: {}", log);
            }
        }

        let log = self.canvas.get_shader_info_log(fs.as_ref().unwrap());
        if let Some(log) = log {
            if !log.is_empty() {
                log::info!("Fragment Shader: {}", log);
            }
        }

        program
    }

    pub fn draw(&mut self) -> Result<(), JsValue> {
        const VS: &str = r#"#version 300 es
        #define POSITION_LOCATION 0
        #define TEXCOORD_LOCATION 4
        
        precision highp float;
        precision highp int;

        layout(location = POSITION_LOCATION) in vec2 position;
        layout(location = TEXCOORD_LOCATION) in vec2 texcoord;

        out vec2 v_st;

        void main()
        {
            v_st = texcoord;
            gl_Position = vec4(position, 0.0, 1.0);
        }
        "#;

        const FS: &str = r#"#version 300 es
        precision highp float;
        precision highp int;

        uniform sampler2D diffuse;

        in vec2 v_st;

        out vec4 color;

        void main()
        {
            color = texture(diffuse, v_st);
        }
        "#;

        self.rotate_box();
        self.image.clear(RGBA8::BLACK);

        self.renderer.draw(&self.scene, &mut self.image);

        let program = self.create_program(VS, FS);
        let diffuse_location = self
            .canvas
            .get_uniform_location(program.as_ref().unwrap(), "diffuse");

        // Vertex buffers
        let positions: [f32; 12] = [
            -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, 1.0, 1.0, -1.0, 1.0, -1.0, -1.0,
        ];
        let positions: &[u8; 12 * std::mem::size_of::<f32>()] =
            unsafe { std::mem::transmute(&positions) };
        let vertex_pos_buffer = self.canvas.create_buffer();
        self.canvas
            .bind_buffer(gl::ARRAY_BUFFER, vertex_pos_buffer.as_ref());
        self.canvas
            .buffer_data_with_u8_array(gl::ARRAY_BUFFER, positions, gl::STATIC_DRAW);
        self.canvas.bind_buffer(gl::ARRAY_BUFFER, None);

        let tex_coords: [f32; 12] = [0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0];
        let tex_coords: &[u8; 12 * std::mem::size_of::<f32>()] =
            unsafe { std::mem::transmute(&tex_coords) };
        let vertex_tex_buffer = self.canvas.create_buffer();
        self.canvas
            .bind_buffer(gl::ARRAY_BUFFER, vertex_tex_buffer.as_ref());
        self.canvas
            .buffer_data_with_u8_array(gl::ARRAY_BUFFER, tex_coords, gl::STATIC_DRAW);
        self.canvas.bind_buffer(gl::ARRAY_BUFFER, None);

        // Vertex array
        let vertex_array = self.canvas.create_vertex_array();
        self.canvas.bind_vertex_array(vertex_array.as_ref());

        let vertex_pos_location = 0; // set with GLSL layout qualifier
        self.canvas.enable_vertex_attrib_array(vertex_pos_location);
        self.canvas
            .bind_buffer(gl::ARRAY_BUFFER, vertex_pos_buffer.as_ref());
        self.canvas
            .vertex_attrib_pointer_with_i32(vertex_pos_location, 2, gl::FLOAT, false, 0, 0);
        self.canvas.bind_buffer(gl::ARRAY_BUFFER, None);

        let vertex_tex_location = 4; // set with GLSL layout qualifier
        self.canvas.enable_vertex_attrib_array(vertex_tex_location);
        self.canvas
            .bind_buffer(gl::ARRAY_BUFFER, vertex_tex_buffer.as_ref());
        self.canvas
            .vertex_attrib_pointer_with_i32(vertex_tex_location, 2, gl::FLOAT, false, 0, 0);
        self.canvas.bind_buffer(gl::ARRAY_BUFFER, None);

        self.canvas.bind_vertex_array(None);

        // Texture
        let texture = self.canvas.create_texture();
        self.canvas.bind_texture(gl::TEXTURE_2D, texture.as_ref());
        //self.canvas.pixel_storei(gl::UNPACK_FLIP_Y_WEBGL, 1);
        self.canvas
            .tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        self.canvas
            .tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        self.canvas
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_u8_array_and_src_offset(
                gl::TEXTURE_2D,
                0,
                gl::RGBA8 as i32,
                self.width as i32,
                self.height as i32,
                0, // border
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                self.image.bytes(),
                0,
            )
            .expect("Failed to upload data to texture");

        // Render
        self.canvas.use_program(program.as_ref());
        self.canvas.uniform1i(diffuse_location.as_ref(), 0);

        // Pass
        self.canvas.bind_framebuffer(gl::FRAMEBUFFER, None);
        self.canvas
            .viewport(0, 0, (self.width) as i32, (self.height) as i32);
        self.canvas
            .clear_bufferfv_with_f32_array(gl::COLOR, 0, &[0.3, 0.3, 0.3, 1.0]);

        // Render
        self.canvas.active_texture(gl::TEXTURE0);
        self.canvas.bind_texture(gl::TEXTURE_2D, texture.as_ref());
        self.canvas.bind_vertex_array(vertex_array.as_ref());
        self.canvas.draw_arrays(gl::TRIANGLES, 0, 6);
        self.canvas.bind_texture(gl::TEXTURE_2D, None);

        Ok(())
    }
}
