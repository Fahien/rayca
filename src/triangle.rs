// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use super::*;

pub struct Triangle<'m, V> {
    pub vertices: [V; 3],
    pub material: &'m Material,
}

impl<'m, V> Triangle<'m, V> {
    pub fn new(a: V, b: V, c: V, material: &'m Material) -> Self {
        Self {
            vertices: [a, b, c],
            material,
        }
    }
}

impl<'m> Triangle<'m, Vertex> {
    pub fn unit(material: &'m Material) -> Self {
        Self::new(
            Vertex::new(-1.0, 0.0, 0.0),
            Vertex::new(1.0, 0.0, 0.0),
            Vertex::new(0.0, 1.0, 0.0),
            material,
        )
    }
}

impl<'m> Intersect for Triangle<'m, Vertex> {
    /// [Ray-triangle intersection](https://www.scratchapixel.com/lessons/3d-basic-rendering/ray-tracing-rendering-a-triangle/ray-triangle-intersection-geometric-solution)
    fn intersects(&self, ray: &Ray) -> Option<Hit> {
        let v0 = &self.vertices[0].pos;
        let v1 = &self.vertices[1].pos;
        let v2 = &self.vertices[2].pos;

        // Plane's normal
        let v0v1 = v1 - v0;
        let v0v2 = v2 - v0;
        // No need to normalize
        let n = v0v1.cross(&v0v2);
        let denom = n.dot(&n);

        // Step 1: finding P

        // Check if ray and plane are parallel
        let n_dot_ray_dir = n.dot(&ray.dir);
        if n_dot_ray_dir.abs() < f32::EPSILON {
            // Parallel do not intersect
            return None;
        }
        // Compute d parameter using equation 2
        let d = -n.dot(&v0);

        // Compute t (equation 3)
        let t = -(n.dot(&ray.origin) + d) / n_dot_ray_dir;

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
        if n.dot(&c) < 0.0 {
            return None; // P is on the right side
        }

        // Edge 1
        let edge1 = v2 - v1;
        let vp1 = p - v1;
        let c = edge1.cross(&vp1);
        let u = n.dot(&c);
        if u < 0.0 {
            return None; // P is on the right side
        }

        // Edge 2
        let edge2 = v0 - v2;
        let vp2 = p - v2;
        let c = edge2.cross(&vp2);
        let v = n.dot(&c);
        if v < 0.0 {
            return None; // P is on the right side;
        }

        let uv = Vec2::new(u / denom, v / denom);
        let hit = Hit::new(t, p, uv);
        Some(hit) // This ray hits the triangle
    }

    fn get_color(&self, hit: &Hit) -> RGBA8 {
        // Interpolate vertex colors
        let c0 = &self.vertices[0].color;
        let c1 = &self.vertices[1].color;
        let c2 = &self.vertices[2].color;

        self.material.color * ((1.0 - hit.uv.x - hit.uv.y) * c2 + hit.uv.x * c0 + hit.uv.y * c1)
    }

    fn get_normal(&self, hit: &Hit) -> Vec3 {
        // Interpolate normals
        let n0 = &self.vertices[0].normal;
        let n1 = &self.vertices[1].normal;
        let n2 = &self.vertices[2].normal;

        n2 * (1.0 - hit.uv.x - hit.uv.y) + n0 * hit.uv.x + n1 * hit.uv.y
    }
}

impl<'m> Intersect for Triangle<'m, &Vertex> {
    /// [Ray-triangle intersection](https://www.scratchapixel.com/lessons/3d-basic-rendering/ray-tracing-rendering-a-triangle/ray-triangle-intersection-geometric-solution)
    fn intersects(&self, ray: &Ray) -> Option<Hit> {
        let v0 = &self.vertices[0].pos;
        let v1 = &self.vertices[1].pos;
        let v2 = &self.vertices[2].pos;

        // Plane's normal
        let v0v1 = v1 - v0;
        let v0v2 = v2 - v0;
        // No need to normalize
        let n = v0v1.cross(&v0v2);

        // Back-face test
        if ray.dir.dot(&n) > 0.0 {
            return None;
        }

        let denom = n.dot(&n);

        // Step 1: finding P

        // Check if ray and plane are parallel
        let n_dot_ray_dir = n.dot(&ray.dir);
        if n_dot_ray_dir.abs() < f32::EPSILON {
            // Parallel do not intersect
            return None;
        }
        // Compute d parameter using equation 2
        let d = -n.dot(&v0);

        // Compute t (equation 3)
        let t = -(n.dot(&ray.origin) + d) / n_dot_ray_dir;

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
        if n.dot(&c) < 0.0 {
            return None; // P is on the right side
        }

        // Edge 1
        let edge1 = v2 - v1;
        let vp1 = p - v1;
        let c = edge1.cross(&vp1);
        let u = n.dot(&c);
        if u < 0.0 {
            return None; // P is on the right side
        }

        // Edge 2
        let edge2 = v0 - v2;
        let vp2 = p - v2;
        let c = edge2.cross(&vp2);
        let v = n.dot(&c);
        if v < 0.0 {
            return None; // P is on the right side;
        }

        let uv = Vec2::new(u / denom, v / denom);
        let hit = Hit::new(t, p, uv);
        Some(hit) // This ray hits the triangle
    }

    fn get_color(&self, hit: &Hit) -> RGBA8 {
        // Interpolate vertex colors
        let c0 = &self.vertices[0].color;
        let c1 = &self.vertices[1].color;
        let c2 = &self.vertices[2].color;

        self.material.color * ((1.0 - hit.uv.x - hit.uv.y) * c2 + hit.uv.x * c0 + hit.uv.y * c1)
    }

    fn get_normal(&self, hit: &Hit) -> Vec3 {
        // Interpolate normals
        let n0 = &self.vertices[0].normal;
        let n1 = &self.vertices[1].normal;
        let n2 = &self.vertices[2].normal;

        n2 * (1.0 - hit.uv.x - hit.uv.y) + n0 * hit.uv.x + n1 * hit.uv.y
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn intersect() {
        let material = Material::new();
        let triangle = Triangle::<Vertex>::unit(&material);
        let ray = Ray::new(Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, -1.0));
        assert!(triangle.intersects(&ray).is_some());
        let ray = Ray::new(Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, 1.0));
        assert!(triangle.intersects(&ray).is_none());
    }
}
