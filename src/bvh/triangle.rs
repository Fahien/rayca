// Copyright © 2022-2024
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

    /// Returns the interpolation of the vertices colors
    pub fn interpolate_colors(&self, hit: &Hit) -> Color {
        self.vertices[2].color * (1.0 - hit.uv.x - hit.uv.y)
            + self.vertices[0].color * hit.uv.x
            + self.vertices[1].color * hit.uv.y
    }

    /// Returns the interpolation of the vertices uvs
    pub fn interpolate_uvs(&self, hit: &Hit) -> Vec2 {
        self.vertices[2].uv * (1.0 - hit.uv.x - hit.uv.y)
            + self.vertices[0].uv * hit.uv.x
            + self.vertices[1].uv * hit.uv.y
    }

    /// Returns the interpolation of the vertices normals
    pub fn interpolate_normals(&self, hit_uv: &Vec2) -> Vec3 {
        let n = self.vertices[2].normal * (1.0 - hit_uv.x - hit_uv.y)
            + self.vertices[0].normal * hit_uv.x
            + self.vertices[1].normal * hit_uv.y;
        n.get_normalized()
    }

    pub fn get_material(&self) -> &Material {
        let material = self
            .model
            .materials
            .get(self.material)
            .unwrap_or(&Material::WHITE);
        material
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
        let vp0 = p - v0;
        // Vector perpendicular to triangle's plane
        let c = edge0.cross(&vp0.into());
        if n.dot(c) < 0.0 {
            return None; // P is on the right side
        }

        // Edge 1
        let edge1 = v2 - v1;
        let vp1 = p - v1;
        let c = edge1.cross(&vp1.into());
        let u = n.dot(c);
        if u < 0.0 {
            return None; // P is on the right side
        }

        // Edge 2
        let edge2 = v0 - v2;
        let vp2 = p - v2;
        let c = edge2.cross(&vp2.into());
        let v = n.dot(c);
        if v < 0.0 {
            return None; // P is on the right side;
        }

        let uv = Vec2::new(u / denom, v / denom);
        let hit = Hit::new(t, p, uv);
        Some(hit) // This ray hits the triangle
    }

    fn get_color(&self, hit: &Hit) -> Color {
        let material = self.get_material();
        let mut color = self.interpolate_colors(hit) * material.color;
        if let Some(albedo_texture) = self.model.textures.get(material.albedo_texture) {
            let sampler = Sampler::default();
            let image = self.model.images.get(albedo_texture.image).unwrap();
            color *= sampler.sample(image, &self.interpolate_uvs(hit));
        }
        color
    }

    fn get_normal(&self, hit: &Hit) -> Vec3 {
        self.interpolate_normals(&hit.uv)
    }

    fn get_metallic_roughness(&self, hit: &Hit) -> (f32, f32) {
        let material = self.get_material();
        if let Some(mr_texture) = self.model.textures.get(material.metallic_roughness_texture) {
            let sampler = Sampler::default();
            let image = self.model.images.get(mr_texture.image).unwrap();
            let color = sampler.sample(image, &self.interpolate_uvs(hit));
            // Blue channel contains metalness value
            // Red channel contains roughness value
            (color.b, color.r)
        } else {
            (material.metallic_factor, material.roughness_factor)
        }
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
