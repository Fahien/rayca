// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::simd::num::SimdFloat;

use crate::*;

pub enum Light {
    Directional(DirectionalLight),
    Point(PointLight),
    Quad(QuadLight),
}

impl Light {
    pub fn directional() -> Self {
        Self::Directional(DirectionalLight::default())
    }

    pub fn point() -> Self {
        Self::Point(PointLight::new())
    }

    pub fn is_quad(&self) -> bool {
        matches!(self, Light::Quad(_))
    }

    /// Returns the intensity of light, which is affected by the distance of the
    /// observer from the source, as well as other factors such as the angle at
    /// which the light hits the observer and the amount of light that is absorbed
    /// or scattered by objects in the environment.
    pub fn set_intensity(&mut self, intensity: f32) {
        match self {
            Light::Directional(light) => light.set_intensity(intensity),
            Light::Point(light) => light.set_intensity(intensity),
            Light::Quad(_) => todo!(),
        }
    }

    pub fn get_distance(&self, light_trs: &Trs, frag_pos: &Point3) -> f32 {
        match self {
            Light::Directional(light) => light.get_distance(light_trs, frag_pos),
            Light::Point(light) => light.get_distance(light_trs, frag_pos),
            Light::Quad(_) => todo!(),
        }
    }

    pub fn get_intensity(&self, light_trs: &Trs, frag_pos: Point3, frag_n: Vec3) -> Color {
        match self {
            Light::Directional(light) => light.get_intensity(),
            Light::Point(light) => light.get_intensity(light_trs, frag_pos, frag_n),
            Light::Quad(light) => light.get_intensity(light_trs, frag_pos, frag_n),
        }
    }

    pub fn get_radiance(&self, light_trs: &Trs, frag_pos: Point3) -> Vec3 {
        match self {
            Light::Directional(light) => light.get_radiance(),
            Light::Point(light) => light.get_radiance(),
            Light::Quad(light) => light.get_radiance(light_trs, frag_pos),
        }
    }

    pub fn get_fallof(&self, light_trs: &Trs, frag_pos: Point3) -> f32 {
        match self {
            Light::Directional(light) => light.get_fallof(),
            Light::Point(light) => light.get_fallof(light_trs, frag_pos),
            Light::Quad(light) => light.get_fallof(),
        }
    }

    /// Returns the direction vector from the fragment position to the light world position
    pub fn get_direction(&self, light_trs: &Trs, frag_pos: &Point3) -> Vec3 {
        match self {
            Light::Directional(light) => light.get_direction(light_trs),
            Light::Point(light) => light.get_direction(light_trs, frag_pos),
            Light::Quad(_) => todo!(),
        }
    }
}

pub struct DirectionalLight {
    color: Color,
    intensity: f32,
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self {
            color: Color::new(1.0, 1.0, 1.0, 1.0),
            intensity: 1.0,
        }
    }
}

impl DirectionalLight {
    pub fn new(color: Color, intensity: f32) -> Self {
        Self { color, intensity }
    }

    pub fn get_radiance(&self) -> Vec3 {
        self.intensity * Vec3::from(self.color)
    }

    pub fn set_intensity(&mut self, intensity: f32) {
        self.intensity = intensity;
    }

    pub fn get_distance(&self, light_trs: &Trs, frag_pos: &Point3) -> f32 {
        let dist = Vec3::from(frag_pos) - light_trs.get_translation();
        dist.len()
    }

    pub fn get_intensity(&self) -> Color {
        self.intensity * self.color
    }

    pub fn get_fallof(&self) -> f32 {
        1.0
    }

    pub fn get_direction(&self, light_trs: &Trs) -> Vec3 {
        let mut light_dir = Vec3::new(1.0, 0.0, 0.0);
        light_dir.rotate(light_trs.rotation);
        -light_dir
    }
}

pub struct PointLight {
    pub color: Color,
    pub intensity: f32,
    pub attenuation: Vec3,
}

impl PointLight {
    pub fn new() -> Self {
        Self {
            color: Color::new(1.0, 1.0, 1.0, 1.0),
            intensity: 1.0,
            attenuation: Vec3::new(0.0, 0.0, 1.0),
        }
    }

    pub fn get_radiance(&self) -> Vec3 {
        self.intensity * Vec3::from(self.color)
    }

    pub fn set_intensity(&mut self, intensity: f32) {
        self.intensity = intensity;
    }

    pub fn get_distance(&self, light_trs: &Trs, frag_pos: &Point3) -> f32 {
        let dist = frag_pos - light_trs.get_translation();
        Vec3::from(dist).len()
    }

    pub fn get_intensity(&self, light_trs: &Trs, frag_pos: Point3, _frag_n: Vec3) -> Color {
        (self.intensity * self.color) / self.get_fallof(light_trs, frag_pos)
    }

    pub fn get_fallof(&self, light_trs: &Trs, frag_pos: Point3) -> f32 {
        let dist = Vec3::from(frag_pos) - light_trs.get_translation();
        let r2 = dist.norm();
        let r = r2.sqrt();

        let rhs = Vec3::new(1.0, r, r2);
        (self.attenuation.simd * rhs.simd).reduce_sum()
    }

    pub fn get_direction(&self, light_trs: &Trs, frag_pos: &Point3) -> Vec3 {
        let mut dist = Vec3::from(frag_pos) - light_trs.get_translation();
        dist.normalize();
        -dist
    }
}

impl Default for PointLight {
    fn default() -> Self {
        Self::new()
    }
}

/// ```text
///    c  /‾‾‾‾‾/ d
///      /     /
///  ac /     /
///    /     /
/// a /_____/ b
///     ab
/// ```
#[derive(Default)]
pub struct QuadLight {
    pub ab: Vec3,
    pub ac: Vec3,
    pub color: Color,
    pub intensity: f32,
    pub material: Handle<Material>,
}

impl QuadLight {
    pub fn new(ab: Vec3, ac: Vec3, color: Color, material: Handle<Material>) -> Self {
        Self {
            ab,
            ac,
            color,
            intensity: 1.0,
            material,
        }
    }

    /// Returns the normal of the quad light in model space.
    pub fn get_normal(&self) -> Vec3 {
        self.ab.cross(self.ac).get_normalized()
    }

    pub fn get_area(&self) -> f32 {
        let ab_len = self.ab.len();
        let ac_len = self.ac.len();
        let cos_theta = self.ab.dot(self.ac) / (ab_len * ac_len);
        let sin_theta = 1.0 - cos_theta;
        sin_theta * ab_len * ac_len
    }

    pub fn get_a(&self, trs: &Trs, edge_index: u32) -> Point3 {
        let a: Point3 = trs.get_translation().into();
        match edge_index {
            0 => a,
            1 => a + self.ab,
            2 => a + self.ab + self.ac,
            3 => a + self.ac,
            _ => panic!("Quad light edge index out of bounds"),
        }
    }

    pub fn get_b(&self, trs: &Trs, edge_index: u32) -> Point3 {
        let a: Point3 = trs.get_translation().into();
        match edge_index {
            0 => a + self.ab,
            1 => a + self.ab + self.ac,
            2 => a + self.ac,
            3 => a,
            _ => panic!("Quad light edge index out of bounds"),
        }
    }

    /// Returns the angle subtended by the `i`th edge of the quad as seen by `frag_pos`
    pub fn theta(&self, trs: &Trs, edge_index: u32, frag_pos: Point3) -> f32 {
        // Frag pos is `r`, `ra` is unit vector from `r` to `a`.
        let ra = (self.get_a(trs, edge_index) - frag_pos).get_normalized();
        // Frag pos is `r`, `rb` is unit vector from `r` to `b`.
        let rb = (self.get_b(trs, edge_index) - frag_pos).get_normalized();
        (ra.dot(rb)).acos()
    }

    /// Returns the unit normal of the polygonal cone with cross section this quad and apex at `frag_pos`
    pub fn gamma(&self, trs: &Trs, edge_index: u32, frag_pos: Point3) -> Vec3 {
        let ra = self.get_a(trs, edge_index) - frag_pos;
        let rb = self.get_b(trs, edge_index) - frag_pos;
        ra.cross(rb).get_normalized()
    }

    /// Returns the intensity of quad light reaching that point, at a certain angle.
    pub fn get_intensity(&self, trs: &Trs, frag_pos: Point3, frag_n: Vec3) -> Color {
        let omega = self.get_radiance(trs, frag_pos);
        let irradiance = omega.dot(frag_n);
        self.intensity * self.color * irradiance
    }

    pub fn get_radiance(&self, trs: &Trs, frag_pos: Point3) -> Vec3 {
        let mut ret = Vec3::default();
        for edge_index in 0..4 {
            ret += self.theta(trs, edge_index, frag_pos) * self.gamma(trs, edge_index, frag_pos);
        }
        ret /= 2.0;
        ret
    }

    pub fn get_fallof(&self) -> f32 {
        1.0
    }
}

impl Default for Light {
    fn default() -> Self {
        Self::Directional(DirectionalLight::default())
    }
}

/// Helper structure which should simplify drawing function interfaces
pub struct Irradiance<'m> {
    // Intensity of incoming light
    pub intensity: Color,

    /// Hit
    pub hit: &'m Hit,

    /// Surface normal
    pub n: Vec3,
    pub n_dot_v: f32,
    pub n_dot_l: f32,

    /// Half-angle (direction between ray and light)
    pub h: Vec3,
    pub n_dot_h: f32,
    pub l_dot_h: f32,

    /// Albedo color
    pub albedo: Color,
    pub uv: Vec2,
}

impl<'m> Irradiance<'m> {
    /// - l: light direction
    /// - n: normal to the surface
    /// - v: view direction
    pub fn new(
        intensity: Color,
        hit: &'m Hit,
        l: Vec3,
        n: Vec3,
        v: Vec3,
        albedo: Color,
        uv: Vec2,
    ) -> Self {
        let n_dot_v = n.dot(&v).clamp(0.0, 1.0) + 1e-5;
        let n_dot_l = n.dot(&l).clamp(0.0, 1.0);
        let h = (v + l).get_normalized();
        let n_dot_h = n.dot(&h).clamp(0.0, 1.0);
        let l_dot_h = l.dot(&h).clamp(0.0, 1.0);

        Self {
            intensity,
            hit,
            n,
            n_dot_v,
            n_dot_l,
            h,
            n_dot_h,
            l_dot_h,
            albedo,
            uv,
        }
    }
}
