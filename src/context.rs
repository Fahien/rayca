// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use wasm_bindgen::{prelude::*, Clamped, JsCast};
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

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

#[wasm_bindgen]
impl Context {
    pub fn new() -> Self {
        set_panic_hook();
        Self {}
    }

    pub fn draw() -> Result<(), JsValue> {
        let canvas = get_canvas("area")?;

        let width = 256;
        let mut image = Image::new(width, width, ColorType::RGBA8);
        let mut scene = Scene::new();

        let mut model = Model::new();
        let prim = Primitive::unit_triangle();
        let prim_handle = model.primitives.push(prim);
        let mesh = Mesh::new(vec![prim_handle]);
        let mesh_handle = model.meshes.push(mesh);
        let node = Node::builder().mesh(mesh_handle).build();
        let node_handle = model.nodes.push(node);
        model.root.children.push(node_handle);

        scene.models.push(model);
        scene.draw(&mut image);

        let data = Clamped(image.bytes());

        let image_data = ImageData::new_with_u8_clamped_array(data, width)?;
        canvas.put_image_data(&image_data, 0.0, 0.0)?;

        Ok(())
    }
}
