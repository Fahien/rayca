// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use super::*;

pub enum LightType {
    Distant,
    Point,
}

pub struct Light {
    color: Color,
    intensity: f32,
    pub _type: LightType,
}

impl Light {
    pub fn distant() -> Self {
        Self {
            color: Color::new(1.0, 1.0, 1.0, 1.0),
            intensity: 1.0,
            _type: LightType::Distant,
        }
    }

    pub fn point() -> Self {
        Self {
            color: Color::new(1.0, 1.0, 1.0, 1.0),
            intensity: 1.0,
            _type: LightType::Point,
        }
    }

    pub fn set_intensity(&mut self, intensity: f32) {
        self.intensity = intensity;
    }

    pub fn get_distance(&self, light_trs: &Trs, frag_pos: &Point3) -> f32 {
        let dist = frag_pos - light_trs.get_translation();
        Vec3::from(dist).len()
    }

    pub fn get_intensity(&self) -> Color {
        self.intensity * self.color
    }

    pub fn get_fallof(&self, light_trs: &Trs, frag_pos: &Point3) -> f32 {
        match self._type {
            LightType::Distant => 1.0,
            LightType::Point => {
                let dist = frag_pos - light_trs.get_translation();
                let r2 = Vec3::from(dist).norm();
                // Square fallof
                1.0 * std::f32::consts::PI * r2
            }
        }
    }

    pub fn get_direction(&self, light_trs: &Trs, frag_pos: &Point3) -> Vec3 {
        match self._type {
            LightType::Distant => {
                let mut light_dir = Vec3::new(1.0, 0.0, 0.0);
                light_dir.rotate(&light_trs.rotation);
                -light_dir
            }
            LightType::Point => {
                let dist = frag_pos - light_trs.get_translation();
                -Vec3::from(dist).get_normalized()
            }
        }
    }
}

impl Default for Light {
    fn default() -> Self {
        Self::distant()
    }
}

/// Helper structure which should simplify drawing function interfaces
pub struct LightIntersection<'m> {
    pub light: &'m BvhLight<'m>,

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

impl<'m> LightIntersection<'m> {
    /// - l: light direction
    /// - n: normal to the surface
    /// - v: view direction
    pub fn new(
        light: &'m BvhLight,
        hit: &'m Hit,
        l: Vec3,
        n: Vec3,
        v: Vec3,
        albedo: Color,
        uv: Vec2,
    ) -> Self {
        let n_dot_v = n.dot(&v).clamp(0.0, 1.0);
        let n_dot_l = n.dot(&l).clamp(0.0, 1.0);
        let h = (v + l).get_normalized();
        let n_dot_h = n.dot(&h).clamp(0.0, 1.0);
        let l_dot_h = l.dot(&h).clamp(0.0, 1.0);

        Self {
            light,
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
