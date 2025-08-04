// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

pub enum NextEventEstimationStrategy {
    None,
    Direct,
    AnalyticDirect,
}

pub enum SamplerStrategy {
    Hemisphere,
    Uniform,
}

pub struct Pathtracer {}

impl Pathtracer {
    pub const RAY_BIAS: f32 = 1e-4;

    pub const fn new() -> Self {
        Self {}
    }
}

impl Pathtracer {
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

    fn get_direct_lighting(
        config: &Config,
        scene: &SceneDrawInfo,
        tlas: &Tlas,
        hit: &Hit,
        n: Vec3,
        r: Vec3,
        material: &PhongMaterial,
    ) -> Color {
        let mut direct_lighting = Color::BLACK;

        let strate_count = config.get_strate_count();
        // Move ray origin slightly along the surface normal to avoid self intersections
        let next_origin = hit.point + n * Self::RAY_BIAS;

        for light_draw_info in scene.light_draw_infos.iter().copied() {
            let light_node = scene.get_node(light_draw_info);
            let quad_light = scene.get_quad_light(light_draw_info);
            let area = quad_light.get_area();

            // Constant radiance?
            let le = quad_light.intensity * quad_light.color;

            let mut ld = Color::BLACK;

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
                if let Some(shadow_hit) = tlas.intersects(scene, &shadow_ray) {
                    let shadow_primitive = tlas.get_primitive(&shadow_hit);
                    if !shadow_primitive.is_emissive(scene) {
                        continue;
                    }
                }

                // BRDF
                let kd = material.diffuse;
                let lambertian = kd * std::f32::consts::FRAC_1_PI;
                let ks = material.specular;
                let s = material.shininess;
                let brdf = lambertian
                    + (ks * (s + 2.0) * r.dot(omega_i).powf(s)) * std::f32::consts::FRAC_1_PI / 2.0;

                let r_squared = x_to_x1.norm();
                let d_omega_i = quad_light.get_normal().dot(omega_i) / r_squared;

                ld += brdf * n.dot(omega_i) * d_omega_i;
            }

            ld = (le * area * ld) / (config.light_samples as f32);

            direct_lighting += ld;
        } // End direct lighting

        direct_lighting
    }

    fn get_indirect_lighting(
        config: &Config,
        scene: &SceneDrawInfo,
        tlas: &Tlas,
        hit: &Hit,
        n: Vec3,
        r: Vec3,
        material: &PhongMaterial,
        depth: u32,
    ) -> Color {
        let mut indirect_lighting = Color::BLACK;

        let collect_emissive = !config.next_event_estimation;
        // Move ray origin slightly along the surface normal to avoid self intersections
        let next_origin = hit.point + n * Self::RAY_BIAS;

        for _ in 0..config.light_samples {
            let omega_i = Self::get_random_dir(n);

            let kd = material.diffuse;
            let ks = material.specular;
            let s = material.shininess;
            let lambertian = kd * std::f32::consts::FRAC_1_PI;
            let brdf = lambertian
                + ((ks * (s + 2.0) * r.dot(omega_i).powf(s)) * std::f32::consts::FRAC_1_PI) / 2.0;

            let cosin_law = omega_i.dot(n);

            let next_ray = Ray::new(next_origin, omega_i);
            if let Some(indirect_sample) =
                Self::trace_impl(config, scene, next_ray, tlas, depth + 1, collect_emissive)
            {
                indirect_lighting += brdf * cosin_law * indirect_sample;
            }
        }

        (2.0 * std::f32::consts::PI * indirect_lighting) / (config.light_samples as f32)
    }

    fn trace_impl(
        config: &Config,
        scene: &SceneDrawInfo,
        ray: Ray,
        tlas: &Tlas,
        depth: u32,
        collect_emissive: bool,
    ) -> Option<Color> {
        if depth >= config.max_depth {
            return None;
        }

        let hit = tlas.intersects(scene, &ray)?;

        let primitive = tlas.get_primitive(&hit);

        // This is the color of the primitive with no light
        let ambient_and_emissive = primitive.get_color(scene, &hit);

        if collect_emissive && primitive.is_emissive(scene) {
            return Some(ambient_and_emissive);
        }

        // Normal at the hit point
        let n = primitive.get_normal(scene, &hit);

        // Reflection direction
        let r = ray.dir.reflect(&n).get_normalized();

        let material = primitive.get_phong_material(scene);

        let mut direct_lighting = Color::BLACK;
        if config.next_event_estimation {
            direct_lighting = Self::get_direct_lighting(config, scene, tlas, &hit, n, r, material);
        }

        let mut indirect_lighting = Color::BLACK;

        let indirect_depth_limit = if config.next_event_estimation {
            config.max_depth - 1
        } else {
            config.max_depth
        };

        if depth < indirect_depth_limit {
            indirect_lighting +=
                Self::get_indirect_lighting(config, scene, tlas, &hit, n, r, material, depth);
        }

        Some(direct_lighting + indirect_lighting)
    }
}

impl Integrator for Pathtracer {
    fn trace(
        &self,
        config: &Config,
        scene: &SceneDrawInfo,
        ray: Ray,
        tlas: &Tlas,
        depth: u32,
    ) -> Option<Color> {
        Self::trace_impl(config, scene, ray, tlas, depth, true)
    }
}
