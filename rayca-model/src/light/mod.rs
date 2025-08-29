// Copyright © 2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

mod directional;
mod point;
mod quad;

pub use directional::*;
pub use point::*;
pub use quad::*;

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

    pub fn get_area(&self) -> f32 {
        match self {
            Light::Directional(_) => 0.0,
            Light::Point(_) => 0.0,
            Light::Quad(light) => light.get_area(),
        }
    }

    pub fn get_normal(&self, light_trs: &Trs) -> Vec3 {
        match self {
            Light::Directional(d) => d.get_direction(light_trs),
            Light::Point(_) => Vec3::default(), // all directions
            Light::Quad(q) => q.get_normal(),
        }
    }

    pub fn intersects(&self, light_trs: &Trs, ray: Ray) -> Option<Hit> {
        match self {
            Light::Directional(_) => None,
            Light::Point(_) => None,
            Light::Quad(light) => light.intersects(light_trs, ray),
        }
    }
}

impl Default for Light {
    fn default() -> Self {
        Self::Directional(DirectionalLight::default())
    }
}
