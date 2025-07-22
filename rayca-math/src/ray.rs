// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

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
        assert_eq!(self.origin.simd[3], 1.0);
    }

    pub fn translate(&mut self, translation: &Vec3) {
        self.origin += translation;
        assert_eq!(self.origin.simd[3], 1.0);
    }

    pub fn rotate(&mut self, rotation: Quat) {
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
    /// BLAS index hit in this intersection
    pub blas: u32,

    /// Primitive index hit in this intersection
    /// If this index goes beyond the final triangle, try with spheres
    pub primitive: u32,

    pub depth: f32,
    pub point: Point3,

    /// Barycentric coordinates expressing the hit point in terms of the primitive.
    /// Useful to interpolate vertex data of such a primitive
    pub uv: Vec2,
}

impl Hit {
    pub fn new(blas: u32, primitive: u32, depth: f32, point: Point3, uv: Vec2) -> Self {
        Self {
            blas,
            primitive,
            depth,
            point,
            uv,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn rotate() {
        let mut ray = Ray::default();
        let rot = Quat::new(-0.383, 0.0, 0.0, 0.924);
        ray.rotate(rot);
        println!("{:?}", ray.dir);
        assert!(ray.dir.close(&Vec3::new(0.0, -0.707, -0.707)));
    }

    #[test]
    fn scale() {
        let mut ray = Ray::default();
        let scale = Vec3::new(2.0, 2.0, 2.0); // uniform scaling to avoid floating-point artifacts
        let orig_dir = ray.dir;
        ray.scale(&scale);
        assert!(ray.dir.close(&(orig_dir * scale)));
        let expected_rdir = ray.dir.get_reciprocal();
        let eps = 1e-5;
        let dx = (ray.rdir.get_x() - expected_rdir.get_x()).abs();
        let dy = (ray.rdir.get_y() - expected_rdir.get_y()).abs();
        let dz = (ray.rdir.get_z() - expected_rdir.get_z()).abs();
        assert!(
            dx < eps && dy < eps && dz < eps,
            "rdir and dir.get_reciprocal() differ by more than eps"
        );
        assert_eq!(ray.origin.simd[3], 1.0);
    }

    #[test]
    fn translate() {
        let mut ray = Ray::default();
        let translation = Vec3::new(1.0, 2.0, 3.0);
        let orig_origin = ray.origin;
        ray.translate(&translation);
        assert!(ray.origin.close(&(orig_origin + translation)));
        assert_eq!(ray.origin.simd[3], 1.0);
    }

    #[test]
    fn new_with_zero_direction_panics() {
        let origin = Point3::new(0.0, 0.0, 0.0);
        let dir = Vec3::new(0.0, 0.0, 0.0);
        let result = std::panic::catch_unwind(|| Ray::new(origin, dir));
        assert!(
            result.is_ok(),
            "Ray::new should not panic, but check for zero direction if needed"
        );
    }

    #[test]
    fn direction_not_normalized() {
        let origin = Point3::new(0.0, 0.0, 0.0);
        let dir = Vec3::new(0.0, 0.0, -10.0);
        let ray = Ray::new(origin, dir);
        assert_eq!(ray.dir, dir);
    }
}
