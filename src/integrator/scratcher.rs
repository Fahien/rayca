// Copyright © 2022-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::{Color, Dot, Integrator, Ray, Scene};

#[derive(Default)]
pub struct Scratcher {}

impl Integrator for Scratcher {
    fn trace(&self, scene: &Scene, ray: Ray, depth: u8) -> Color {
        let mut pixel_color = Color::BLACK;

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
        let color = primitive.get_color(scene, &hit);

        const SHADOW_BIAS: f32 = 1e-4;
        // Before getting color, we should check whether it is visible from the sun
        let shadow_origin = hit.point + n * SHADOW_BIAS;

        for light_node in scene.default_lights.nodes.iter() {
            let light = scene
                .default_lights
                .lights
                .get(light_node.light.unwrap())
                .unwrap();
            let light_dir = light.get_direction(light_node, &hit.point);

            let shadow_ray = Ray::new(shadow_origin, light_dir);
            let shadow_result = scene.tlas.intersects(&shadow_ray);

            let is_lit = match shadow_result {
                None => true,
                Some(shadow_hit) => {
                    // Distance between current surface and the light source
                    let light_distance = light.get_distance(light_node, &hit.point);
                    // If the obstacle is beyong the light source then the current surface is light
                    shadow_hit.depth > light_distance
                }
            };

            if is_lit {
                let n_dot_l = n.dot(&light_dir).clamp(0.0, 1.0);
                let fallof = light.get_fallof(&light_node.trs, &hit.point);
                pixel_color += (color / std::f32::consts::PI * light.color * n_dot_l) / fallof;
            }
        }

        // Get reflection?
        let reflection_dir = ray.dir.reflect(&n);
        let reflection_ray = Ray::new(hit.point, reflection_dir);
        let reflection_color = self.trace(scene, reflection_ray, depth + 1);
        pixel_color += reflection_color / 2.0;

        pixel_color
    }
}
