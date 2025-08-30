// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

#[derive(Default)]
pub struct Direct {}

impl Direct {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Integrator for Direct {
    fn trace(
        &self,
        config: &Config,
        scene: &SceneDrawInfo,
        tlas: &Tlas,
        ray: Ray,
        depth: u32,
    ) -> Option<Color> {
        if depth >= config.max_depth {
            return None;
        }

        let mut hit = tlas.intersects(scene, ray)?;

        // This is the color of the primitive with no light
        let ambient_and_emissive = hit.get_color();

        if hit.is_emissive() {
            return Some(ambient_and_emissive);
        }

        let mut light_contribution = Color::BLACK;

        let strate_count = config.get_strate_count();

        // Expecting only area lights here
        for light_draw_info in scene.light_draw_infos.iter().copied() {
            let light = scene.get_light(light_draw_info);
            let light_node = scene.get_node(light_draw_info);
            if let Light::Quad(quad_light) = light {
                let mut ld = Color::BLACK;

                for i in 0..config.light_samples {
                    let x1 = quad_light.get_random_point(
                        &light_node.trs,
                        config.light_stratify,
                        strate_count,
                        i,
                    );

                    // Random sample incident direction
                    let x1_to_hit_point = x1 - hit.hit.point;
                    let omega_i = x1_to_hit_point.get_normalized();

                    // Let us see if we actually see the light
                    let shadow_ray = Ray::new(hit.get_next_ray_origin(), omega_i);
                    if let Some(mut shadow_hit) = tlas.intersects(scene, shadow_ray) {
                        if !shadow_hit.is_emissive() {
                            continue;
                        }
                    }

                    let brdf = hit.get_brdf(omega_i);

                    let r_squared = x1_to_hit_point.len().powf(2.0);
                    let d_omega_i = quad_light.get_normal().dot(omega_i) / r_squared;

                    ld += brdf * hit.get_normal().dot(omega_i) * d_omega_i;
                }

                // Constant radiance?
                let li = quad_light.intensity * quad_light.color;
                let area = quad_light.get_area();
                ld = (li * area * ld) / (config.light_samples as f32);
                light_contribution += ld;
            }
        }

        Some(light_contribution)
    }
}
