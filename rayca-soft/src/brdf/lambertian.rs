// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

pub fn get_radiance(material: &PhongMaterial, ir: &Irradiance) -> Color {
    let diffuse = material.diffuse * ir.n_dot_l;
    let specular = material.specular * ir.n_dot_h.powf(material.shininess);
    (diffuse + specular) * ir.intensity
}

pub fn get_brdf(material: &PhongMaterial, r: Vec3, omega_i: Vec3) -> Color {
    use std::f32::consts::FRAC_1_PI;
    let lambertian = material.diffuse * FRAC_1_PI;
    let s = material.shininess;
    let specular = (material.specular * (s + 2.0) * r.dot(omega_i).powf(s) * FRAC_1_PI) / 2.0;
    lambertian + specular
}
