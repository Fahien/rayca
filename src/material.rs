// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use super::*;

fn saturate_mediump(x: f32) -> f32 {
    const MEDIUMP_FLT_MAX: f32 = 65504.0;
    x.min(MEDIUMP_FLT_MAX)
}

/// Models the distribution of the microfacet
/// Surfaces are not smooth at the micro level, but made of a
/// large number of randomly aligned planar surface fragments.
/// This implementation is good for half-precision floats.
fn distribution_ggx(n_dot_h: f32, n: &Vec3, h: &Vec3, roughness: f32) -> f32 {
    let n_x_h = n.cross(h);
    let a = n_dot_h * roughness;
    let k = roughness / (n_x_h.dot(&n_x_h) + a * a);
    let d = k * k * (1.0 / std::f32::consts::PI);
    saturate_mediump(d)
}

fn fresnel_schlick(cos_theta: f32, f0: Vec3) -> Vec3 {
    let f = (1.0 - cos_theta).powf(5.0);
    f + f0 * (Vec3::splat(1.0) - f0)
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

    pub fn get_radiance(&self, li: &LightIntersection, model: &Model) -> Color {
        let (metallic, roughness) = self.get_metallic_roughness(&li.uv, model);

        // Cook-Torrance approximation of the microfacet model integration
        let d = distribution_ggx(li.n_dot_h, &li.n, &li.h, roughness);

        let reflectance = 0.5;
        let f0_value = 0.16 * reflectance * reflectance * (1.0 - metallic);
        let f0 = Vec3::splat(f0_value) + Vec3::from(&li.albedo) * metallic;
        let f = fresnel_schlick(li.l_dot_h, f0);

        let g = geometry_smith_ggx(li.n_dot_v, li.n_dot_l, roughness);

        let fr = (d * g) * Color::from(f);

        // Lambertian diffuse (1/PI)
        let fd = li.albedo * std::f32::consts::FRAC_1_PI;

        let light_color = li.light.light.get_intensity();
        let fallof = li.light.light.get_fallof(li.light.trs, &li.hit.point);
        ((fd + fr) * light_color * li.n_dot_l) / fallof
    }
}

impl Default for Material {
    fn default() -> Self {
        Self::new()
    }
}
