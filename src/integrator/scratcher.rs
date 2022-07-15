// Copyright © 2022-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::{Color, Dot, Integrator, Ray, Scene, Vec3};

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
pub struct Scratcher {}

impl Integrator for Scratcher {
    fn trace(&self, scene: &Scene, ray: Ray, depth: u8) -> Color {
        let mut pixel_color = Color::new(0.0, 0.0, 0.0, 0.0);

        if depth > 1 {
            return pixel_color;
        }

        let Some(hit) = scene.tlas.intersects(&ray) else {
            return pixel_color;
        };

        let blas_node = &scene.tlas.blas_nodes[hit.blas as usize];
        let bvh = scene.tlas.bvhs.get(blas_node.bvh).unwrap();
        let primitive = bvh.get_shade(hit.primitive);
        let n = primitive.get_normal(scene, &hit);
        let mut color = primitive.get_color(scene, &hit);
        pixel_color += color / 8.0;

        const RAY_BIAS: f32 = 1e-4;
        let n_dot_v = n.dot(&ray.dir).abs() + RAY_BIAS;

        if color.a < 1.0 {
            let transmit_origin = hit.point + -n * RAY_BIAS;
            let transmit_ray = Ray::new(transmit_origin, ray.dir);
            let mut transmit_color = self.trace(scene, transmit_ray, depth + 1);
            // continue with the rest of the shading?
            transmit_color.over(color);
            color = transmit_color;
        }

        let (metallic, roughness) = primitive.get_metallic_roughness(scene, &hit);

        // Before getting color, we should check whether it is visible from the sun
        let next_origin = hit.point + n * RAY_BIAS;

        for solved_light in scene.default_lights.lights.iter() {
            let light = &solved_light.light;
            let light_dir = light.get_direction(&solved_light.trs, &hit.point);

            let shadow_ray = Ray::new(next_origin, light_dir);
            let shadow_result = scene.tlas.intersects(&shadow_ray);

            let is_lit = match shadow_result {
                None => true,
                Some(shadow_hit) => {
                    // Distance between current surface and the light source
                    let light_distance = light.get_distance(&solved_light.trs, &hit.point);
                    // If the obstacle is beyong the light source then the current surface is light
                    if shadow_hit.depth > light_distance {
                        true
                    } else {
                        // Check whether the obstacle is a transparent surface
                        let blas_node = &scene.tlas.blas_nodes[shadow_hit.blas as usize];
                        let shadow_bvh = scene.tlas.bvhs.get(blas_node.bvh).unwrap();
                        let shadow_primitive = shadow_bvh.get_shade(shadow_hit.primitive);
                        let shadow_color = shadow_primitive.get_color(scene, &shadow_hit);
                        shadow_color.a < 1.0
                    }
                }
            };

            if is_lit {
                // Cook-Torrance approximation of the microfacet model integration
                let n_dot_l = n.dot(&light_dir).clamp(0.0, 1.0);
                let h = (-ray.dir + light_dir).get_normalized();
                let n_dot_h = n.dot(&h).clamp(0.0, 1.0);
                let l_dot_h = light_dir.dot(&h).clamp(0.0, 1.0);
                let d = distribution_ggx(n_dot_h, roughness);
                let f0 = Vec3::splat(0.04) * (1.0 - metallic) + Vec3::from(&color) * metallic;
                let f = fresnel_schlick(l_dot_h, f0);
                let ks = f;
                let kd = (Vec3::splat(1.0) - ks) * (1.0 - metallic);
                let g = geometry_smith_ggx(n_dot_v, n_dot_l, roughness);
                let fr = (d * g) * Color::from(f);
                // Lambertian diffuse (1/PI)
                let fd = Color::from(kd) * color * std::f32::consts::FRAC_1_PI;
                let fallof = light.get_fallof(&solved_light.trs, &hit.point);
                pixel_color += ((fd + fr) * light.color * n_dot_l) / fallof;
            }
        } // end iterate light

        // Reflection component
        let reflection_dir = ray.dir.reflect(&n).get_normalized();
        let reflection_ray = Ray::new(next_origin, reflection_dir);
        let reflection_color = self.trace(scene, reflection_ray, depth + 1);
        // Cosine-law applies here as well
        let n_dot_r = n.dot(&reflection_dir);
        pixel_color += reflection_color * (metallic + 0.125) * n_dot_r;

        pixel_color
    }
}
