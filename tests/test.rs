// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use rayca::{*, png::*};

#[test]
fn simple() {
    let mut image = Image::new(32, 32);
    let scene = Scene::new();
    scene.draw(&mut image);
    dump(&image, "target/reds.png");
}
