// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

pub struct BvhTriangle<'m> {
    pub vertices: [Vertex; 3],
    pub centroid: Point3,
    pub material: Handle<Material>,
    pub model: &'m Model,
}

impl<'m> BvhTriangle<'m> {
    pub fn new(
        a: Vertex,
        b: Vertex,
        c: Vertex,
        material: Handle<Material>,
        model: &'m Model,
    ) -> Self {
        let centroid = (a.pos + Vec3::from(b.pos) + Vec3::from(c.pos)) * 0.3333;
        Self {
            vertices: [a, b, c],
            centroid,
            material,
            model,
        }
    }

    pub fn unit(material: Handle<Material>, model: &'m Model) -> Self {
        Self::new(
            Vertex::new(-1.0, 0.0, 0.0),
            Vertex::new(1.0, 0.0, 0.0),
            Vertex::new(0.0, 1.0, 0.0),
            material,
            model,
        )
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
}

impl<'m> Intersect for BvhTriangle<'m> {
    /// [Ray-triangle intersection](https://www.scratchapixel.com/lessons/3d-basic-rendering/ray-tracing-rendering-a-triangle/ray-triangle-intersection-geometric-solution)
    fn intersects(&self, ray: &Ray) -> Option<Hit> {
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

        let denom = n.dot(&n);

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
        let hit = Hit::new(t, p, uv);
        Some(hit) // This ray hits the triangle
    }

    fn get_color(&self, hit: &Hit) -> Color {
        // Interpolate vertex colors
        let c0 = &self.vertices[0].color;
        let c1 = &self.vertices[1].color;
        let c2 = &self.vertices[2].color;

        let white_material = Material::default();
        let material = self
            .model
            .materials
            .get(self.material)
            .unwrap_or(&white_material);

        let mut color = material.color;

        if let Some(albedo_texture) = self.model.textures.get(material.albedo) {
            let sampler = Sampler::default();
            let image = self.model.images.get(albedo_texture.image).unwrap();

            let uvs = [
                &self.vertices[0].uv,
                &self.vertices[1].uv,
                &self.vertices[2].uv,
            ];
            let uv = uvs[2] * (1.0 - hit.uv.x - hit.uv.y) + uvs[0] * hit.uv.x + uvs[1] * hit.uv.y;
            color = color * sampler.sample(image, &uv);
        }

        color * ((1.0 - hit.uv.x - hit.uv.y) * c2 + hit.uv.x * c0 + hit.uv.y * c1)
    }

    fn get_normal(&self, hit: &Hit) -> Vec3 {
        // Interpolate normals
        let n0 = &self.vertices[0].normal;
        let n1 = &self.vertices[1].normal;
        let n2 = &self.vertices[2].normal;

        let mut n = n2 * (1.0 - hit.uv.x - hit.uv.y) + n0 * hit.uv.x + n1 * hit.uv.y;
        n.normalize();
        n
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
        let triangles = triangle_prim.triangles(&Trs::default(), material, &model);
        let triangle_ref = &triangles[0];

        let ray = Ray::new(Point3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, -1.0));
        assert!(triangle_ref.intersects(&ray).is_some());
        let ray = Ray::new(Point3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, 1.0));
        assert!(triangle_ref.intersects(&ray).is_none());
    }
}