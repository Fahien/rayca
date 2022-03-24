// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use super::*;

pub struct Material {
    pub color: RGBA8,
}

impl Material {
    pub fn new() -> Self {
        Self {
            color: RGBA8::white(),
        }
    }
}

impl Default for Material {
    fn default() -> Self {
        Self::new()
    }
}
