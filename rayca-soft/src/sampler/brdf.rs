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
        hit.get_pdf(sample.get_omega())
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
        let pdf = hit.get_pdf(omega);
        let mut x = Color::BLACK;

        let shadow_ray = hit.get_next_ray(omega);
        if let Some(mut shadow_hit) = hit.tlas.intersects(hit.scene, shadow_ray) {
            if shadow_hit.is_emissive() {
                // This calculation already includes division by pdf
                let li = shadow_hit.get_emission();
                let cd = hit.get_diffuse();
                let cs = hit.get_specular_component(omega);
                x = li * (cd + cs);
            }
        }

        BrdfSample::new(omega, x, pdf)
    }
}

impl SoftSampler for BrdfSampler {
    fn get_random_dir(&self, hit: &mut HitInfo) -> Vec3 {
        // This varies according to the material model
        hit.get_random_dir()
    }

    /// Returns the indirect lighting for the given parameters.
    /// - `hit`: The hit information.
    /// - `omega_i`: The incoming direction.
    /// - `indirect_sample`: The indirect sample color.
    /// - `weight`: The weight for the sample.
    fn get_radiance(
        &self,
        hit: &mut HitInfo,
        omega_i: Vec3,
        indirect_sample: Color,
        weight: f32,
    ) -> Color {
        let cd = hit.get_diffuse();
        let cs = hit.get_specular_component(omega_i);
        // Boost factor applies to the returned radiance as well
        indirect_sample * (cd + cs) * weight
    }
}
