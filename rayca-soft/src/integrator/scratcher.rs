// Copyright © 2022-2024
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

#[derive(Default, Clone)]
pub struct Scratcher {}

impl Scratcher {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Integrator for Scratcher {
    fn trace(
        &self,
        config: &Config,
        scene: &SceneDrawInfo,
        ray: Ray,
        tlas: &Tlas,
        depth: u32,
    ) -> Option<Color> {
        if depth > config.max_depth {
            return None;
        }

        let hit = tlas.intersects(scene, ray)?;
        let blas = &tlas.get_blas(hit.blas);
        let primitive = &blas.model.primitives[hit.primitive as usize];

        let n = primitive.get_normal(scene, &hit);

        let albedo_color = primitive.get_color(scene, &hit);

        // Ambient?
        let mut pixel_color = Color::BLACK;
        const RAY_BIAS: f32 = 1e-3;

        if albedo_color.is_transparent() {
            let transmit_origin = hit.point + -n * RAY_BIAS;
            let transmit_ray = Ray::new(transmit_origin, hit.ray.dir);
            let transmit_result = self.trace(config, scene, transmit_ray, tlas, depth + 1);

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
        for light_draw_info in scene.light_draw_infos.iter().copied() {
            let light_node = scene.get_node(light_draw_info);
            let light = scene.get_light(light_draw_info);
            let light_dir = light.get_direction(&light_node.trs, &hit.point);

            let shadow_ray = Ray::new(next_origin, light_dir);
            let shadow_result = tlas.intersects(scene, shadow_ray);

            // Whether this object is light (verb) by a light (noun)
            let is_light = match shadow_result {
                None => true,
                Some(shadow_hit) => {
                    // Distance between current surface and the light source
                    let light_distance = light.get_distance(&light_node.trs, &hit.point);
                    // If the obstacle is beyong the light source then the current surface is light
                    if shadow_hit.depth > light_distance {
                        true
                    } else {
                        // Check whether the obstacle is a transparent surface
                        let shadow_blas = &tlas.get_blas(shadow_hit.blas);
                        let shadow_primitive =
                            &shadow_blas.model.primitives[shadow_hit.primitive as usize];
                        let shadow_color = shadow_primitive.get_color(scene, &shadow_hit);
                        shadow_color.is_transparent()
                    }
                }
            };

            if is_light {
                let intensity = light.get_intensity(&light_node.trs, hit.point, n);
                let ir = Irradiance::new(
                    intensity,
                    &hit,
                    light_dir,
                    n,
                    -hit.ray.dir,
                    albedo_color,
                    uv,
                );
                pixel_color += primitive.get_radiance(scene, &ir);
            }
        } // end iterate light

        // Reflection component
        let reflection_dir = hit.ray.dir.reflect(&n).get_normalized();
        let reflection_ray = Ray::new(next_origin, reflection_dir);
        if let Some(reflection_intensity) =
            self.trace(config, scene, reflection_ray, tlas, depth + 1)
        {
            let ir = Irradiance::new(
                reflection_intensity,
                &hit,
                reflection_dir,
                n,
                -hit.ray.dir,
                albedo_color,
                uv,
            );
            pixel_color += primitive.get_radiance(scene, &ir);
        }

        Some(pixel_color)
    }
}
