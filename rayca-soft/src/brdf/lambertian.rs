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
