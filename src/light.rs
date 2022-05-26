// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use super::*;

pub enum Light {
    Directional(DirectionalLight),
    Point(PointLight),
}

impl Light {
    pub fn directional() -> Self {
        Self::Directional(DirectionalLight::new())
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

    pub fn get_distance(&self, light_node: &Node, frag_pos: &Point3) -> f32 {
        match self {
            Light::Directional(light) => light.get_distance(light_node, frag_pos),
            Light::Point(light) => light.get_distance(light_node, frag_pos),
        }
    }

    pub fn get_intensity(&self) -> Color {
        match self {
            Light::Directional(light) => light.get_intensity(),
            Light::Point(light) => light.get_intensity(),
        }
    }

    pub fn get_fallof(&self, light_trs: &Trs, frag_pos: &Point3) -> f32 {
        match self {
            Light::Directional(light) => light.get_fallof(),
            Light::Point(light) => light.get_fallof(light_trs, frag_pos),
        }
    }

    pub fn get_direction(&self, light_node: &Node, frag_pos: &Point3) -> Vec3 {
        match self {
            Light::Directional(light) => light.get_direction(light_node),
            Light::Point(light) => light.get_direction(light_node, frag_pos),
        }
    }
}

pub struct DirectionalLight {
    color: Color,
    intensity: f32,
}

impl DirectionalLight {
    pub fn new() -> Self {
        Self {
            color: Color::new(1.0, 1.0, 1.0, 1.0),
            intensity: 1.0,
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

    pub fn get_fallof(&self) -> f32 {
        1.0
    }

    pub fn get_direction(&self, light_node: &Node) -> Vec3 {
        let mut light_dir = Vec3::new(1.0, 0.0, 0.0);
        light_dir.rotate(&light_node.trs.rotation);
        -light_dir
    }
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self::new()
    }
}

pub struct PointLight {
    color: Color,
    intensity: f32,
}

impl PointLight {
    pub fn new() -> Self {
        Self {
            color: Color::new(1.0, 1.0, 1.0, 1.0),
            intensity: 1.0,
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

impl Default for PointLight {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for Light {
    fn default() -> Self {
        Self::Directional(DirectionalLight::new())
    }
}
