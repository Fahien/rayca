// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

#[derive(Default)]
pub struct CosineSampler {}

impl CosineSampler {
    pub const fn new() -> Self {
        Self {}
    }
}

impl SoftSampler for CosineSampler {
    /// Returns a random direction in the cosine-weighted hemisphere
    /// centered around the normal `n`.
    fn get_random_dir(&self, hit: &mut HitInfo) -> Vec3 {
        let e1 = fastrand::f32();
        let e2 = fastrand::f32();

        let theta = e1.sqrt().acos();
        let omega = 2.0 * std::f32::consts::PI * e2;

        let s = Vec3::new(
            omega.cos() * theta.sin(),
            omega.sin() * theta.sin(),
            theta.cos(),
        );
        // We need to rotate s so that the emisphere is centered around n
        let w = hit.get_normal();
        let a = if w.close(Vec3::Y_AXIS) {
            Vec3::X_AXIS
        } else {
            Vec3::Y_AXIS
        };
        let u = a.cross(w).get_normalized();
        let v = w.cross(u);

        s.get_x() * u + s.get_y() * v + s.get_z() * w
    }

    fn get_radiance(
        &self,
        hit: &mut HitInfo,
        omega_i: Vec3,
        indirect_sample: Color,
        weight: f32,
    ) -> Color {
        let brdf = lambertian::get_brdf(hit, omega_i);
        std::f32::consts::PI * brdf * indirect_sample * weight
    }
}
