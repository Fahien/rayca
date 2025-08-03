// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use rayca_math::*;

pub struct SphereBuilder {
    center: Point3,
    radius: f32,
}

impl SphereBuilder {
    pub fn new() -> Self {
        Self {
            center: Point3::default(),
            radius: 1.0,
        }
    }

    pub fn center(mut self, center: Point3) -> Self {
        self.center = center;
        self
    }

    pub fn radius(mut self, radius: f32) -> Self {
        if radius < 0.0 {
            panic!("Sphere radius cannot be negative");
        }
        self.radius = radius;
        self
    }

    pub fn build(self) -> Sphere {
        Sphere::new(self.center, self.radius)
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Sphere {
    center: Point3,
    radius: f32,
    radius2: f32,
}

impl Sphere {
    pub fn builder() -> SphereBuilder {
        SphereBuilder::new()
    }

    pub fn new(center: Point3, radius: f32) -> Self {
        if radius < 0.0 {
            panic!("Sphere radius cannot be negative");
        }
        // Precompute radius squared for performance
        let radius2 = radius * radius;
        Self {
            center,
            radius,
            radius2,
        }
    }

    pub fn unit() -> Self {
        Self::new(Point3::default(), 1.0)
    }

    /// Returns the center of the sphere in model space.
    pub fn get_model_center(&self) -> Point3 {
        self.center
    }

    pub fn get_center(&self, trs: &Trs) -> Point3 {
        trs * self.center
    }

    pub fn set_center(&mut self, center: Point3) {
        self.center = center;
    }

    pub fn set_radius(&mut self, radius: f32) {
        self.radius = radius;
        self.radius2 = radius * radius;
    }

    pub fn get_model_radius(&self) -> f32 {
        self.radius
    }

    pub fn get_radius(&self, trs: &Trs) -> f32 {
        self.radius * trs.scale.reduce_max()
    }

    /// - `point`: should be in model space
    pub fn get_model_normal(&self, point: &Point3) -> Vec3 {
        (point - self.center).get_normalized()
    }

    /// Geometric formula.
    /// - `ray`: Should be in model space
    pub fn intersects_impl(&self, ray: &Ray) -> Option<Hit> {
        // a = p1 * p1
        let a = ray.dir.dot(&ray.dir);

        // b = 2(p1 * (p0 - c))
        // sphere center to ray origin vector
        let c_to_r = ray.origin - self.center;
        let b = c_to_r.dot(&ray.dir);

        // c = (p0 - c) * (p0 - c) - r^2
        let c = c_to_r.dot(&c_to_r) - self.radius2;

        // (-b +- sqrt(b^2 - 4ac) ) / 2a;
        let det = b * b - a * c;
        if det < 0.0 {
            return None;
        }

        let det_sqrt = det.sqrt();
        let t0 = (-b + det_sqrt) / a;
        let t1 = (-b - det_sqrt) / a;

        if t0 < 0.0 && t1 < 0.0 {
            return None; // Sphere behind ray origin
        }

        let t = if t0 >= 0.0 && t1 >= 0.0 {
            // Two positive roots, pick smaller
            t0.min(t1)
        } else if t0 >= 0.0 {
            // Ray origin inside sphere, pick positive
            t0
        } else {
            t1
        };

        let point = ray.origin + ray.dir * t;
        let hit = Hit::new(u32::MAX, u32::MAX, t, point, Vec2::default());
        Some(hit)
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

impl Sphere {
    /// [Ray-sphere intersection](https://www.scratchapixel.com/lessons/3d-basic-rendering/minimal-ray-tracer-rendering-simple-shapes/ray-sphere-intersection)
    pub fn intersects(&self, trs: &Trs, ray: &Ray) -> Option<Hit> {
        let ray = ray.clone();
        let inverse = Inversed::from(trs);
        let inverse_ray = &inverse * ray;
        let mut hit = self.intersects_impl(&inverse_ray)?;
        let transformed_point = hit.point;
        hit.point = trs * transformed_point;
        Some(hit)
    }

    pub fn get_centroid(&self, trs: &Trs) -> Vec3 {
        self.get_center(trs).into()
    }

    /// This will return a point outside of the sphere, useful for the AABB
    pub fn min(&self, trs: &Trs) -> Point3 {
        let rad3 = Vec3::splat(self.get_radius(trs));
        self.get_center(trs) - rad3
    }

    pub fn max(&self, trs: &Trs) -> Point3 {
        let rad3 = Vec3::splat(self.get_radius(trs));
        self.get_center(trs) + rad3
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intersect() {
        let orig = Point3::new(0.0, 0.0, 0.0);
        let sphere = Sphere::new(orig, 1.0);

        let right = Vec3::new(1.0, 0.0, 0.0);
        let ray = Ray::new(orig, right);
        assert!(sphere.intersects(&Trs::IDENTITY, &ray).is_some());

        let ray = Ray::new(Point3::new(2.0, 0.0, 0.0), right);
        assert!(sphere.intersects(&Trs::IDENTITY, &ray).is_none());

        let sphere = Sphere::new(Point3::new(4.0, 0.0, 0.0), 1.0);
        let forward = Vec3::new(0.0, 0.0, -1.0);
        let ray = Ray::new(orig, forward);
        assert!(sphere.intersects(&Trs::IDENTITY, &ray).is_none());
    }

    #[test]
    fn min_max_and_centroid() {
        let sphere = Sphere::new(Point3::new(1.0, 2.0, 3.0), 2.0);
        let trs = Trs::IDENTITY;
        let min = sphere.min(&trs);
        let max = sphere.max(&trs);
        assert_eq!(min, Point3::new(-1.0, 0.0, 1.0));
        assert_eq!(max, Point3::new(3.0, 4.0, 5.0));
        let centroid = sphere.get_centroid(&trs);
        assert_eq!(centroid, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn get_normal() {
        let sphere = Sphere::new(Point3::new(0.0, 0.0, 0.0), 1.0);
        let point = Point3::new(0.0, 1.0, 0.0);
        let normal = sphere.get_model_normal(&point);
        assert!((normal - Vec3::new(0.0, 1.0, 0.0)).norm() < 1e-6);
    }

    #[test]
    #[should_panic]
    fn negative_radius() {
        let _ = Sphere::new(Point3::new(0.0, 0.0, 0.0), -1.0);
    }
}
