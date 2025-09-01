// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

#[repr(C, align(16))]
#[derive(Default)]
pub struct PbrMaterialBuilder {
    color: Color,
    albedo: Handle<Texture>,
    normal: Handle<Texture>,
    metallic_factor: f32,
    roughness_factor: f32,
    metallic_roughness: Handle<Texture>,
}

impl PbrMaterialBuilder {
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn albedo(mut self, albedo: Handle<Texture>) -> Self {
        self.albedo = albedo;
        self
    }

    pub fn normal(mut self, normal: Handle<Texture>) -> Self {
        self.normal = normal;
        self
    }

    pub fn metallic_factor(mut self, metallic_factor: f32) -> Self {
        self.metallic_factor = metallic_factor;
        self
    }

    pub fn roughness_factor(mut self, roughness_factor: f32) -> Self {
        self.roughness_factor = roughness_factor;
        self
    }

    pub fn metallic_roughness(mut self, metallic_roughness: Handle<Texture>) -> Self {
        self.metallic_roughness = metallic_roughness;
        self
    }

    pub fn build(self) -> PbrMaterial {
        PbrMaterial {
            color: self.color,
            albedo: self.albedo,
            normal: self.normal,
            metallic_factor: self.metallic_factor,
            roughness_factor: self.roughness_factor,
            metallic_roughness: self.metallic_roughness,
        }
    }
}

#[repr(C, align(16))]
#[derive(Clone, Default, Debug)]
pub struct PbrMaterial {
    pub color: Color,
    pub albedo: Handle<Texture>,
    pub normal: Handle<Texture>,
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub metallic_roughness: Handle<Texture>,
}

impl PbrMaterial {
    pub const WHITE: Self = Self {
        color: Color::WHITE,
        albedo: Handle::NONE,
        normal: Handle::NONE,
        metallic_factor: 0.0,
        roughness_factor: 1.0,
        metallic_roughness: Handle::NONE,
    };

    pub fn builder() -> PbrMaterialBuilder {
        PbrMaterialBuilder::default()
    }

    pub fn is_emissive(&self) -> bool {
        false
    }

    pub fn get_emission(&self) -> Color {
        Color::BLACK
    }

    pub fn get_color(&self, model: &Model, uv: Vec2) -> Color {
        if let Some(albedo_texture) = model.textures.get(self.albedo) {
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
        uv: Vec2,
        normal: Vec3,
        tangent: Vec3,
        bitangent: Vec3,
    ) -> Vec3 {
        if let Some(normal_texture) = model.textures.get(self.normal) {
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

    pub fn get_metallic_roughness(&self, model: &Model, uv: Vec2) -> (f32, f32) {
        if let Some(mr_texture) = model.textures.get(self.metallic_roughness) {
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

impl std::fmt::Display for PbrMaterial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{ \"pbrMetallicRoughness\": {{ \"baseColorFactor\": {},\"metallicFactor\": {}, \"roughnessFactor\": {}",
            self.color, self.metallic_factor, self.roughness_factor,
        )?;

        if self.albedo.is_valid() {
            write!(
                f,
                ", \"baseColorTexture\": {{ \"index\": {} }}",
                self.albedo.id
            )?;
        }
        if self.normal.is_valid() {
            write!(
                f,
                ", \"normalTexture\": {{ \"index\": {} }}",
                self.normal.id
            )?;
        }
        if self.metallic_roughness.is_valid() {
            write!(
                f,
                ", \"metallicRoughnessTexture\": {{ \"index\": {} }}",
                self.metallic_roughness.id
            )?;
        }
        write!(f, "}} }}")?;
        Ok(())
    }
}
