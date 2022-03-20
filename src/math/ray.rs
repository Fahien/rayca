// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use super::*;

#[derive(Debug)]
pub struct Ray {
    pub origin: Point3,
    pub dir: Vec3,
}

impl Ray {
    pub fn new(origin: Point3, dir: Vec3) -> Self {
        Self { origin, dir }
    }
}

pub struct Hit {
    pub depth: f32,
    pub point: Vec3,

    /// Barycentric coordinates expressing the hit point in terms of the primitive.
    /// Useful to interpolate vertex data of such a primitive
    pub uv: Vec2,
}

impl Hit {
    pub fn new(depth: f32, point: Vec3, uv: Vec2) -> Self {
        Self { depth, point, uv }
    }
}

pub trait Intersect {
    fn intersects(&self, ray: &Ray) -> Option<Hit>;
}

pub trait IntersectEx {}
