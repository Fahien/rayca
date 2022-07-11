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

    pub fn get_color(&self, uv: &Vec2, model: &Model) -> Color {
        match self.albedo_texture {
            Some(albedo_handle) => {
                let texture = model.textures.get(albedo_handle).unwrap();
                let sampler = Sampler::default();
                let image = model.images.get(texture.image).unwrap();
                self.color * sampler.sample(image, uv)
            }
            None => self.color,
        }
    }

    pub fn get_normal(
        &self,
        uv: &Vec2,
        normal: Vec3,
        tangent: Vec3,
        bitangent: Vec3,
        model: &Model,
    ) -> Vec3 {
        match self.normal_texture {
            Some(normal_handle) => {
                let texture = model.textures.get(normal_handle).unwrap();
                let sampler = Sampler::default();
                let image = model.images.get(texture.image).unwrap();
                let mut sampled_normal = Vec3::from(sampler.sample(image, uv));
                sampled_normal = sampled_normal * 2.0 - 1.0;

                let tbn = Mat3::tbn(&tangent, &bitangent, &normal);
                (&tbn * sampled_normal).get_normalized()
            }
            None => normal,
        }
    }

    pub fn get_metallic_roughness(&self, uv: &Vec2, model: &Model) -> (f32, f32) {
        match self.metallic_roughness_texture {
            Some(mr_handle) => {
                let mr_texture = model.textures.get(mr_handle).unwrap();
                let sampler = Sampler::default();
                let image = model.images.get(mr_texture.image).unwrap();
                let color = sampler.sample(image, uv);
                // Blue channel contains metalness value
                // Red channel contains roughness value
                (color.b, color.r)
            }
            None => (self.metallic_factor, self.roughness_factor),
        }
    }
}

impl Default for Material {
    fn default() -> Self {
        Self::new()
    }
}
