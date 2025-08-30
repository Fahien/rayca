// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use bon::Builder;

use crate::*;

#[derive(Clone, Debug, Builder)]
pub struct GgxMaterial {
    /// The diffuse color, representing light reflected in all directions.
    #[builder(default = Color::BLACK)]
    pub diffuse: Color,

    /// The specular color, representing the color and intensity of highlights caused by mirror-like reflections.
    #[builder(default = Color::BLACK)]
    pub specular: Color,

    /// The roughness value, controlling the microfacet distribution for specular reflections.
    #[builder(default = 0.0)]
    pub roughness: f32,
}

impl Default for GgxMaterial {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl GgxMaterial {
    pub const DEFAULT: Self = Self {
        diffuse: Color::BLACK,
        specular: Color::BLACK,
        roughness: 0.0,
    };

    pub fn new(diffuse: Color, specular: Color, roughness: f32) -> Self {
        Self {
            diffuse,
            specular,
            roughness,
        }
    }

    pub fn is_emissive(&self) -> bool {
        false
    }

    pub fn get_emission(&self) -> Color {
        Color::BLACK
    }

    /// Returns the specular weight
    pub fn get_t(&self) -> f32 {
        let kd_avg = self.diffuse.get_rgb().reduce_avg();
        let ks_avg = self.specular.get_rgb().reduce_avg();
        if ks_avg == 0.0 && kd_avg == 0.0 {
            return 1.0;
        }
        (ks_avg / (ks_avg + kd_avg)).max(0.25)
    }
}
