// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

#[derive(Default)]
pub struct BrdfSampler {}

impl BrdfSampler {
    pub const fn new() -> Self {
        Self {}
    }

    fn get_t(&self, material: &PhongMaterial) -> f32 {
        let kd_avg = material.diffuse.get_rgb().reduce_avg();
        let ks_avg = material.specular.get_rgb().reduce_avg();
        ks_avg / (ks_avg + kd_avg)
    }
}

impl SoftSampler for BrdfSampler {
    /// - `material`: The material properties.
    /// - `n`: Surface normal
    /// - `r`: Reflection vector
    fn get_random_dir(&self, material: &PhongMaterial, n: Vec3, r: Vec3) -> Vec3 {
        let e0 = fastrand::f32();
        let e1 = fastrand::f32();
        let e2 = fastrand::f32();

        let t = self.get_t(material);
        let s = material.shininess;

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
        let w = if e0 <= t { r } else { n };
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
    /// - `brdf`: The BRDF color.
    /// - `n`: The normal vector at the point of intersection.
    /// - `omega_i`: The incoming direction.
    /// - `li`: The indirect light color.
    /// - `weight`: The weight for the sample.
    fn get_radiance(
        &self,
        material: &PhongMaterial,
        n: Vec3,
        _r: Vec3,
        omega_i: Vec3,
        li: Color,
        weight: f32,
    ) -> Color {
        let cosine_law = n.dot(omega_i).clamp(0.0, 1.0);
        let cd = material.diffuse;
        let cs = material.specular
            * (cosine_law * (material.shininess + 2.0) / (material.shininess + 1.0)).min(1.0);
        // Boost factor applies to the returned radiance as well
        li * (cd + cs) * weight
    }
}
