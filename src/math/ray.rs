// Copyright © 2022-2024
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use super::*;

#[derive(Debug, Clone)]
pub struct Ray {
    pub origin: Point3,
    pub dir: Vec3,

    // Reciprocal of direction
    pub rdir: Vec3,
}

impl Ray {
    pub fn new(mut origin: Point3, dir: Vec3) -> Self {
        let rdir = dir.get_reciprocal();
        origin.simd[3] = 1.0;
        Self { origin, dir, rdir }
    }

    pub fn scale(&mut self, scale: &Vec3) {
        self.dir.scale(scale);
        self.rdir = self.dir.get_reciprocal();
        self.origin.scale(scale);

        assert!(self.origin.simd[3] == 1.0);
    }

    pub fn translate(&mut self, translation: &Vec3) {
        self.origin += translation;

        assert!(self.origin.simd[3] == 1.0);
    }

    pub fn rotate(&mut self, rotation: &Quat) {
        self.origin.rotate(rotation);
        self.dir.rotate(rotation);
        self.rdir = self.dir.get_reciprocal();
        self.origin.rotate(rotation);
        self.origin.simd[3] = 1.0;
    }
}

impl Default for Ray {
    fn default() -> Self {
        Self::new(Default::default(), Vec3::new(0.0, 0.0, -1.0))
    }
}

pub struct Hit {
    pub depth: f32,
    pub point: Point3,

    /// Barycentric coordinates expressing the hit point in terms of the primitive.
    /// Useful to interpolate vertex data of such a primitive
    pub uv: Vec2,
}

impl Hit {
    pub fn new(depth: f32, point: Point3, uv: Vec2) -> Self {
        Self { depth, point, uv }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn rotate() {
        let mut ray = Ray::default();
        let rot = Quat::new(-0.383, 0.0, 0.0, 0.924);
        ray.rotate(&rot);
        println!("{:?}", ray.dir);
        assert!(ray.dir.close(&Vec3::new(0.0, -0.707, -0.707)));
    }
}
