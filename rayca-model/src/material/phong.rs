// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use bon::Builder;

use crate::*;

#[derive(Clone, Debug, Builder)]
pub struct PhongMaterial {
    #[builder(default = 0)]
    pub shader: u32,

    #[builder(default = Color::WHITE)]
    pub color: Color,

    #[builder(default = Color::WHITE)]
    pub specular: Color,

    #[builder(default = 0.0)]
    pub shininess: f32,
}

impl Default for PhongMaterial {
    fn default() -> Self {
        PhongMaterial {
            shader: 0,
            color: Color::WHITE,
            specular: Color::WHITE,
            shininess: 0.0,
        }
    }
}

impl PhongMaterial {
    pub const WHITE: PhongMaterial = PhongMaterial {
        shader: 0,
        color: Color::WHITE,
        specular: Color::WHITE,
        shininess: 0.0,
    };

    pub fn get_color(&self) -> Color {
        self.color
    }
}
