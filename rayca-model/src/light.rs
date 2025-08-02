// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::simd::num::SimdFloat;

use crate::*;

pub enum Light {
    Directional(DirectionalLight),
    Point(PointLight),
}

impl Light {
    pub fn directional() -> Self {
        Self::Directional(DirectionalLight::default())
    }

    pub fn point() -> Self {
        Self::Point(PointLight::new())
    }

    pub fn set_intensity(&mut self, intensity: f32) {
        match self {
            Light::Directional(light) => light.set_intensity(intensity),
            Light::Point(light) => light.set_intensity(intensity),
        }
    }

    pub fn get_distance(&self, light_trs: &Trs, frag_pos: &Point3) -> f32 {
        match self {
            Light::Directional(light) => light.get_distance(light_trs, frag_pos),
            Light::Point(light) => light.get_distance(light_trs, frag_pos),
        }
    }

    pub fn get_intensity(&self, light_trs: &Trs, frag_pos: &Point3) -> Color {
        match self {
            Light::Directional(light) => light.get_intensity(),
            Light::Point(light) => light.get_intensity(light_trs, frag_pos),
        }
    }

    pub fn get_fallof(&self, light_trs: &Trs, frag_pos: &Point3) -> f32 {
        match self {
            Light::Directional(light) => light.get_fallof(),
            Light::Point(light) => light.get_fallof(light_trs, frag_pos),
        }
    }

    /// Returns the direction vector from the fragment position to the light world position
    pub fn get_direction(&self, light_trs: &Trs, frag_pos: &Point3) -> Vec3 {
        match self {
            Light::Directional(light) => light.get_direction(light_trs),
            Light::Point(light) => light.get_direction(light_trs, frag_pos),
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

    pub fn set_intensity(&mut self, intensity: f32) {
        self.intensity = intensity;
    }

    pub fn get_distance(&self, light_trs: &Trs, frag_pos: &Point3) -> f32 {
        let dist = frag_pos - light_trs.get_translation();
        Vec3::from(dist).len()
    }

    pub fn get_intensity(&self, light_trs: &Trs, frag_pos: &Point3) -> Color {
        (self.intensity * self.color) / self.get_fallof(light_trs, frag_pos)
    }

    pub fn get_fallof(&self, light_trs: &Trs, frag_pos: &Point3) -> f32 {
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
