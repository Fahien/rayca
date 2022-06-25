// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

pub struct BvhSphere {
    pub center: Point3,
    radius: f32,
    radius2: f32,
}

impl BvhSphere {
    pub fn new(mut center: Point3, radius: f32) -> Self {
        center.simd[3] = 1.0;
        Self {
            center,
            radius,
            radius2: radius * radius,
        }
    }

    pub fn get_radius(&self) -> f32 {
        self.radius
    }

    /// This will return a point outside of the sphere, useful for the AABB
    pub fn min(&self) -> Point3 {
        let rad3 = Vec3::splat(self.radius);
        self.center - rad3
    }

    pub fn max(&self) -> Point3 {
        let rad3 = Vec3::splat(self.radius);
        self.center + rad3
    }

    /// Point should be in geometry space
    pub fn get_normal(&self, point: &Point3) -> Vec3 {
        (point - self.center).get_normalized()
    }

    /// Geometric formula.
    /// Ray should be in model space
    pub fn intersects(&self, ray: &Ray) -> Option<Hit> {
        // a = p1 * p1
        let a = ray.dir.dot(&ray.dir);

        // b = 2(p1 * (p0 - c))
        // sphere center to ray origin vector
        let c_to_r = ray.origin - self.center;
        let b = 2.0 * c_to_r.dot(&ray.dir);

        // c = (p0 - c) * (p0 - c) - r^2
        let c = c_to_r.dot(&c_to_r) - self.radius2;

        // (-b +- sqrt(b^2 - 4ac) ) / 2a;
        let det = b * b - 4.0 * a * c;
        if det < 0.0 {
            return None;
        }

        let t0 = (-b + det.sqrt()) / (2.0 * a);
        let t1 = (-b - det.sqrt()) / (2.0 * a);

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
        let hit = Hit::new(t, point, Vec2::default());

        Some(hit)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn intersect() {
        let orig = Point3::new(0.0, 0.0, 0.0);
        let sphere = BvhSphere::new(orig, 1.0);

        let right = Vec3::new(1.0, 0.0, 0.0);
        let ray = Ray::new(orig, right);
        assert!(sphere.intersects(&ray).is_some());

        let ray = Ray::new(Point3::new(2.0, 0.0, 0.0), right);
        assert!(sphere.intersects(&ray).is_none());

        let sphere = BvhSphere::new(Point3::new(4.0, 0.0, 0.0), 1.0);
        let forward = Vec3::new(0.0, 0.0, -1.0);
        let ray = Ray::new(orig, forward);
        assert!(sphere.intersects(&ray).is_none());
    }
}
