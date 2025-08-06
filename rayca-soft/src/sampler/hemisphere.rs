// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

pub struct HemisphereSampler {}

impl HemisphereSampler {
    pub const fn new() -> Self {
        Self {}
    }

    /// Returns a random direction in the hemisphere centered around the normal `n`.
    fn get_random_dir(n: Vec3) -> Vec3 {
        let e1 = fastrand::f32();
        let e2 = fastrand::f32();

        let theta = e1.acos();
        let omega = 2.0 * std::f32::consts::PI * e2;

        let s = Vec3::new(
            omega.cos() * theta.sin(),
            omega.sin() * theta.sin(),
            theta.cos(),
        );
        // We need to rotate s so that the emisphere is centered around n
        let w = n;
        let a = if w.close(Vec3::Y_AXIS) {
            Vec3::X_AXIS
        } else {
            Vec3::Y_AXIS
        };
        let u = a.cross(w).get_normalized();
        let v = w.cross(u).get_normalized();

        s.get_x() * u + s.get_y() * v + s.get_z() * w
    }
}

impl SoftSampler for HemisphereSampler {
    fn get_indirect_lighting(
        &self,
        config: &Config,
        scene: &SceneDrawInfo,
        tlas: &Tlas,
        hit: &Hit,
        n: Vec3,
        r: Vec3,
        material: &PhongMaterial,
        depth: u32,
    ) -> Color {
        let mut li = Color::BLACK;

        let collect_emissive = !config.next_event_estimation;
        // Move ray origin slightly along the surface normal to avoid self intersections
        let next_origin = hit.point + n * Ray::BIAS;

        for _ in 0..config.light_samples {
            let omega_i = Self::get_random_dir(n);
            let brdf = lambertian::get_brdf(material, r, omega_i);

            let mut next_ray = Ray::new(next_origin, omega_i);
            let mut weight = 1.0;

            if config.russian_roulette {
                use std::f32::consts::PI;
                let next_throughput = 2.0 * PI * hit.ray.throughput * brdf * n.dot(omega_i);
                if let Some(boost_factor) = hit.ray.next_russian_roulette(next_throughput) {
                    weight = boost_factor;
                    next_ray.throughput = next_throughput * boost_factor;
                } else {
                    continue; // Russian roulette terminated the ray
                }
            }

            if let Some(indirect_sample) =
                Pathtracer::trace_impl(config, scene, next_ray, tlas, depth + 1, collect_emissive)
            {
                let cosin_law = omega_i.dot(n);
                // Boost factor applies to the returned radiance as well
                li += brdf * cosin_law * indirect_sample * weight;
            }
        }

        (2.0 * std::f32::consts::PI * li) / (config.light_samples as f32)
    }
}
