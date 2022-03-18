// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use super::*;

pub struct Triangle {
    pub vertices: [Vertex; 3],
}

impl Triangle {
    pub fn new(a: Vertex, b: Vertex, c: Vertex) -> Self {
        Self {
            vertices: [a, b, c],
        }
    }
}

impl Default for Triangle {
    fn default() -> Self {
        Self::new(
            Vertex::new(-1.0, 0.0, 0.0),
            Vertex::new(1.0, 0.0, 0.0),
            Vertex::new(0.0, 1.0, 0.0),
        )
    }
}

impl Intersect for Triangle {
    /// [Ray-triangle intersection](https://www.scratchapixel.com/lessons/3d-basic-rendering/ray-tracing-rendering-a-triangle/ray-triangle-intersection-geometric-solution)
    fn intersects(&self, ray: &Ray) -> Option<Hit> {
        let v0 = Vec3::from(self.vertices[0].point);
        let v1 = Vec3::from(self.vertices[1].point);
        let v2 = Vec3::from(self.vertices[2].point);

        // Plane's normal
        let v0v1 = v1 - v0;
        let v0v2 = v2 - v0;
        // No need to normalize
        let n = v0v1.cross(&v0v2);
        let denom = n.dot(n);

        // Step 1: finding P

        // Check if ray and plane are parallel
        let n_dot_ray_dir = n.dot(ray.dir);
        if n_dot_ray_dir.abs() < f32::EPSILON {
            // Parallel do not intersect
            return None;
        }
        // Compute d parameter using equation 2
        let d = -n.dot(v0);

        // Compute t (equation 3)
        let t = -(n.dot(ray.origin) + d) / n_dot_ray_dir;

        // Check if the triangle is behind the ray
        if t < 0.0 {
            return None;
        }

        // Compute the intersection point using equation 1
        let p = ray.origin + ray.dir * t;

        // Step 2: inside-outside test

        // Edge 0
        let edge0 = v1 - v0;
        let vp0 = p - v0;
        // Vector perpendicular to triangle's plane
        let c = edge0.cross(&vp0);
        if n.dot(c) < 0.0 {
            return None; // P is on the right side
        }

        // Edge 1
        let edge1 = v2 - v1;
        let vp1 = p - v1;
        let c = edge1.cross(&vp1);
        let u = n.dot(c);
        if u < 0.0 {
            return None; // P is on the right side
        }

        // Edge 2
        let edge2 = v0 - v2;
        let vp2 = p - v2;
        let c = edge2.cross(&vp2);
        let v = n.dot(c);
        if v < 0.0 {
            return None; // P is on the right side;
        }

        let uv = Vec2::new(u / denom, v / denom);
        let hit = Hit::new(p, uv);
        Some(hit) // This ray hits the triangle
    }

    fn get_color(&self, hit: &Hit) -> Color {
        // Interpolate vertex colors
        let c0 = &self.vertices[0].color;
        let c1 = &self.vertices[1].color;
        let c2 = &self.vertices[2].color;

        (1.0 - hit.uv.x - hit.uv.y) * c2 + hit.uv.x * c0 + hit.uv.y * c1
    }

    fn get_normal(&self, _hit: &Hit) -> Vec3 {
        Vec3::new(0.0, 0.0, 1.0)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn intersect() {
        let triangle = Triangle::default();
        let ray = Ray::new(Point3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, -1.0));
        assert!(triangle.intersects(&ray).is_some());
        let ray = Ray::new(Point3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, 1.0));
        assert!(triangle.intersects(&ray).is_none());
    }
}
