// Copyright © 2022-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::{Color, Handle, Texture};

pub struct GgxMaterial {
    pub color: Color,
    pub albedo_texture: Handle<Texture>,
    pub normal_texture: Handle<Texture>,

    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub metallic_roughness_texture: Handle<Texture>,
}

impl GgxMaterial {
    pub const WHITE: Self = Self {
        color: Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        },
        albedo_texture: Handle::NONE,
        normal_texture: Handle::NONE,
        metallic_factor: 1.0,
        roughness_factor: 1.0,
        metallic_roughness_texture: Handle::NONE,
    };

    pub fn new() -> Self {
        Self {
            color: Color::white(),
            albedo_texture: Handle::NONE,
            normal_texture: Handle::NONE,
            metallic_factor: 1.0,
            roughness_factor: 1.0,
            metallic_roughness_texture: Handle::NONE,
        }
    }
}

impl Default for GgxMaterial {
    fn default() -> Self {
        Self::new()
    }
}
