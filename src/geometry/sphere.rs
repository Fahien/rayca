// Copyright © 2022-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::{
    ray::Intersect, Bvh, Color, Dot, GgxMaterial, GltfModel, Handle, Hit, Inversed, Mat3, Point3,
    Ray, Scene, Shade, SolvedTrs, Vec2, Vec3, RGBA8,
};

#[derive(Debug, Clone)]
pub struct Sphere {
    center: Point3,
    radius: f32,
    radius2: f32,
    pub trs: Handle<SolvedTrs>,
}

impl Sphere {
    pub fn new(center: Point3, radius: f32) -> Self {
        let radius2 = radius * radius;
        Self {
            center,
            radius,
            radius2,
            trs: Handle::NONE,
        }
    }

    pub fn get_center(&self, bvh: &Bvh) -> Point3 {
        let solved_trs = bvh.trss.get(self.trs).unwrap_or(&SolvedTrs::IDENTITY);
        &solved_trs.trs * self.center
    }

    pub fn set_radius(&mut self, radius: f32) {
        self.radius = radius;
        self.radius2 = radius * radius;
    }

    pub fn get_radius(&self) -> f32 {
        self.radius
    }

    /// - `point`: should be in model space
    fn get_normal_impl(&self, point: &Point3) -> Vec3 {
        (point - self.center).get_normalized()
    }

    /// Geometric formula.
    /// - `ray`: Should be in model space
    pub fn intersects_impl(&self, ray: &Ray) -> Option<Hit> {
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
        let hit = Hit::new(u32::MAX, u32::MAX, t, point, Vec2::default());
        Some(hit)
    }
}

impl Default for Sphere {
    fn default() -> Self {
        Self {
            center: Default::default(),
            radius: 1.0,
            radius2: 1.0,
            trs: Default::default(),
        }
    }
}

impl Intersect for Sphere {
    /// [Ray-sphere intersection](https://www.scratchapixel.com/lessons/3d-basic-rendering/minimal-ray-tracer-rendering-simple-shapes/ray-sphere-intersection)
    fn intersects(&self, bvh: &Bvh, ray: &Ray) -> Option<Hit> {
        let trs = &bvh.trss.get(self.trs).unwrap_or(&SolvedTrs::IDENTITY).trs;
        let ray = ray.clone();
        let inverse = Inversed::from(trs);
        let inverse_ray = &inverse * ray;
        let mut hit = self.intersects_impl(&inverse_ray)?;
        let transformed_point = hit.point;
        hit.point = trs * transformed_point;
        Some(hit)
    }

    fn get_centroid(&self, bvh: &Bvh) -> Vec3 {
        self.get_center(bvh).into()
    }

    /// This will return a point outside of the sphere, useful for the AABB
    fn min(&self, bvh: &Bvh) -> Point3 {
        let rad3 = Vec3::splat(self.radius);
        self.get_center(bvh) - rad3
    }

    fn max(&self, bvh: &Bvh) -> Point3 {
        let rad3 = Vec3::splat(self.radius);
        self.get_center(bvh) + rad3
    }
}

#[cfg(test)]
mod test {
    use crate::Vec3;

    use super::*;

    #[test]
    fn intersect() {
        let bvh = Bvh::default();

        let orig = Point3::new(0.0, 0.0, 0.0);
        let sphere = Sphere::new(orig, 1.0);

        let right = Vec3::new(1.0, 0.0, 0.0);
        let ray = Ray::new(orig, right);
        assert!(sphere.intersects(&bvh, &ray).is_some());

        let ray = Ray::new(Point3::new(2.0, 0.0, 0.0), right);
        assert!(sphere.intersects(&bvh, &ray).is_none());

        let sphere = Sphere::new(Point3::new(4.0, 0.0, 0.0), 1.0);
        let forward = Vec3::new(0.0, 0.0, -1.0);
        let ray = Ray::new(orig, forward);
        assert!(sphere.intersects(&bvh, &ray).is_none());
    }
}

#[derive(Debug, Clone)]
pub struct SphereEx {
    pub color: RGBA8,
    pub material: Handle<GgxMaterial>,
}

impl Default for SphereEx {
    fn default() -> Self {
        Self {
            color: RGBA8::WHITE,
            material: Handle::none(),
        }
    }
}

impl SphereEx {
    pub fn new(color: RGBA8, material: Handle<GgxMaterial>) -> Self {
        Self { color, material }
    }

    pub fn get_material<'a>(&self, model: &'a GltfModel) -> &'a GgxMaterial {
        model
            .materials
            .get(self.material)
            .unwrap_or(&GgxMaterial::WHITE)
    }
}

impl Shade for SphereEx {
    fn get_color(&self, scene: &Scene, hit: &Hit) -> Color {
        let blas_node = &scene.tlas.blas_nodes[hit.blas as usize];
        let model = scene.gltf_models.get(blas_node.model).unwrap();
        self.get_material(model).color
    }

    fn get_normal(&self, scene: &Scene, hit: &Hit) -> Vec3 {
        // TODO: Make it onliner: scene.get_sphere(hit);
        let blas_node = &scene.tlas.blas_nodes[hit.blas as usize];
        let bvh = scene.tlas.bvhs.get(blas_node.bvh).unwrap();
        let sphere = bvh.get_sphere(hit.primitive);
        let inverse = bvh.get_trs(sphere.trs).get_inversed();
        let hit_point = &inverse * hit.point;
        let normal = sphere.get_normal_impl(&hit_point);
        let normal_matrix = Mat3::from(&inverse).get_transpose();
        (&normal_matrix * normal).get_normalized()
    }

    fn get_metallic_roughness(&self, _scene: &Scene, _hit: &Hit) -> (f32, f32) {
        (1.0, 1.0)
    }
}
