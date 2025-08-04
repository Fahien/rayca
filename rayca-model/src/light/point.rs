// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::simd::num::SimdFloat;

use crate::*;

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
