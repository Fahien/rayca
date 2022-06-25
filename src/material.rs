// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use super::*;

#[derive(Default)]
pub struct MaterialBuilder {
    color: Color,
}

impl MaterialBuilder {
    pub fn new() -> Self {
        Self {
            color: Color::default(),
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn build(self) -> Material {
        let mut material = Material::new();
        material.color = self.color;
        material
    }
}

pub struct Material {
    pub color: Color,
    pub albedo_texture: Option<Handle<Texture>>,
    pub normal_texture: Option<Handle<Texture>>,

    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub metallic_roughness_texture: Option<Handle<Texture>>,
}

impl Material {
    pub const WHITE: Material = Material {
        color: Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        },
        albedo_texture: None,
        normal_texture: None,
        metallic_factor: 1.0,
        roughness_factor: 1.0,
        metallic_roughness_texture: None,
    };

    pub fn builder() -> MaterialBuilder {
        MaterialBuilder::new()
    }

    pub fn new() -> Self {
        Self {
            color: Color::white(),
            albedo_texture: None,
            normal_texture: None,
            metallic_factor: 1.0,
            roughness_factor: 1.0,
            metallic_roughness_texture: None,
        }
    }

    pub fn get_metallic_roughness(&self, uv: &Vec2, model: &Model) -> (f32, f32) {
        if let Some(mr_handle) = self.metallic_roughness_texture {
            let mr_texture = model.textures.get(mr_handle).unwrap();
            let sampler = Sampler::default();
            let image = model.images.get(mr_texture.image).unwrap();
            let color = sampler.sample(image, uv);
            // Blue channel contains metalness value
            // Red channel contains roughness value
            (color.b, color.r)
        } else {
            (self.metallic_factor, self.roughness_factor)
        }
    }
}

impl Default for Material {
    fn default() -> Self {
        Self::new()
    }
}
