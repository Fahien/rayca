// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

pub fn get_radiance(hit: &mut HitInfo, ir: Irradiance) -> Color {
    let diffuse = hit.get_diffuse() * ir.n_dot_l;
    let specular = hit.get_specular() * ir.n_dot_h.powf(hit.get_shininess());
    (diffuse + specular) * ir.intensity
}

pub fn get_brdf(hit: &mut HitInfo, omega_i: Vec3) -> Color {
    use std::f32::consts::FRAC_1_PI;
    let lambertian = hit.get_diffuse() * FRAC_1_PI;
    let s = hit.get_shininess();
    let specular =
        (hit.get_specular() * (s + 2.0) * hit.get_reflection().dot(omega_i).powf(s) * FRAC_1_PI)
            / 2.0;
    lambertian + specular
}

pub fn get_pdf(hit: &mut HitInfo, omega: Vec3) -> f32 {
    // specular pdf
    let r_dot_omega = hit.get_reflection().dot(omega).clamp(0.0, 1.0);
    let s = hit.get_shininess();
    let spec = (s + 1.0) * std::f32::consts::FRAC_2_PI * r_dot_omega.powf(s);
    // diffuse pdf
    let diff = hit.get_normal().dot(omega).clamp(0.0, 1.0) * std::f32::consts::FRAC_1_PI;
    let t = hit.get_t();
    (1.0 - t) * diff + t * spec
}
