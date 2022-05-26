// Copyright © 2022-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::{Color, Node, Point3, Trs, Vec3};

#[repr(C, align(16))]
#[derive(Clone, Debug)]
pub struct Light {
    pub color: Color,
    pub attenuation: Vec3,
}

impl Light {
    pub fn new(color: Color, attenuation: Vec3) -> Self {
        Self { color, attenuation }
    }

    pub fn point() -> Self {
        Self::default()
    }

    pub fn scale_intensity(&mut self, intensity: f32) {
        self.color.r *= intensity;
        self.color.g *= intensity;
        self.color.b *= intensity;
    }

    pub fn get_distance(&self, light_node: &Node, frag_pos: &Point3) -> f32 {
        let dist = Vec3::from(frag_pos) - light_node.trs.translation;
        dist.len()
    }

    pub fn get_fallof(&self, light_trs: &Trs, frag_pos: &Point3) -> f32 {
        let dist = Vec3::from(frag_pos) - light_trs.translation;
        let r2 = dist.norm();
        // Square fallof
        1.0 * std::f32::consts::PI * r2
    }

    pub fn get_direction(&self, light_node: &Node, frag_pos: &Point3) -> Vec3 {
        let mut dist = Vec3::from(frag_pos) - light_node.trs.translation;
        dist.normalize();
        -dist
    }
}

impl Default for Light {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            attenuation: Vec3::new(0.0, 0.0, 1.0),
        }
    }
}
