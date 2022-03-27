// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use super::*;

pub struct Material {
    pub color: Color,
    pub albedo: Option<Handle<Texture>>,
}

impl Material {
    pub fn new() -> Self {
        Self {
            color: Color::white(),
            albedo: None,
        }
    }
}

impl Default for Material {
    fn default() -> Self {
        Self::new()
    }
}
