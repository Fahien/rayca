// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

#[derive(Default)]
pub struct NextEventEstimationSampler {}

impl NextEventEstimationSampler {
    fn sample(
        config: &Config,
        scene: &SceneDrawInfo,
        tlas: &Tlas,
        hit: &Hit,
        material: &PhongMaterial,
        n: Vec3,
        r: Vec3,
        light_index: usize,
    ) -> Color {
        let light_draw_info = scene.light_draw_infos[light_index];
        let light_node = scene.get_node(light_draw_info);
        let quad_light = scene.get_quad_light(light_draw_info);
        let area = quad_light.get_area();

        // Constant radiance?
        let le = quad_light.intensity * quad_light.color;
        let next_origin = hit.point + n * Ray::BIAS;

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
            let x = hit.point;
            // Random sample incident direction
            let x_to_x1 = x1 - x;
            let omega_i = x_to_x1.get_normalized();

            // Let us see if we actually see the light
            let shadow_ray = Ray::new(next_origin, omega_i);
            if let Some(shadow_hit) = tlas.intersects(scene, shadow_ray) {
                let shadow_primitive = tlas.get_primitive(&shadow_hit);
                if !shadow_primitive.is_emissive(scene) {
                    continue;
                }
            }

            let brdf = lambertian::get_brdf(material, r, omega_i);

            let r_squared = x_to_x1.norm();
            let d_omega_i = quad_light.get_normal().dot(omega_i) / r_squared;

            ld += brdf * n.dot(omega_i) * d_omega_i;
        }

        (le * area * ld) / (config.light_samples as f32)
    }

    fn get_samples(
        config: &Config,
        scene: &SceneDrawInfo,
        tlas: &Tlas,
        hit: &Hit,
        material: &PhongMaterial,
        n: Vec3,
        r: Vec3,
    ) -> Vec<Color> {
        let mut samples = vec![];
        for light_index in 0..scene.light_draw_infos.len() {
            let light_sample = Self::sample(config, scene, tlas, hit, material, n, r, light_index);
            samples.push(light_sample);
        }
        samples
    }
}

impl DirectSampler for NextEventEstimationSampler {
    fn get_direct_lighting(
        &self,
        config: &Config,
        scene: &SceneDrawInfo,
        tlas: &Tlas,
        hit: &Hit,
        material: &PhongMaterial,
        n: Vec3,
        r: Vec3,
    ) -> Color {
        let samples = Self::get_samples(config, scene, tlas, hit, material, n, r);
        let mut ret = Color::BLACK;
        for sample in samples {
            ret += sample;
        }
        ret
    }
}
