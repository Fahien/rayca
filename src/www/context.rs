// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::f32::consts::FRAC_PI_8;

use js_sys::{ArrayBuffer, Uint8Array};
use wasm_bindgen::{prelude::*, Clamped, JsCast};
use wasm_bindgen_futures::JsFuture;
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement, ImageData};
use web_sys::{Request, RequestInit, RequestMode, Response};

use crate::*;

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

#[macro_export]
macro_rules! rfmt {
    ( $( $t:tt )* ) => {
        format!($( $t )*)
    }
}

// Wrap web-sys console log function in a println! style macro
#[cfg(target_arch = "wasm32")]
#[macro_export]
macro_rules! rlog {
    ( $( $t:tt )* ) => {
        log(&format!( $( $t )* ))
    }
}

#[wasm_bindgen]
pub struct Context {}

fn get_canvas(id: &str) -> Result<CanvasRenderingContext2d, JsValue> {
    let doc = window().unwrap().document().unwrap();
    let canvas = doc
        .get_element_by_id(id)
        .expect(&format!("Failed to get canvas: {}", id));
    let canvas: HtmlCanvasElement = canvas.dyn_into::<HtmlCanvasElement>()?;
    canvas.set_width(canvas.client_width() as u32);
    canvas.set_height(canvas.client_height() as u32);

    let canvas = canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()?;

    Ok(canvas)
}

async fn get_model() -> Result<Model, JsValue> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

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

    let array = Uint8Array::new(&buffer);
    let bytes = array.to_vec();

    let model = Model::builder()
        .data(&bytes)
        .expect("Failed to load data")
        .build()
        .expect("Failed to build model");

    Ok(model)
}

fn tweak_box_scene(model: &mut Model) {
    let mut blue_material = Material::new();
    blue_material.color = Color::new(0.1, 0.2, 0.7, 1.0);
    let blue_material_handle = model.materials.push(blue_material);

    let mut blue_primitive_handles = vec![];
    let primitive_clones: Vec<_> = model.primitives.iter().cloned().collect();
    for mut blue_primitive in primitive_clones {
        blue_primitive.material = blue_material_handle;
        let blue_primitive_handle = model.primitives.push(blue_primitive);
        blue_primitive_handles.push(blue_primitive_handle);
    }

    let blue_mesh = Mesh::new(blue_primitive_handles);
    let blue_mesh_handle = model.meshes.push(blue_mesh);

    let mut box_node = model.nodes.get(1.into()).unwrap().clone();
    box_node.trs.scale = Vec3::new(16.0, 0.125, 16.0);
    box_node
        .trs
        .translation
        .set_y(box_node.trs.translation.get_y() - 0.75);
    box_node.mesh = blue_mesh_handle;
    box_node.id = model.nodes.len();

    model.nodes.get_mut(0.into()).unwrap().trs.rotation =
        Quat::new(0.0, FRAC_PI_8.sin(), 0.0, FRAC_PI_8.cos());

    model.root.children.push(model.nodes.push(box_node.clone()));
}

#[wasm_bindgen]
pub struct Context {
    canvas: CanvasRenderingContext2d,
    image: Image,
    scene: Scene,
    image_data: ImageData,
    timer: Timer,
}

#[wasm_bindgen]
impl Context {
    pub async fn new() -> Result<Context, JsValue> {
        set_panic_hook();

        let canvas = get_canvas("area")?;

        const WIDTH: u32 = 128;
        let mut image = Image::new(WIDTH, WIDTH, ColorType::RGBA8);
        image.clear(RGBA8::black());

        let mut model = get_model().await.unwrap();
        tweak_box_scene(&mut model);

        let mut scene = Scene::new();
        scene.models.push(model);

        let data = Clamped(image.bytes());

        let image_data = ImageData::new_with_u8_clamped_array(data, WIDTH)?;
        canvas.put_image_data(&image_data, 0.0, 0.0)?;

        Ok(Self {
            canvas,
            image,
            scene,
            image_data,
            timer: Timer::new(),
        })
    }

    pub fn rotate_box(&mut self) {
        let delta = self.timer.get_delta().as_secs_f32();
        let angle = FRAC_PI_8 * delta;
        self.scene.models[0]
            .nodes
            .get_mut(1.into())
            .unwrap()
            .trs
            .rotation *= Quat::new(0.0, angle.sin(), 0.0, angle.cos());
    }

    pub fn draw(&mut self) -> Result<(), JsValue> {
        self.rotate_box();
        self.image.clear(RGBA8::black());
        self.scene.draw(&mut self.image);

        self.canvas.put_image_data(&self.image_data, 0.0, 0.0)?;
        Ok(())
    }
}
