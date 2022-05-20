// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use js_sys::{ArrayBuffer, Uint8Array};
use wasm_bindgen::{prelude::*, Clamped, JsCast};
use wasm_bindgen_futures::JsFuture;
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement, ImageData};
use web_sys::{Request, RequestInit, RequestMode, Response};

use super::*;

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

    let url = "https://raw.githubusercontent.com/KhronosGroup/glTF-Sample-Models/master/2.0/Duck/glTF-Embedded/Duck.gltf";

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

#[wasm_bindgen]
impl Context {
    pub fn new() -> Self {
        set_panic_hook();
        Self {}
    }

    pub async fn draw() -> Result<(), JsValue> {
        let canvas = get_canvas("area")?;

        let width = 512;
        let mut image = Image::new(width, width, ColorType::RGBA8);

        let mut model = get_model().await.unwrap();

        // Custom camera
        let mut camera_node = Node::builder()
            .id(model.nodes.len())
            .translation(Vec3::new(0.1, 0.8, 2.2))
            .build();
        camera_node.camera = Some(model.cameras.push(Camera::default()));
        let camera_node_handle = model.nodes.push(camera_node);
        model.root.children.push(camera_node_handle);

        let mut scene = Scene::new();
        scene.models.push(model);
        scene.draw(&mut image);

        let data = Clamped(image.bytes());

        let image_data = ImageData::new_with_u8_clamped_array(data, width)?;
        canvas.put_image_data(&image_data, 0.0, 0.0)?;

        Ok(())
    }
}
