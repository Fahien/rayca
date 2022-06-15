// Copyright © 2022-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::{
    Color, Dot, GgxMaterial, GltfModel, Handle, Hit, Intersect, Mat3, Point3, Ray, Sampler, Scene,
    Shade, Vec2, Vec3, Vertex,
};

pub struct Triangle {
    pub vertices: [Point3; 3],
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

        // A triangle does not know its primitive index
        // Setting the primitive index is responsiblity of the caller
        let hit = Hit::new(u32::MAX, u32::MAX, t, p, uv);
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

    /// Returns the interpolation of the vertices tangents
    pub fn interpolate_tangents(&self, hit: &Hit) -> Vec3 {
        let mut t = self.vertices[2].tangent * (1.0 - hit.uv.x - hit.uv.y)
            + self.vertices[0].tangent * hit.uv.x
            + self.vertices[1].tangent * hit.uv.y;
        t.normalize();
        t
    }

    /// Returns the interpolation of the vertices bitangents
    pub fn interpolate_bitangents(&self, hit: &Hit) -> Vec3 {
        let mut b = self.vertices[2].bitangent * (1.0 - hit.uv.x - hit.uv.y)
            + self.vertices[0].bitangent * hit.uv.x
            + self.vertices[1].bitangent * hit.uv.y;
        b.normalize();
        b
    }

    pub fn get_material<'a>(&self, model: &'a GltfModel) -> &'a GgxMaterial {
        model
            .materials
            .get(self.material)
            .unwrap_or(&GgxMaterial::WHITE)
    }
}

impl Shade for TriangleEx {
    fn get_color(&self, scene: &Scene, hit: &Hit) -> Color {
        // TODO: Make onliner?
        let blas_node = &scene.tlas.blas_nodes[hit.blas as usize];
        let model = scene.gltf_models.get(blas_node.model).unwrap();
        let material = self.get_material(model);
        let mut color = self.interpolate_colors(hit) * material.color;

        if let Some(texture) = model.textures.get(material.albedo_texture) {
            let sampler = Sampler::default();
            let image = model.images.get(texture.image).unwrap();
            color *= sampler.sample(image, &self.interpolate_uvs(hit));
        }
        color
    }

    fn get_normal(&self, scene: &Scene, hit: &Hit) -> Vec3 {
        let normal = self.interpolate_normals(&hit.uv);

        // TODO make it oneliner
        let blas_node = &scene.tlas.blas_nodes[hit.blas as usize];
        let model = scene.gltf_models.get(blas_node.model).unwrap();

        let material = self.get_material(model);
        if let Some(texture) = model.textures.get(material.normal_texture) {
            let sampler = Sampler::default();
            let image = model.images.get(texture.image).unwrap();
            let mut sampled_normal = Vec3::from(sampler.sample(image, &self.interpolate_uvs(hit)));
            sampled_normal = sampled_normal * 2.0 - 1.0;

            let tangent = self.interpolate_tangents(hit);
            let bitangent = self.interpolate_bitangents(hit);

            let tbn = Mat3::tbn(&tangent, &bitangent, &normal);
            (&tbn * sampled_normal).get_normalized()
        } else {
            normal
        }
    }

    fn get_metallic_roughness(&self, scene: &Scene, hit: &Hit) -> (f32, f32) {
        // TODO: Make onliner?
        let blas_node = &scene.tlas.blas_nodes[hit.blas as usize];
        let model = scene.gltf_models.get(blas_node.model).unwrap();
        let material = self.get_material(model);
        if let Some(texture) = model.textures.get(material.metallic_roughness_texture) {
            let sampler = Sampler::default();
            let image = model.images.get(texture.image).unwrap();
            let color = sampler.sample(image, &self.interpolate_uvs(hit));
            // Blue channel contains metalness value
            // Red channel contains roughness value
            (color.b, color.r)
        } else {
            (material.metallic_factor, material.roughness_factor)
        }
    }
}
