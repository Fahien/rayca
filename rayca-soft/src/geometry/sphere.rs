// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

#[derive(Debug, Clone)]
pub struct Sphere {
    pub center: Point3,
    radius: f32,
    radius2: f32,
}

impl Sphere {
    pub fn new(center: Point3, radius: f32) -> Self {
        let radius2 = radius * radius;
        Self {
            center,
            radius,
            radius2,
        }
    }

    pub fn set_radius(&mut self, radius: f32) {
        self.radius = radius;
        self.radius2 = radius * radius;
    }

    pub fn get_radius(&self) -> f32 {
        self.radius
    }

    pub fn primitives(&self, node: Handle<Node>, material: Handle<Material>) -> Vec<BvhPrimitive> {
        // Transforming a sphere is complicated. The trick is to store transform with sphere,
        // then pre-transform the ray, and post-transform the intersection point.
        let sphere = BvhSphere::new(self.center, self.radius);
        let geometry = BvhGeometry::Sphere(sphere);
        let primitive = BvhPrimitive::new(geometry, node, material);

        vec![primitive]
    }
}

impl Default for Sphere {
    fn default() -> Self {
        Self {
            center: Default::default(),
            radius: 1.0,
            radius2: 1.0,
        }
    }
}
