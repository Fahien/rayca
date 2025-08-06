// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

#[derive(Default)]
pub struct NextEventEstimationSampler {}

impl NextEventEstimationSampler {
    fn sample(config: &Config, hit: &mut HitInfo, light_index: usize) -> Color {
        let light_draw_info = hit.scene.light_draw_infos[light_index];
        let light_node = hit.scene.get_node(light_draw_info);
        let quad_light = hit.scene.get_quad_light(light_draw_info);
        let area = quad_light.get_area();

        // Constant radiance?
        let le = quad_light.intensity * quad_light.color;

        let mut ld = Color::BLACK;

        let strate_count = config.get_strate_count();
        for i in 0..config.light_samples {
            let x1 = quad_light.get_random_point(
                &light_node.trs,
                config.light_stratify,
                strate_count,
                i,
            );

            // X is the point on the surface
            let x = hit.get_point();
            // Random sample incident direction
            let x_to_x1 = x1 - x;
            let omega_i = x_to_x1.get_normalized();

            // Let us see if we actually see the light
            let shadow_ray = hit.get_next_ray(omega_i);
            if let Some(mut shadow_hit) = hit.tlas.intersects(hit.scene, shadow_ray) {
                if !shadow_hit.is_emissive() {
                    continue;
                }
            }

            let brdf = lambertian::get_brdf(hit, omega_i);

            let r_squared = x_to_x1.norm();
            let d_omega_i = quad_light.get_normal().dot(omega_i) / r_squared;

            ld += brdf * hit.get_normal().dot(omega_i) * d_omega_i;
        }

        (le * area * ld) / (config.light_samples as f32)
    }

    fn get_samples(config: &Config, hit: &mut HitInfo) -> Vec<Color> {
        let mut samples = vec![];
        for light_index in 0..hit.scene.light_draw_infos.len() {
            let light_sample = Self::sample(config, hit, light_index);
            samples.push(light_sample);
        }
        samples
    }
}

impl DirectSampler for NextEventEstimationSampler {
    fn get_direct_lighting(&self, config: &Config, hit: &mut HitInfo) -> Color {
        let samples = Self::get_samples(config, hit);
        let mut ret = Color::BLACK;
        for sample in samples {
            ret += sample;
        }
        ret
    }
}
