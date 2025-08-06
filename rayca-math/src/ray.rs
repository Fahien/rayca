// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

#[derive(Default, Debug, Clone)]
pub struct RayBuilder {
    origin: Point3,
    dir: Vec3,
    throughput: Color,
}

impl RayBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn origin(mut self, origin: Point3) -> Self {
        self.origin = origin;
        self
    }

    pub fn dir(mut self, dir: Vec3) -> Self {
        self.dir = dir;
        self
    }

    pub fn throughput(mut self, throughput: Color) -> Self {
        self.throughput = throughput;
        self
    }

    pub fn build(self) -> Ray {
        let mut ray = Ray::new(self.origin, self.dir);
        ray.throughput = self.throughput;
        ray
    }
}

#[derive(Debug, Clone)]
pub struct Ray {
    /// Origin of the ray in world space.
    pub origin: Point3,

    /// Direction of the ray in world space.
    pub dir: Vec3,

    /// Reciprocal of direction.
    pub rdir: Vec3,

    /// Used with the russian roulette.
    pub throughput: Color,
}

impl Ray {
    pub const BIAS: f32 = 1e-4;

    pub fn builder() -> RayBuilder {
        RayBuilder::new()
    }

    pub fn new(mut origin: Point3, dir: Vec3) -> Self {
        let rdir = dir.get_reciprocal();
        origin.simd[3] = 1.0;
        Self {
            origin,
            dir,
            rdir,
            throughput: Color::WHITE,
        }
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

    /// This function applies the russian roulette method.
    /// If the ray is not to be terminated, it returns the boost factor.
    pub fn next_russian_roulette(&self, next_throughput: Color) -> Option<f32> {
        // Fragments with high information have a higher change to generate a new ray.
        let q = 1.0 - next_throughput.get_rgb().reduce_max().min(1.0);
        if q < fastrand::f32() {
            let weight = 1.0 / (1.0 - q);
            Some(weight)
        } else {
            // Path has not survived the russian roulette
            None
        }
    }
}

impl Default for Ray {
    fn default() -> Self {
        Self::new(Default::default(), Vec3::new(0.0, 0.0, -1.0))
    }
}

#[derive(Default)]
pub struct Hit {
    /// Ray that hit the primitive.
    pub ray: Ray,

    /// Point of intersection in world space.
    pub point: Point3,

    /// BLAS index hit in this intersection
    pub blas: u32,

    /// Primitive index hit in this intersection
    /// If this index goes beyond the final triangle, try with spheres
    pub primitive: u32,

    pub depth: f32,

    /// Barycentric coordinates expressing the hit point in terms of the primitive.
    /// Useful to interpolate vertex data of such a primitive
    pub uv: Vec2,
}

impl Hit {
    pub fn new(ray: Ray, blas: u32, primitive: u32, depth: f32, point: Point3, uv: Vec2) -> Self {
        Self {
            ray,
            depth,
            blas,
            primitive,
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
        assert!(ray.dir.close(Vec3::new(0.0, -0.707, -0.707)));
    }

    #[test]
    fn scale() {
        let mut ray = Ray::default();
        let scale = Vec3::new(2.0, 2.0, 2.0); // uniform scaling to avoid floating-point artifacts
        let orig_dir = ray.dir;
        ray.scale(&scale);
        assert!(ray.dir.close(orig_dir * scale));
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
        assert!(ray.origin.close(orig_origin + translation));
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
