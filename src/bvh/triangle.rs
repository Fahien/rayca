// Copyright © 2022-2024
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

pub struct BvhTriangle {
    pub vertices: [Vertex; 3],
    pub centroid: Point3,
}

impl BvhTriangle {
    pub fn new(a: Vertex, b: Vertex, c: Vertex) -> Self {
        let centroid =
            Point3::from((Vec3::from(a.pos) + Vec3::from(b.pos) + Vec3::from(c.pos)) * 0.3333);
        Self {
            vertices: [a, b, c],
            centroid,
        }
    }

    pub fn min(&self) -> Point3 {
        Point3::new(f32::MAX, f32::MAX, f32::MAX)
            .min(&self.vertices[0].pos)
            .min(&self.vertices[1].pos)
            .min(&self.vertices[2].pos)
    }

    pub fn max(&self) -> Point3 {
        Point3::new(f32::MIN, f32::MIN, f32::MIN)
            .max(&self.vertices[0].pos)
            .max(&self.vertices[1].pos)
            .max(&self.vertices[2].pos)
    }

    /// Returns the interpolation of the vertices colors
    pub fn interpolate_colors(&self, hit_uv: &Vec2) -> Color {
        self.vertices[2].color * (1.0 - hit_uv.x - hit_uv.y)
            + self.vertices[0].color * hit_uv.x
            + self.vertices[1].color * hit_uv.y
    }

    /// Returns the interpolation of the vertices uvs
    pub fn interpolate_uvs(&self, hit_uv: &Vec2) -> Vec2 {
        self.vertices[2].uv * (1.0 - hit_uv.x - hit_uv.y)
            + self.vertices[0].uv * hit_uv.x
            + self.vertices[1].uv * hit_uv.y
    }

    /// Returns the interpolation of the vertices normals
    pub fn interpolate_normals(&self, hit_uv: &Vec2) -> Vec3 {
        let n = self.vertices[2].normal * (1.0 - hit_uv.x - hit_uv.y)
            + self.vertices[0].normal * hit_uv.x
            + self.vertices[1].normal * hit_uv.y;
        n.get_normalized()
    }

    /// Returns the interpolation of the vertices tangents
    pub fn interpolate_tangents(&self, hit_uv: &Vec2) -> Vec3 {
        let mut t = self.vertices[2].tangent * (1.0 - hit_uv.x - hit_uv.y)
            + self.vertices[0].tangent * hit_uv.x
            + self.vertices[1].tangent * hit_uv.y;
        t.normalize();
        t
    }

    /// Returns the interpolation of the vertices bitangents
    pub fn interpolate_bitangents(&self, hit_uv: &Vec2) -> Vec3 {
        let mut b = self.vertices[2].bitangent * (1.0 - hit_uv.x - hit_uv.y)
            + self.vertices[0].bitangent * hit_uv.x
            + self.vertices[1].bitangent * hit_uv.y;
        b.normalize();
        b
    }

    /// [Ray-triangle intersection](https://www.scratchapixel.com/lessons/3d-basic-rendering/ray-tracing-rendering-a-triangle/ray-triangle-intersection-geometric-solution)
    pub fn intersects(&self, ray: &Ray) -> Option<Hit> {
        let v0 = Vec3::from(self.vertices[0].pos);
        let v1 = Vec3::from(self.vertices[1].pos);
        let v2 = Vec3::from(self.vertices[2].pos);

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
        let d = -n.dot(v0);

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
        let hit = Hit::new(t, p, uv);
        Some(hit) // This ray hits the triangle
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn intersect() {
        let mut model = Model::new();
        let material = model.materials.push(Material::new());
        let triangle_prim = Primitive::unit_triangle();
        let node = model.nodes.push(Node::default());
        let triangles = triangle_prim.primitives(node, material, &model);
        let triangle_ref = &triangles[0];

        let ray = Ray::new(Point3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, -1.0));
        assert!(triangle_ref.intersects(&model, &ray).is_some());
        let ray = Ray::new(Point3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, 1.0));
        assert!(triangle_ref.intersects(&model, &ray).is_none());
    }
}
