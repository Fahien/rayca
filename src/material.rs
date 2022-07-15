// Copyright © 2022-2024
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use super::*;

/// Models the distribution of the microfacet
/// Surfaces are not smooth at the micro level, but made of a
/// large number of randomly aligned planar surface fragments.
/// This implementation is good for half-precision floats.
fn distribution_ggx(n_dot_h: f32, roughness: f32) -> f32 {
    let a = n_dot_h * roughness;
    let k = roughness / (1.0 - n_dot_h * n_dot_h + a * a);
    k * k * std::f32::consts::FRAC_1_PI
}

/// The amount of light the viewer sees reflected from a surface depends on the
/// viewing angle, in fact at grazing angles specular reflections become more intense
fn fresnel_schlick(cos_theta: f32, f0: Vec3) -> Vec3 {
    let f = (1.0 - cos_theta).powf(5.0);
    f0 + (Vec3::splat(1.0) - f0) * f
}

/// Models the visibility of the microfacets, or occlusion or shadow-masking
fn geometry_smith_ggx(n_dot_v: f32, n_dot_l: f32, roughness: f32) -> f32 {
    let a = roughness;
    let ggxv = n_dot_l * (n_dot_v * (1.0 - a) + a);
    let ggxl = n_dot_v * (n_dot_l * (1.0 - a) + a);
    0.5 / (ggxv + ggxl)
}

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

    pub fn get_radiance(&self, ir: &Irradiance, model: &Model) -> Color {
        let (metallic, roughness) = self.get_metallic_roughness(model, &ir.uv);

        let d = distribution_ggx(ir.n_dot_h, roughness);

        let f0 = Vec3::splat(0.04) * (1.0 - metallic) + Vec3::from(&ir.albedo) * metallic;
        let f = fresnel_schlick(ir.l_dot_h, f0);

        let ks = f;
        let kd = (Vec3::splat(1.0) - ks) * (1.0 - metallic);

        let g = geometry_smith_ggx(ir.n_dot_v, ir.n_dot_l, roughness);

        let fr = (d * g) * Color::from(f);

        // Lambertian diffuse (1/PI)
        let fd = Color::from(kd) * ir.albedo * std::f32::consts::FRAC_1_PI;

        (fd + fr) * ir.intensity * ir.n_dot_l
    }
}

impl Default for Material {
    fn default() -> Self {
        Self::new()
    }
}
