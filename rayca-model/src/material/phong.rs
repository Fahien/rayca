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
    pub ambient: Color,

    #[builder(default = Color::BLACK)]
    pub emission: Color,

    #[builder(default = Color::BLACK)]
    pub diffuse: Color,

    #[builder(default = Color::BLACK)]
    pub specular: Color,

    #[builder(default = 0.0)]
    pub shininess: f32,
}

impl Default for PhongMaterial {
    fn default() -> Self {
        Self::WHITE
    }
}

impl PhongMaterial {
    pub const WHITE: PhongMaterial = PhongMaterial {
        shader: 0,
        ambient: Color::new(0.2, 0.2, 0.2, 1.0),
        emission: Color::BLACK,
        diffuse: Color::BLACK,
        specular: Color::BLACK,
        shininess: 0.0,
    };

    pub fn get_color(&self) -> Color {
        self.ambient + self.emission
    }
}
