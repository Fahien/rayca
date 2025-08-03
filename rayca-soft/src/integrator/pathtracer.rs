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

impl Integrator for Pathtracer {
    fn trace(
        &self,
        config: &Config,
        scene: &SceneDrawInfo,
        ray: Ray,
        tlas: &Tlas,
        depth: u32,
    ) -> Option<Color> {
        if depth >= config.max_depth {
            return None;
        }

        let hit = tlas.intersects(scene, &ray)?;

        let primitive = tlas.get_primitive(&hit);

        // This is the color of the primitive with no light
        let ambient_and_emissive = primitive.get_color(scene, &hit);

        if primitive.is_emissive(scene) {
            return Some(ambient_and_emissive);
        }

        // Ray generation for hemisphere sampling
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
        let n = primitive.get_normal(scene, &hit);
        let w = n;
        let a = if w.close(Vec3::Y_AXIS) {
            Vec3::X_AXIS
        } else {
            Vec3::Y_AXIS
        };
        let u = a.cross(w).get_normalized();
        let v = w.cross(u).get_normalized();

        let omega_i = s.get_x() * u + s.get_y() * v + s.get_z() * w;

        let uv = primitive.get_uv(&hit);
        let kd = primitive.get_diffuse(scene, &hit, uv);
        let lambertian = kd * std::f32::consts::FRAC_1_PI;
        let brdf = lambertian;

        let cosin_law = omega_i.dot(n);

        let next_ray = Ray::new(hit.point + n * Self::RAY_BIAS, omega_i);
        let light_contribution = 2.0
            * std::f32::consts::PI
            * brdf
            * cosin_law
            * self.trace(config, scene, next_ray, tlas, depth + 1)?;

        Some(light_contribution)
    }
}
