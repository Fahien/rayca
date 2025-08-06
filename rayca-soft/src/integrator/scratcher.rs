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
        tlas: &Tlas,
        ray: Ray,
        depth: u32,
    ) -> Option<Color> {
        if depth > config.max_depth {
            return None;
        }

        let mut hit = tlas.intersects(scene, ray)?;

        // Ambient?
        let mut pixel_color = Color::BLACK;

        if hit.is_transparent() {
            let transmit_result =
                self.trace(config, scene, tlas, hit.get_transmit_ray(), depth + 1);

            if let Some(mut transmit_color) = transmit_result {
                // continue with the rest of the shading?
                transmit_color.over(hit.get_color());
                pixel_color += transmit_color;
            }
        }

        // Direct component
        for light_draw_info in scene.light_draw_infos.iter().copied() {
            let light_node = scene.get_node(light_draw_info);
            let light = scene.get_light(light_draw_info);
            let light_dir = light.get_direction(&light_node.trs, &hit.get_point());

            let shadow_ray = hit.get_shadow_ray(light_dir);
            let shadow_result = tlas.intersects(scene, shadow_ray);

            // Whether this object is light (verb) by a light (noun)
            let is_light = match shadow_result {
                None => true,
                Some(mut shadow_hit) => {
                    // Distance between current surface and the light source
                    let light_distance = light.get_distance(&light_node.trs, &hit.get_point());
                    // If the obstacle is beyong the light source then the current surface is light
                    if shadow_hit.get_depth() > light_distance {
                        true
                    } else {
                        // Check whether the obstacle is a transparent surface
                        shadow_hit.is_transparent()
                    }
                }
            };

            if is_light {
                let intensity =
                    light.get_intensity(&light_node.trs, hit.get_point(), hit.get_normal());
                let ir = Irradiance::new(intensity, &mut hit, light_dir);
                pixel_color += hit.get_radiance(ir);
            }
        } // end iterate light

        // Reflection component
        let reflection = hit.get_reflection();
        let reflection_ray = hit.get_reflection_ray();
        if let Some(reflection_intensity) =
            self.trace(config, scene, tlas, reflection_ray, depth + 1)
        {
            let ir = Irradiance::new(reflection_intensity, &mut hit, reflection);
            pixel_color += hit.get_radiance(ir);
        }

        Some(pixel_color)
    }
}
