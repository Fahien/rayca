// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::{sample::Sample, *};

pub struct CosineSample {
    omega: Vec3,
    x: Color,
}

impl CosineSample {
    fn new(omega: Vec3, x: Color) -> Self {
        Self { omega, x }
    }
}

impl Sample for CosineSample {
    fn get_omega(&self) -> Vec3 {
        self.omega
    }

    fn get_x(&self) -> Color {
        self.x
    }

    fn get_pdf(&self) -> f32 {
        std::f32::consts::FRAC_1_PI / 2.0
    }

    fn get_pdf_for(&self, _hit: &mut HitInfo<'_>, _sample: &dyn Sample) -> f32 {
        self.get_pdf()
    }
}

#[derive(Default)]
pub struct CosineSampler {}

impl CosineSampler {
    pub const fn new() -> Self {
        Self {}
    }

    pub fn sample_direct(&self, hit: &mut HitInfo) -> CosineSample {
        let n = hit.get_normal();
        let omega_i = self.get_random_dir(hit);
        let shadow_ray = hit.get_next_ray(omega_i);
        let mut x = Color::BLACK;
        if let Some(mut shadow_hit) = hit.tlas.intersects(hit.scene, shadow_ray) {
            if shadow_hit.is_emissive() {
                let li = shadow_hit.get_emission();
                let brdf = hit.get_brdf(omega_i);
                let n_dot_omega_i = n.dot(omega_i).clamp(0.0, 1.0);
                let frac_1_pdf = 2.0 * std::f32::consts::PI;
                x = li * brdf * n_dot_omega_i * frac_1_pdf;
            }
        };
        CosineSample::new(omega_i, x)
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
        let brdf = hit.get_brdf(omega_i);
        std::f32::consts::PI * brdf * indirect_sample * weight
    }
}
