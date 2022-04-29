// Copyright © 2022-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::{
    Color, Dot, GgxMaterial, Handle, Hit, Intersect, Point3, Ray, Sampler, Scene, Vec2, Vec3,
    Vertex,
};

pub struct Triangle {
    vertices: [Point3; 3],
    pub centroid: Vec3,
}

impl Triangle {
    pub fn new(a: Point3, b: Point3, c: Point3) -> Self {
        let centroid = (Vec3::from(a) + Vec3::from(b) + Vec3::from(c)) * 0.3333;
        Self {
            vertices: [a, b, c],
            centroid,
        }
    }

    pub fn get_vertex_mut(&mut self, index: usize) -> &mut Point3 {
        &mut self.vertices[index]
    }

    pub fn min(&self) -> Point3 {
        Point3::new(f32::MAX, f32::MAX, f32::MAX)
            .min(&self.vertices[0])
            .min(&self.vertices[1])
            .min(&self.vertices[2])
    }

    pub fn max(&self) -> Point3 {
        Point3::new(f32::MIN, f32::MIN, f32::MIN)
            .max(&self.vertices[0])
            .max(&self.vertices[1])
            .max(&self.vertices[2])
    }
}

impl Default for Triangle {
    fn default() -> Self {
        Self::new(
            Point3::new(-1.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        )
    }
}

impl Intersect for Triangle {
    /// [Ray-triangle intersection](https://www.scratchapixel.com/lessons/3d-basic-rendering/ray-tracing-rendering-a-triangle/ray-triangle-intersection-geometric-solution)
    fn intersects(&self, ray: &Ray) -> Option<Hit> {
        let v0 = Vec3::from(self.vertices[0]);
        let v1 = Vec3::from(self.vertices[1]);
        let v2 = Vec3::from(self.vertices[2]);

        // Plane's normal
        let v0v1 = v1 - v0;
        let v0v2 = v2 - v0;
        // No need to normalize
        let n = v0v1.cross(&v0v2);

        // Back-face test
        if ray.dir.dot(n) > 0.0 {
            return None;
        }

        let denom = n.dot(&n);

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
        let t = -(n.dot(&Vec3::from(ray.origin)) + d) / n_dot_ray_dir;

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

        // A triangle does not know its primitive index
        // Setting the primitive index is responsiblity of the caller
        let hit = Hit::new(u32::MAX, t, p, uv);
        Some(hit) // This ray hits the triangle
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

#[derive(Default)]
pub struct TriangleEx {
    pub vertices: [Vertex; 3],
    pub material: Handle<GgxMaterial>,
}

impl TriangleEx {
    pub fn new(a: Vertex, b: Vertex, c: Vertex, material: Handle<GgxMaterial>) -> Self {
        Self {
            vertices: [a, b, c],
            material,
        }
    }

    pub fn get_color(&self, hit: &Hit, scene: &Scene) -> Color {
        // Interpolate vertex colors
        let c0 = &self.vertices[0].color;
        let c1 = &self.vertices[1].color;
        let c2 = &self.vertices[2].color;
        let vertex_color = (1.0 - hit.uv.x - hit.uv.y) * c2 + hit.uv.x * c0 + hit.uv.y * c1;

        let material = if self.material.valid() {
            scene.gltf_model.materials.get(self.material).unwrap()
        } else {
            &GgxMaterial::WHITE
        };
        let mut color = material.color;

        if let Some(texture) = scene.gltf_model.textures.get(material.albedo) {
            let sampler = Sampler::default();
            let image = scene.gltf_model.images.get(texture.image).unwrap();

            let uvs = [
                &self.vertices[0].uv,
                &self.vertices[1].uv,
                &self.vertices[2].uv,
            ];
            let uv = uvs[2] * (1.0 - hit.uv.x - hit.uv.y) + uvs[0] * hit.uv.x + uvs[1] * hit.uv.y;
            color = color * sampler.sample(image, &uv);
        }

        color * vertex_color
    }

    pub fn get_normal(&self, hit: &Hit) -> Vec3 {
        // Interpolate normals
        let n0 = &self.vertices[0].normal;
        let n1 = &self.vertices[1].normal;
        let n2 = &self.vertices[2].normal;

        n2 * (1.0 - hit.uv.x - hit.uv.y) + n0 * hit.uv.x + n1 * hit.uv.y
    }
}
