// Copyright © 2022-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

#![cfg(target_arch = "wasm32")]

use js_sys::{ArrayBuffer, Uint8Array};
use rayca::Model;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use web_sys::{Request, RequestInit, RequestMode, Response};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn download_model() -> () {
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

    Model::builder()
        .data(&bytes)
        .expect("Failed to load data")
        .build()
        .expect("Failed to build model");
}
