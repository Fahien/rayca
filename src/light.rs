// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use super::*;

pub enum LightType {
    Distant,
    Point,
}

pub struct Light {
    color: Vec3,
    intensity: f32,
    pub _type: LightType,
}

impl Light {
    pub fn distant() -> Self {
        Self {
            color: Vec3::new(1.0, 1.0, 1.0),
            intensity: 1.0,
            _type: LightType::Distant,
        }
    }

    pub fn point() -> Self {
        Self {
            color: Vec3::new(1.0, 1.0, 1.0),
            intensity: 1.0,
            _type: LightType::Point,
        }
    }

    pub fn set_intensity(&mut self, intensity: f32) {
        self.intensity = intensity;
    }

    pub fn get_distance(&self, light_node: &Node, frag_pos: &Vec3) -> f32 {
        let dist = frag_pos - light_node.trs.translation;
        dist.len()
    }

    pub fn get_intensity(&self, light_node: &Node, frag_pos: &Vec3) -> Vec3 {
        let colored_intensity = self.intensity * self.color;

        match self._type {
            LightType::Distant => colored_intensity,
            LightType::Point => {
                let dist = frag_pos - light_node.trs.translation;
                let r2 = dist.norm();
                let square_falloff = 1.0 * std::f32::consts::PI * r2;
                colored_intensity / square_falloff
            }
        }
    }

    pub fn get_direction(&self, light_node: &Node, frag_pos: &Vec3) -> Vec3 {
        match self._type {
            LightType::Distant => {
                let mut light_dir = Vec3::new(1.0, 0.0, 0.0);
                light_dir.rotate(&light_node.trs.rotation);
                -light_dir
            }
            LightType::Point => {
                let mut dist = frag_pos - light_node.trs.translation;
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
