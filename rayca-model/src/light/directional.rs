// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

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
