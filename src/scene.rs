// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use super::*;

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
                image.set(x, y, 0, (x as f32 / image.width() as f32 * 255.0) as u8);
                image.set(x, y, 3, 0xFFu8);
            }
        }
    }
}
