// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::{sample::Sample, *};

pub struct BrdfSample {
    omega: Vec3,

    /// Color towards outgoing direction
    x: Color,
    pdf: f32,
}

impl BrdfSample {
    fn new(omega: Vec3, x: Color, pdf: f32) -> Self {
        Self { omega, x, pdf }
    }
}

impl Sample for BrdfSample {
    fn get_omega(&self) -> Vec3 {
        self.omega
    }

    fn get_x(&self) -> Color {
        self.x
    }

    fn get_pdf(&self) -> f32 {
        self.pdf
    }

    fn get_pdf_for(&self, hit: &mut HitInfo, sample: &dyn Sample) -> f32 {
        lambertian::get_pdf(hit, sample.get_omega())
    }
}

#[derive(Default)]
pub struct BrdfSampler {}

impl BrdfSampler {
    pub const fn new() -> Self {
        Self {}
    }

    pub fn sample_direct(&self, hit: &mut HitInfo) -> BrdfSample {
        // The direct lighting algorithm is not recursive
        let omega = self.get_random_dir(hit);
        let pdf = lambertian::get_pdf(hit, omega);
        let mut x = Color::BLACK;

        let shadow_ray = hit.get_next_ray(omega);
        if let Some(mut shadow_hit) = hit.tlas.intersects(hit.scene, shadow_ray) {
            if shadow_hit.is_emissive() {
                // This calculation already includes division by pdf
                let li = shadow_hit.get_emission();
                let cd = hit.get_diffuse();
                let cs = hit.get_specular();
                x = li * (cd + cs);
            }
        }

        BrdfSample::new(omega, x, pdf)
    }
}

impl SoftSampler for BrdfSampler {
    /// - `material`: The material properties.
    /// - `n`: Surface normal
    /// - `r`: Reflection vector
    fn get_random_dir(&self, hit: &mut HitInfo) -> Vec3 {
        let e0 = fastrand::f32();
        let e1 = fastrand::f32();
        let e2 = fastrand::f32();

        let t = hit.get_t();
        let s = hit.get_shininess();

        let theta = if e0 <= t {
            e1.powf(1.0 / (s + 1.0)).clamp(-1.0, 1.0).acos()
        } else {
            e1.sqrt().clamp(-1.0, 1.0).acos()
        };
        let omega = 2.0 * std::f32::consts::PI * e2;

        let s = Vec3::new(
            omega.cos() * theta.sin(),
            omega.sin() * theta.sin(),
            theta.cos(),
        );

        // We need to rotate s so that it is centered around n (or r)
        let w = if e0 <= t {
            hit.get_reflection()
        } else {
            hit.get_normal()
        };
        let a = if w.close(Vec3::Y_AXIS) {
            Vec3::X_AXIS
        } else {
            Vec3::Y_AXIS
        };
        let u = a.cross(w).get_normalized();
        let v = w.cross(u);

        s.get_x() * u + s.get_y() * v + s.get_z() * w
    }

    /// Returns the radiance for the given parameters.
    /// - `hit`: The hit information.
    /// - `omega_i`: The incoming direction.
    /// - `li`: The indirect light color.
    /// - `weight`: The weight for the sample.
    fn get_radiance(&self, hit: &mut HitInfo, omega_i: Vec3, li: Color, weight: f32) -> Color {
        let cosine_law = hit.get_normal().dot(omega_i).clamp(0.0, 1.0);
        let cd = hit.get_diffuse();
        let cs = hit.get_specular()
            * (cosine_law * (hit.get_shininess() + 2.0) / (hit.get_shininess() + 1.0)).min(1.0);
        // Boost factor applies to the returned radiance as well
        li * (cd + cs) * weight
    }
}
