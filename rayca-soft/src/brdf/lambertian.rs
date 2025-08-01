// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

pub fn get_radiance(material: &PhongMaterial, ir: &Irradiance) -> Color {
    let diffuse = material.diffuse * ir.n_dot_l;
    let specular = material.specular * ir.n_dot_h.powf(material.shininess);
    (diffuse + specular) * ir.intensity
}
