// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NextEventEstimationStrategy {
    None,
    Direct,
    AnalyticDirect,
}

pub struct Pathtracer {}

impl Pathtracer {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Pathtracer {
    fn get_indirect_lighting(config: &Config, hit: &mut HitInfo, depth: u32) -> Color {
        let mut li = Color::BLACK;

        let collect_emissive = config.direct_sampler.is_none();

        let sampler = config.indirect_sampler.get_indirect_sampler();

        for _ in 0..config.light_samples {
            let omega_i = sampler.get_random_dir(hit);
            let brdf = hit.get_brdf(omega_i);

            let mut next_ray = hit.get_next_ray(omega_i);
            let mut weight = 1.0;

            if config.russian_roulette {
                use std::f32::consts::PI;
                let next_throughput = 2.0
                    * PI
                    * hit.get_ray().throughput
                    * brdf
                    * hit.get_normal().dot(omega_i).clamp(0.0, 1.0);
                if let Some(boost_factor) = hit.get_ray().next_russian_roulette(next_throughput) {
                    weight = boost_factor;
                    next_ray.throughput = next_throughput * boost_factor;
                } else {
                    continue; // Russian roulette terminated the ray
                }
            }

            if let Some(indirect_sample) = Pathtracer::trace_impl(
                config,
                hit.scene,
                hit.tlas,
                next_ray,
                depth + 1,
                collect_emissive,
            ) {
                // Boost factor applies to the returned radiance as well
                li += sampler.get_radiance(hit, omega_i, indirect_sample, weight);
            }
        }

        li / (config.light_samples as f32)
    }

    pub fn trace_impl(
        config: &Config,
        scene: &SceneDrawInfo,
        tlas: &Tlas,
        ray: Ray,
        depth: u32,
        collect_emissive: bool,
    ) -> Option<Color> {
        if !config.russian_roulette && depth >= config.max_depth {
            return None;
        }

        let mut hit = tlas.intersects(scene, ray)?;

        // This is the color of the primitive with no light
        let ambient_and_emissive = hit.get_color();

        if collect_emissive && hit.is_emissive() {
            return Some(ambient_and_emissive);
        }

        let direct_lighting = config
            .direct_sampler
            .get_direct_sampler()
            .get_direct_lighting(config, &mut hit);

        let indirect_depth_limit = if !config.direct_sampler.is_none() {
            config.max_depth - 1
        } else {
            config.max_depth
        };

        let mut indirect_lighting = Color::BLACK;
        if config.russian_roulette || depth < indirect_depth_limit {
            indirect_lighting += Self::get_indirect_lighting(config, &mut hit, depth);
        }

        Some(direct_lighting + indirect_lighting)
    }
}

impl Integrator for Pathtracer {
    fn trace(
        &self,
        config: &Config,
        scene: &SceneDrawInfo,
        tlas: &Tlas,
        ray: Ray,
        depth: u32,
    ) -> Option<Color> {
        Self::trace_impl(config, scene, tlas, ray, depth, true)
    }
}
