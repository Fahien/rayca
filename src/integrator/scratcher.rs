// Copyright © 2022-2024
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

#[derive(Default)]
pub struct Scratcher {}

impl Scratcher {
    pub fn new() -> Self {
        Self::default()
    }
}

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

impl Integrator for Scratcher {
    fn trace(&self, model: &Model, ray: Ray, bvh: &Bvh, depth: u32) -> Option<Color> {
        if depth > 1 {
            return None;
        }

        let (hit, primitive) = bvh.intersects_iter(model, &ray)?;

        let n = primitive.get_normal(model, &hit);

        const RAY_BIAS: f32 = 1e-4;

        let n_dot_v = n.dot(&ray.dir).abs() + RAY_BIAS;

        let mut color = primitive.get_color(model, &hit);

        let mut pixel_color = Color::black() + color / 8.0;

        if color.a < 1.0 {
            let transmit_origin = hit.point + -n * RAY_BIAS;
            let transmit_ray = Ray::new(transmit_origin, ray.dir);
            let transmit_result = self.trace(model, transmit_ray, bvh, depth + 1);

            if let Some(mut transmit_color) = transmit_result {
                // continue with the rest of the shading?
                transmit_color.over(color);
                color = transmit_color;
            }
        }

        let (metallic, roughness) = primitive.get_metallic_roughness(model, &hit);

        // Before getting color, we should check whether it is visible from the sun
        let next_origin = hit.point + n * RAY_BIAS;

        for light_node_handle in &model.light_nodes {
            let light_node = model.nodes.get(*light_node_handle).unwrap();
            let light = model.lights.get(light_node.light).unwrap();
            let light_dir = light.get_direction(light_node, &hit.point);

            let shadow_ray = Ray::new(next_origin, light_dir);
            let shadow_result = bvh.intersects_iter(model, &shadow_ray);

            // Whether this object is light (verb) by a light (noun)
            let is_light = match shadow_result {
                None => true,
                Some((shadow_hit, primitive)) => {
                    // Distance between current surface and the light source
                    let light_distance = light.get_distance(light_node, &hit.point);
                    // If the obstacle is beyong the light source then the current surface is light
                    if shadow_hit.depth > light_distance {
                        true
                    } else {
                        // Check whether the obstacle is a transparent surface
                        let shadow_color = primitive.get_color(model, &shadow_hit);
                        shadow_color.a < 1.0
                    }
                }
            };

            if is_light {
                let n_dot_l = n.dot(&light_dir).clamp(0.0, 1.0);

                // Cook-Torrance approximation of the microfacet model integration
                let h = (-ray.dir + light_dir).get_normalized();
                let n_dot_h = n.dot(&h).clamp(0.0, 1.0);
                let d = distribution_ggx(n_dot_h, &n, &h, roughness);

                let l_dot_h = light_dir.dot(&h).clamp(0.0, 1.0);
                let reflectance = 0.5;
                let f0_value = 0.16 * reflectance * reflectance * (1.0 - metallic);
                let f0 = Vec3::splat(f0_value) + Vec3::from(&color) * metallic;
                let f = fresnel_schlick(l_dot_h, f0);

                let g = geometry_smith_ggx(n_dot_v, n_dot_l, roughness);

                let fr = (d * g) * Color::from(f);

                // Lambertian diffuse (1/PI)
                let fd = color * std::f32::consts::FRAC_1_PI;

                let light_color = light.get_intensity();
                let fallof = light.get_fallof(&light_node.trs, &hit.point);
                pixel_color += ((fd + fr) * light_color * n_dot_l) / fallof;
            }
        } // end iterate light

        let reflection_dir = ray.dir.reflect(&n).get_normalized();
        let reflection_ray = Ray::new(next_origin, reflection_dir);
        if let Some(reflection_color) = self.trace(model, reflection_ray, bvh, depth + 1) {
            // Cosine-law applies here as well
            let n_dot_r = n.dot(&reflection_dir);
            pixel_color += reflection_color * (metallic + 0.125) * n_dot_r;
        }

        Some(pixel_color)
    }
}
