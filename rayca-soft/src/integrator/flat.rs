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
    fn trace(
        &self,
        _config: &Config,
        scene: &SceneDrawInfo,
        tlas: &Tlas,
        ray: Ray,
        _depth: u32,
    ) -> Option<Color> {
        let mut hit = tlas.intersects(scene, ray)?;
        let color = hit.get_color();
        Some(color)
    }
}
