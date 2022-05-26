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

    pub fn get_distance(&self, light_node: &Node, frag_pos: &Point3) -> f32 {
        let dist = Vec3::from(frag_pos) - light_node.trs.translation;
        dist.len()
    }

    pub fn get_intensity(&self) -> Color {
        self.intensity * self.color
    }

    pub fn get_fallof(&self, light_trs: &Trs, frag_pos: &Point3) -> f32 {
        match self._type {
            LightType::Distant => 1.0,
            LightType::Point => {
                let dist = Vec3::from(frag_pos) - light_trs.translation;
                let r2 = dist.norm();
                // Square fallof
                1.0 * std::f32::consts::PI * r2
            }
        }
    }

    pub fn get_direction(&self, light_node: &Node, frag_pos: &Point3) -> Vec3 {
        match self._type {
            LightType::Distant => {
                let mut light_dir = Vec3::new(1.0, 0.0, 0.0);
                light_dir.rotate(&light_node.trs.rotation);
                -light_dir
            }
            LightType::Point => {
                let mut dist = Vec3::from(frag_pos) - light_node.trs.translation;
                dist.normalize();
                -dist
            }
        }
    }
}

impl Default for Light {
    fn default() -> Self {
        Self::distant()
    }
}
