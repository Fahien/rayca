// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

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

pub fn get_radiance(material: &PbrMaterial, ir: &Irradiance, model: &Model) -> Color {
    let (metallic, roughness) = material.get_metallic_roughness(model, ir.uv);

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
