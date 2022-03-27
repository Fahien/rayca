// Copyright © 2022-2024
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use super::*;

pub struct Material {
    pub color: Color,
    pub albedo: Handle<Texture>,
}

impl Material {
    pub fn new() -> Self {
        Self {
            color: Color::white(),
            albedo: Handle::NONE,
        }
    }
}

impl Default for Material {
    fn default() -> Self {
        Self::new()
    }
}
