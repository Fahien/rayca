// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

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

/// - `material`: The material properties.
/// - `n`: Surface normal
/// - `r`: Reflection vector
pub fn get_random_dir(hit: &mut HitInfo) -> Vec3 {
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

pub fn get_specular_component(hit: &mut HitInfo, omega: Vec3) -> Color {
    let s = hit.get_shininess();
    let n = hit.get_normal();
    let n_dot_omega = n.dot(omega).clamp(0.0, 1.0);
    let ks = hit.get_specular();
    ks * n_dot_omega * (s + 2.0) / (s + 1.0)
}

pub fn get_radiance(hit: &mut HitInfo, ir: Irradiance) -> Color {
    let diffuse = hit.get_diffuse() * ir.n_dot_l;
    let specular = hit.get_specular() * ir.n_dot_h.powf(hit.get_shininess());
    (diffuse + specular) * ir.intensity
}
