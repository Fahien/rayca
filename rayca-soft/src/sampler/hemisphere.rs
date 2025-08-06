// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

pub struct HemisphereSampler {}

impl HemisphereSampler {
    pub const fn new() -> Self {
        Self {}
    }
}

impl SoftSampler for HemisphereSampler {
    /// Returns a random direction in the hemisphere centered around the normal `n`.
    fn get_random_dir(&self, _material: &PhongMaterial, n: Vec3, _r: Vec3) -> Vec3 {
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

    fn get_radiance(
        &self,
        material: &PhongMaterial,
        n: Vec3,
        r: Vec3,
        omega_i: Vec3,
        indirect_sample: Color,
        weight: f32,
    ) -> Color {
        let brdf = lambertian::get_brdf(material, r, omega_i);
        let cosine_law = n.dot(omega_i).clamp(0.0, 1.0);
        2.0 * std::f32::consts::PI * brdf * cosine_law * indirect_sample * weight
    }
}
