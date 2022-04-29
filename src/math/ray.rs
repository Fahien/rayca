// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use super::*;

#[derive(Debug, Clone)]
pub struct Ray {
    pub origin: Point3,
    pub dir: Vec3,
}

impl Ray {
    pub fn new(origin: Point3, dir: Vec3) -> Self {
        Self { origin, dir }
    }

    pub fn translate(&mut self, translation: &Vec3) {
        self.origin += translation;
    }

    pub fn rotate(&mut self, rotation: &Quat) {
        self.dir.rotate(rotation);
    }
}

impl Default for Ray {
    fn default() -> Self {
        Self::new(Default::default(), Vec3::new(0.0, 0.0, -1.0))
    }
}

pub struct Hit {
    /// Primitive index hit in this intersection
    /// If this index goes beyond the final triangle, try with spheres
    pub primitive: u32,

    pub depth: f32,
    pub point: Vec3,

    /// Barycentric coordinates expressing the hit point in terms of the primitive.
    /// Useful to interpolate vertex data of such a primitive
    pub uv: Vec2,
}

impl Hit {
    pub fn new(primitive: u32, depth: f32, point: Vec3, uv: Vec2) -> Self {
        Self {
            primitive,
            depth,
            point,
            uv,
        }
    }
}

pub trait Intersect {
    fn intersects(&self, ray: &Ray) -> Option<Hit>;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn rotate() {
        let mut ray = Ray::default();
        let rot = Quat::new(-0.383, 0.0, 0.0, 0.924);
        ray.rotate(&rot);
        println!("{} {} {}", ray.dir.x, ray.dir.y, ray.dir.z);
        assert!(ray.dir.close(&Vec3::new(0.0, -0.707, -0.707)));
    }
}
