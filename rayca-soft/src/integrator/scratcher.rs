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

impl Integrator for Scratcher {
    fn trace(&self, model: &Model, ray: Ray, bvh: &Bvh, depth: u32) -> Option<Color> {
        if depth > 1 {
            return None;
        }

        let (hit, primitive) = bvh.intersects_iter(model, &ray)?;

        let n = primitive.get_normal(model, &hit);

        let albedo_color = primitive.get_color(model, &hit);

        // Ambient?
        let mut pixel_color = Color::black() + albedo_color / 8.0;
        const RAY_BIAS: f32 = 1e-3;

        if albedo_color.a < 1.0 {
            let transmit_origin = hit.point + -n * RAY_BIAS;
            let transmit_ray = Ray::new(transmit_origin, ray.dir);
            let transmit_result = self.trace(model, transmit_ray, bvh, depth + 1);

            if let Some(mut transmit_color) = transmit_result {
                // continue with the rest of the shading?
                transmit_color.over(albedo_color);
                pixel_color += transmit_color;
            }
        }

        // Before getting color, we should check whether it is visible from the sun
        let next_origin = hit.point + n * RAY_BIAS;

        let uv = primitive.geometry.get_uv(&hit);

        // Direct component
        for light_node_handle in &model.light_nodes {
            let light_node = model.nodes.get(*light_node_handle).unwrap();
            let light = model.lights.get(light_node.light).unwrap();
            let light_dir = light.get_direction(&light_node.trs, &hit.point);

            let shadow_ray = Ray::new(next_origin, light_dir);
            let shadow_result = bvh.intersects_iter(model, &shadow_ray);

            // Whether this object is light (verb) by a light (noun)
            let is_light = match shadow_result {
                None => true,
                Some((shadow_hit, primitive)) => {
                    // Distance between current surface and the light source
                    let light_distance = light.get_distance(&light_node.trs, &hit.point);
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
                let intensity = light.get_intensity(&light_node.trs, &hit.point);
                let ir = Irradiance::new(intensity, &hit, light_dir, n, -ray.dir, albedo_color, uv);
                pixel_color += primitive.get_radiance(model, &ir);
            }
        } // end iterate light

        // Reflection component
        let reflection_dir = ray.dir.reflect(&n).get_normalized();
        let reflection_ray = Ray::new(next_origin, reflection_dir);
        if let Some(reflection_intensity) = self.trace(model, reflection_ray, bvh, depth + 1) {
            let ir = Irradiance::new(
                reflection_intensity,
                &hit,
                reflection_dir,
                n,
                -ray.dir,
                albedo_color,
                uv,
            );
            pixel_color += primitive.get_radiance(model, &ir);
        }

        Some(pixel_color)
    }
}
