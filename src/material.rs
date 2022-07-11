// Copyright © 2022-2024
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
    pub albedo_texture: Handle<Texture>,
    pub normal_texture: Handle<Texture>,

    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub metallic_roughness_texture: Handle<Texture>,
}

impl Material {
    pub const WHITE: Material = Material {
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

    pub fn builder() -> MaterialBuilder {
        MaterialBuilder::new()
    }

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

    pub fn get_color(&self, model: &Model, uv: &Vec2) -> Color {
        if let Some(albedo_texture) = model.textures.get(self.albedo_texture) {
            let sampler = Sampler::default();
            let image = model.images.get(albedo_texture.image).unwrap();
            self.color * sampler.sample(image, uv)
        } else {
            self.color
        }
    }

    pub fn get_normal(
        &self,
        model: &Model,
        uv: &Vec2,
        normal: Vec3,
        tangent: Vec3,
        bitangent: Vec3,
    ) -> Vec3 {
        if let Some(normal_texture) = model.textures.get(self.normal_texture) {
            let sampler = Sampler::default();
            let image = model.images.get(normal_texture.image).unwrap();
            let mut sampled_normal = Vec3::from(sampler.sample(image, uv));
            sampled_normal = sampled_normal * 2.0 - 1.0;

            let tbn = Mat3::tbn(&tangent, &bitangent, &normal);
            (&tbn * sampled_normal).get_normalized()
        } else {
            normal
        }
    }

    pub fn get_metallic_roughness(&self, model: &Model, uv: &Vec2) -> (f32, f32) {
        if let Some(mr_texture) = model.textures.get(self.metallic_roughness_texture) {
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
