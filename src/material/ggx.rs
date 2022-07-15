// Copyright © 2022-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::{Color, GltfModel, Handle, Mat3, Sampler, Texture, Vec2, Vec3};

#[derive(Default)]
pub struct GgxMaterialBuilder {
    color: Color,
}

impl GgxMaterialBuilder {
    pub fn new() -> Self {
        Self {
            color: Color::default(),
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn build(self) -> GgxMaterial {
        let mut material = GgxMaterial::new();
        material.color = self.color;
        material
    }
}

#[derive(Debug)]
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

    pub fn builder() -> GgxMaterialBuilder {
        GgxMaterialBuilder::new()
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

    pub fn get_color(&self, uv: Vec2, model: &GltfModel) -> Color {
        if let Some(texture) = model.textures.get(self.albedo_texture) {
            let sampler = Sampler::default();
            let image = model.images.get(texture.image).unwrap();
            self.color * sampler.sample(image, uv)
        } else {
            self.color
        }
    }

    pub fn get_normal(
        &self,
        uv: Vec2,
        normal: Vec3,
        tangent: Vec3,
        bitangent: Vec3,
        model: &GltfModel,
    ) -> Vec3 {
        if let Some(texture) = model.textures.get(self.normal_texture) {
            let sampler = Sampler::default();
            let image = model.images.get(texture.image).unwrap();
            let mut sampled_normal = Vec3::from(sampler.sample(image, uv));
            sampled_normal = sampled_normal * 2.0 - 1.0;
            let tbn = Mat3::tbn(&tangent, &bitangent, &normal);
            (&tbn * sampled_normal).get_normalized()
        } else {
            normal
        }
    }

    pub fn get_metallic_roughness(&self, uv: Vec2, model: &GltfModel) -> (f32, f32) {
        if let Some(texture) = model.textures.get(self.metallic_roughness_texture) {
            let sampler = Sampler::default();
            let image = model.images.get(texture.image).unwrap();
            let color = sampler.sample(image, uv);
            // Blue channel contains metalness value
            // Red channel contains roughness value
            (color.b, color.r)
        } else {
            (self.metallic_factor, self.roughness_factor)
        }
    }
}

impl Default for GgxMaterial {
    fn default() -> Self {
        Self::new()
    }
}
