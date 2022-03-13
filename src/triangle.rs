// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use super::*;

pub struct Triangle {
    vertices: [Vec3; 3],
}

impl Triangle {
    pub fn new(a: Vec3, b: Vec3, c: Vec3) -> Self {
        Self {
            vertices: [a, b, c],
        }
    }
}

impl Default for Triangle {
    fn default() -> Self {
        Self::new(
            Vec3::new(-1.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        )
    }
}

impl Intersect for Triangle {
    /// [Ray-triangle intersection](https://www.scratchapixel.com/lessons/3d-basic-rendering/ray-tracing-rendering-a-triangle/ray-triangle-intersection-geometric-solution)
    fn intersects(&self, ray: &Ray) -> bool {
        // Plane's normal
        let v0v1 = self.vertices[1] - self.vertices[0];
        let v0v2 = self.vertices[2] - self.vertices[0];
        // No need to normalize
        let n = v0v1.cross(&v0v2);

        // Step 1: finding P

        // Check if ray and plane are parallel
        let n_dot_ray_dir = n.dot(&ray.dir);
        if n_dot_ray_dir.abs() < f32::EPSILON {
            // Parallel do not intersect
            return false;
        }
        // Compute d parameter using equation 2
        let d = -n.dot(&self.vertices[0]);

        // Compute t (equation 3)
        let t = -(n.dot(&ray.origin) + d) / n_dot_ray_dir;

        // Check if the triangle is behind the ray
        if t < 0.0 {
            return false;
        }

        // Compute the intersection point using equation 1
        let p = ray.origin + ray.dir * t;

        // Step 2: inside-outside test

        // Edge 0
        let edge0 = self.vertices[1] - self.vertices[0];
        let vp0 = p - self.vertices[0];
        // Vector perpendicular to triangle's plane
        let c = edge0.cross(&vp0);
        if n.dot(&c) < 0.0 {
            return false; // P is on the right side
        }

        // Edge 1
        let edge1 = self.vertices[2] - self.vertices[1];
        let vp1 = p - self.vertices[1];
        let c = edge1.cross(&vp1);
        if n.dot(&c) < 0.0 {
            return false; // P is on the right side
        }

        // Edge 2
        let edge2 = self.vertices[0] - self.vertices[2];
        let vp2 = p - self.vertices[2];
        let c = edge2.cross(&vp2);
        if n.dot(&c) < 0.0 {
            return false; // P is on the right side;
        }

        true // This ray hits the triangle
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn intersect() {
        let triangle = Triangle::default();
        let ray = Ray::new(Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, -1.0));
        assert!(triangle.intersects(&ray));
        let ray = Ray::new(Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, 1.0));
        assert!(!triangle.intersects(&ray));
    }
}
