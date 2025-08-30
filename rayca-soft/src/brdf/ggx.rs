// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

/// Models the distribution of the microfacet
/// Surfaces are not smooth at the micro level, but made of a
/// large number of randomly aligned planar surface fragments.
/// This implementation is good for half-precision floats.
fn distribution_ggx(n_dot_h: f32, roughness: f32) -> f32 {
    let a = n_dot_h * roughness;
    let k = roughness / (1.0 - n_dot_h * n_dot_h + a * a);
    k * k * std::f32::consts::FRAC_1_PI
}

/// The amount of light the viewer sees reflected from a surface depends on the
/// viewing angle, in fact at grazing angles specular reflections become more intense
fn fresnel_schlick(cos_theta: f32, f0: Vec3) -> Vec3 {
    let f = (1.0 - cos_theta).powf(5.0);
    f0 + (Vec3::splat(1.0) - f0) * f
}

/// Models the visibility of the microfacets, or occlusion or shadow-masking
fn geometry_smith_ggx(n_dot_v: f32, n_dot_l: f32, roughness: f32) -> f32 {
    let a = roughness;
    let ggxv = n_dot_l * (n_dot_v * (1.0 - a) + a);
    let ggxl = n_dot_v * (n_dot_l * (1.0 - a) + a);
    0.5 / (ggxv + ggxl)
}

pub fn get_radiance(hit: &mut HitInfo, ir: Irradiance) -> Color {
    let (metallic, roughness) = hit.get_metallic_roughness();

    let d = distribution_ggx(ir.n_dot_h, roughness);

    let albedo = hit.get_color();
    let f0 = Vec3::splat(0.04) * (1.0 - metallic) + Vec3::from(&albedo) * metallic;
    let f = fresnel_schlick(ir.l_dot_h, f0);

    let ks = f;
    let kd = (Vec3::splat(1.0) - ks) * (1.0 - metallic);

    let g = geometry_smith_ggx(ir.n_dot_v, ir.n_dot_l, roughness);

    let fr = (d * g) * Color::from(f);

    // Lambertian diffuse (1/PI)
    let fd = Color::from(kd) * albedo * std::f32::consts::FRAC_1_PI;

    (fd + fr) * ir.intensity * ir.n_dot_l
}

/// [GGX Microfacet distribution function](https://www.cs.cornell.edu/~srm/publications/EGSR07-btdf.pdf)
/// - a: roughness
/// - h: half vector
/// - n: surface normal
fn get_d(a: f32, h: Vec3, n: Vec3) -> f32 {
    let a_squared = a * a;
    let cos_theta = h.dot(n).clamp(0.0, 1.0);
    let theta = cos_theta.acos();
    let denominator = cos_theta.powf(4.0) * (a_squared + theta.tan().powf(2.0)).powf(2.0);
    if denominator == 0.0 {
        return 0.0;
    }
    a_squared * std::f32::consts::FRAC_1_PI / denominator
}

/// Monodirectional shadow-masking term.
/// It depends on D, meaning that each D has its own G1.
/// - a: roughness
/// - omega: either incoming or outgoing direction
/// - n: surface normal
fn get_g1(a: f32, omega: Vec3, n: Vec3) -> f32 {
    let cos_theta = omega.dot(n);
    if cos_theta <= 0.0 {
        0.0
    } else {
        let theta = cos_theta.acos();
        let denominator = 1.0 + (1.0 + a * a * theta.tan().powf(2.0)).sqrt();
        2.0 / denominator
    }
}

/// Smith shadow masking approximation function
/// - a: roughness
/// - n: surface normal
/// - omega_i: incoming direction
/// - omega_o: outgoing direction
pub fn get_g(a: f32, n: Vec3, omega_i: Vec3, omega_o: Vec3) -> f32 {
    get_g1(a, omega_i, n) * get_g1(a, omega_o, n)
}

/// Fresnel term [Schlick's approximation](https://en.wikipedia.org/wiki/Schlick%27s_approximation).
/// Small at normal incidence and increases to unity at grazing angles.
/// - ks: specular color
/// - omega_i: incoming direction
/// - h: half vector
pub fn get_f(ks: Color, omega_i: Vec3, h: Vec3) -> Color {
    let omega_i_dot_h = omega_i.dot(h).abs();
    ks + (Color::WHITE - ks) * (1.0 - omega_i_dot_h).powf(5.0)
}

/// Specular BRDF (BsDF?)
pub fn get_bsdf(hit: &mut HitInfo, omega_i: Vec3) -> Color {
    let omega_o = hit.get_view();
    let n = hit.get_normal();
    let omega_i_dot_n = omega_i.dot(n).clamp(0.0, 1.0);
    let omega_o_dot_n = omega_o.dot(n).clamp(0.0, 1.0);
    if omega_i_dot_n == 0.0 || omega_o_dot_n == 0.0 {
        return Color::BLACK;
    }

    let ks = hit.get_specular();
    let a = hit.get_roughness();
    let h = (omega_i + omega_o).get_normalized();
    let f = get_f(ks, omega_i, h);
    let g = get_g(a, n, omega_i, omega_o);
    let d = get_d(a, h, n);
    let denominator = 4.0 * omega_i_dot_n * omega_o_dot_n;
    f * g * d / denominator
}

pub fn get_brdf(hit: &mut HitInfo, omega_i: Vec3) -> Color {
    let kd = hit.get_diffuse();
    kd * std::f32::consts::FRAC_1_PI + get_bsdf(hit, omega_i)
}

pub fn get_pdf(hit: &mut HitInfo, omega_i: Vec3) -> f32 {
    let omega_o = hit.get_view();
    // Half vector
    let h = (omega_o + omega_i).get_normalized();
    let h_dot_omega_i = h.dot(omega_i).clamp(0.0, 1.0);
    if h_dot_omega_i == 0.0 {
        return 0.0;
    }
    let n = hit.get_normal();
    let n_dot_h = n.dot(h).clamp(0.0, 1.0);
    let spec = get_d(hit.get_roughness(), h, n) * n_dot_h / (4.0 * h_dot_omega_i);
    // diffuse pdf
    let dif = n.dot(omega_i).clamp(0.0, 1.0) * std::f32::consts::FRAC_1_PI;
    let t = hit.get_t();
    (1.0 - t) * dif + t * spec
}

pub fn get_random_dir(hit: &mut HitInfo) -> Vec3 {
    let e0 = fastrand::f32();
    let e1 = fastrand::f32();
    let e2 = fastrand::f32();

    let t = hit.get_t();
    let a = hit.get_roughness();

    let theta = if e0 <= t {
        ((a * e1.sqrt()) / (1.0 - e1).sqrt()).atan() // specular
    } else {
        e1.sqrt().clamp(-1.0, 1.0).acos() // diffuse
    };
    let omega = 2.0 * std::f32::consts::PI * e2;

    let s = Vec3::new(
        omega.cos() * theta.sin(),
        omega.sin() * theta.sin(),
        theta.cos(),
    );

    // We need to rotate s so that it is centered around n
    let w = hit.get_normal();
    let a = if w.close(Vec3::Y_AXIS) {
        Vec3::X_AXIS
    } else {
        Vec3::Y_AXIS
    };
    let u = a.cross(w).get_normalized();
    let v = w.cross(u);

    let s = s.get_x() * u + s.get_y() * v + s.get_z() * w;

    if e0 <= t {
        // For the specular component, s that we generated is the "half vector".
        // To get a omega_i, we need to reflect omega_o off s.
        let omega_o = hit.get_view();
        (-omega_o).reflect(&s)
    } else {
        s
    }
}

pub fn get_specular_component(hit: &mut HitInfo, omega_i: Vec3) -> Color {
    let omega_o = hit.get_view();
    let n = hit.get_normal();
    let omega_o_dot_n = omega_o.dot(n).clamp(0.0, 1.0);
    let h = (omega_i + omega_o).get_normalized();
    let n_dot_h = n.dot(h).clamp(0.0, 1.0);
    if omega_o_dot_n == 0.0 || n_dot_h == 0.0 {
        return Color::BLACK;
    }
    let ks = hit.get_specular();
    let a = hit.get_roughness();
    let h_dot_omega_i = h.dot(omega_i).clamp(0.0, 1.0);
    get_f(ks, omega_i, h) * get_g(a, n, omega_i, omega_o) * h_dot_omega_i
        / (omega_o_dot_n * n_dot_h)
}
