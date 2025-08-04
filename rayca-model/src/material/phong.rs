// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use bon::Builder;

use crate::*;

#[derive(Clone, Debug, Builder)]
pub struct PhongMaterial {
    #[builder(default = 0)]
    pub shader: u32,

    /// The ambient color, representing constant background light reflected by the material.
    #[builder(default = Color::BLACK)]
    pub ambient: Color,

    /// The emission color, representing light emitted by the material.
    #[builder(default = Color::BLACK)]
    pub emission: Color,

    /// The diffuse color, representing light reflected in all directions.
    #[builder(default = Color::BLACK)]
    pub diffuse: Color,

    /// The specular color, representing the color and intensity of highlights caused by mirror-like reflections.
    #[builder(default = Color::BLACK)]
    pub specular: Color,

    /// The shininess factor, controlling the size and sharpness of specular highlights.
    #[builder(default = 0.0)]
    pub shininess: f32,
}

impl Default for PhongMaterial {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl PhongMaterial {
    pub const DEFAULT: PhongMaterial = PhongMaterial {
        shader: 0,
        ambient: Color::BLACK,
        emission: Color::BLACK,
        diffuse: Color::BLACK,
        specular: Color::BLACK,
        shininess: 0.0,
    };

    pub fn is_emissive(&self) -> bool {
        !self.emission.close(Color::BLACK)
    }

    pub fn get_color(&self) -> Color {
        self.ambient + self.emission
    }
}
