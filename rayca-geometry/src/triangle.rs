// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

pub struct TriangleBuilder {
    vertices: [Point3; 3],
}

impl Default for TriangleBuilder {
    fn default() -> Self {
        Self {
            vertices: [
                Point3::new(-1.0, 0.0, 0.0),
                Point3::new(1.0, 0.0, 0.0),
                Point3::new(0.0, 1.0, 0.0),
            ],
        }
    }
}

impl TriangleBuilder {
    pub fn a(mut self, a: Point3) -> Self {
        self.vertices[0] = a;
        self
    }

    pub fn b(mut self, b: Point3) -> Self {
        self.vertices[1] = b;
        self
    }

    pub fn c(mut self, c: Point3) -> Self {
        self.vertices[2] = c;
        self
    }

    pub fn build(self) -> Triangle {
        Triangle::new(self.vertices)
    }
}

#[repr(C, align(16))]
pub struct Triangle {
    vertices: [Point3; 3],
    centroid: Vec3,
}

impl Triangle {
    pub fn builder() -> TriangleBuilder {
        TriangleBuilder::default()
    }

    pub fn new(vertices: [Point3; 3]) -> Self {
        let centroid =
            (Vec3::from(vertices[0]) + Vec3::from(vertices[1]) + Vec3::from(vertices[2])) * 0.3333;
        Self { vertices, centroid }
    }

    /// Returns the i-th vertex in transformed from model space to world space
    /// applying the model transform `trs`.
    pub fn get_vertex(&self, i: usize, trs: &Trs) -> Point3 {
        trs * self.vertices[i]
    }
}

impl Default for Triangle {
    fn default() -> Self {
        Self::new([
            Point3::new(-1.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        ])
    }
}

impl Triangle {
    /// [Ray-triangle intersection](https://www.scratchapixel.com/lessons/3d-basic-rendering/ray-tracing-rendering-a-triangle/ray-triangle-intersection-geometric-solution)
    pub fn intersects(&self, trs: &Trs, ray: &Ray) -> Option<Hit> {
        let v0 = Vec3::from(self.get_vertex(0, trs));
        let v1 = Vec3::from(self.get_vertex(1, trs));
        let v2 = Vec3::from(self.get_vertex(2, trs));

        // Plane's normal
        let v0v1 = v1 - v0;
        let v0v2 = v2 - v0;
        // No need to normalize
        let n = v0v1.cross(&v0v2);

        // Back-face test
        if ray.dir.dot(n) > 0.0 {
            return None;
        }

        let denom = n.dot(n);

        // Step 1: finding P

        // Check if ray and plane are parallel
        let n_dot_ray_dir = n.dot(ray.dir);
        if n_dot_ray_dir.abs() < f32::EPSILON {
            // Parallel do not intersect
            return None;
        }
        // Compute d parameter using equation 2
        let d = -n.dot(&v0);

        // Compute t (equation 3)
        let t = -(n.dot(Vec3::from(ray.origin)) + d) / n_dot_ray_dir;

        // Check if the triangle is behind the ray
        if t < 0.0 {
            return None;
        }

        // Compute the intersection point using equation 1
        let p = ray.origin + ray.dir * t;

        // Step 2: inside-outside test

        // Edge 0
        let edge0 = v1 - v0;
        let vp0 = Vec3::from(p - v0);
        // Vector perpendicular to triangle's plane
        let c = edge0.cross(&vp0);
        if n.dot(c) < 0.0 {
            return None; // P is on the right side
        }

        // Edge 1
        let edge1 = v2 - v1;
        let vp1 = Vec3::from(p - v1);
        let c = edge1.cross(&vp1);
        let u = n.dot(c);
        if u < 0.0 {
            return None; // P is on the right side
        }

        // Edge 2
        let edge2 = v0 - v2;
        let vp2 = Vec3::from(p - v2);
        let c = edge2.cross(&vp2);
        let v = n.dot(c);
        if v < 0.0 {
            return None; // P is on the right side;
        }

        let uv = Vec2::new(u / denom, v / denom);

        // A triangle does not know its primitive index
        // Setting the primitive index is responsiblity of the caller
        let hit = Hit::new(u32::MAX, u32::MAX, t, p, uv);
        Some(hit) // This ray hits the triangle
    }

    pub fn get_centroid(&self, trs: &Trs) -> Vec3 {
        trs * self.centroid
    }

    pub fn min(&self, trs: &Trs) -> Point3 {
        Point3::new(f32::MAX, f32::MAX, f32::MAX)
            .min(self.get_vertex(0, trs))
            .min(self.get_vertex(1, trs))
            .min(self.get_vertex(2, trs))
    }

    pub fn max(&self, trs: &Trs) -> Point3 {
        Point3::new(f32::MIN, f32::MIN, f32::MIN)
            .max(self.get_vertex(0, trs))
            .max(self.get_vertex(1, trs))
            .max(self.get_vertex(2, trs))
    }
}

/// Collection of triangles defined by vertices and indices.
#[derive(Debug, Clone)]
pub struct TriangleMesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u8>,

    /// Index size in bytes. This is not index count
    pub index_size_in_bytes: usize,
}

impl Default for TriangleMesh {
    fn default() -> Self {
        Self::new(vec![], vec![])
    }
}

impl TriangleMesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u8>) -> Self {
        Self {
            vertices,
            indices,
            index_size_in_bytes: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intersect() {
        let triangle = Triangle::default();
        let ray = Ray::new(Point3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, -1.0));
        assert!(triangle.intersects(&Trs::IDENTITY, &ray).is_some());
        let ray = Ray::new(Point3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, 1.0));
        assert!(triangle.intersects(&Trs::IDENTITY, &ray).is_none());
    }

    #[test]
    fn centroid_and_min_max() {
        let triangle = Triangle::default();
        let trs = Trs::IDENTITY;
        let centroid = triangle.get_centroid(&trs);
        let expected_centroid = (Vec3::from(Point3::new(-1.0, 0.0, 0.0))
            + Vec3::from(Point3::new(1.0, 0.0, 0.0))
            + Vec3::from(Point3::new(0.0, 1.0, 0.0)))
            * 0.3333;
        assert!((centroid - expected_centroid).len() < 1e-3);

        let min = triangle.min(&trs);
        let max = triangle.max(&trs);
        assert_eq!(min, Point3::new(-1.0, 0.0, 0.0));
        assert_eq!(max, Point3::new(1.0, 1.0, 0.0));
    }

    #[test]
    #[should_panic]
    fn get_vertex_out_of_bounds() {
        let triangle = Triangle::default();
        let _ = triangle.get_vertex(3, &Trs::IDENTITY);
    }
}
