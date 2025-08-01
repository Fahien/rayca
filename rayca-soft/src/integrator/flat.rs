// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

#[derive(Default)]
pub struct Flat {}

impl Flat {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Integrator for Flat {
    fn trace(&self, scene: &SceneDrawInfo, ray: Ray, tlas: &Tlas, _depth: u32) -> Option<Color> {
        let hit = tlas.intersects(scene, &ray)?;
        let blas = tlas.get_blas(hit.blas);
        let primitive = blas.model.get_primitive(hit.primitive);
        Some(primitive.get_color(scene, &hit))
    }
}
