// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use super::*;

#[derive(Default)]
pub struct Scene {}

impl Scene {
    pub fn new() -> Self {
        Self {}
    }
}

impl Draw for Scene {
    fn draw(&self, image: &mut Image) {
        for y in 0..image.height() {
            for x in 0..image.width() {
                // generate primary ray
                //for obj in scene.objects {
                //    let hit = ray.intersect(obj);
                //}
                let r_channel = (x as f32 / image.width() as f32 * 255.0) as u32;
                image.set(x, y, 0xFF000000 | r_channel);
            }
        }
    }
}
