// Copyright © 2022-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::{
    ray::Intersect, Bvh, Color, Dot, Handle, Hit, Inversed, Point3, Ray, Scene, Shade, SolvedTrs,
    Vec2, Vec3, RGBA8,
};

#[derive(Debug, Clone)]
pub struct Sphere {
    center: Point3,
    radius: f32,
    radius2: f32,
    trs: Handle<SolvedTrs>,
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

    /// Ray is expected to be in model space
    pub fn intersects_impl(&self, ray: &Ray) -> Option<Hit> {
        let l = self.center - ray.origin;
        // angle between sphere-center-to-ray-origin and ray-direction
        let tca = l.dot(&ray.dir);
        if tca < 0.0 {
            return None;
        }

        let d2 = l.dot(&l) - tca * tca;
        if d2 > self.radius2 {
            return None;
        }

        let thc = (self.radius2 - d2).sqrt();
        let mut t0 = tca - thc;
        let mut t1 = tca + thc;

        if t0 > t1 {
            std::mem::swap(&mut t0, &mut t1);
        }

        if t0 < 0.0 {
            t0 = t1;
            if t0 < 0.0 {
                return None;
            }
        }

        let point = ray.origin + ray.dir * t0;
        let hit = Hit::new(u32::MAX, u32::MAX, t0, point, Vec2::default());

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

pub struct SphereEx {
    pub color: RGBA8,
}

impl Default for SphereEx {
    fn default() -> Self {
        Self {
            color: RGBA8::WHITE,
        }
    }
}

impl SphereEx {
    pub fn new(color: RGBA8) -> Self {
        Self { color }
    }
}

impl Shade for SphereEx {
    fn get_color(&self, scene: &Scene, hit: &Hit) -> Color {
        let normal = self.get_normal(scene, hit);
        Color::from(normal)
    }

    fn get_normal(&self, scene: &Scene, hit: &Hit) -> Vec3 {
        // TODO: Make it onliner: scene.get_sphere(hit);
        let blas_node = &scene.tlas.blas_nodes[hit.blas as usize];
        let bvh = scene.tlas.bvhs.get(blas_node.bvh).unwrap();
        let sphere = bvh.get_sphere(hit.primitive);
        let mut normal = hit.point - sphere.get_center(bvh);
        normal.normalize();
        normal
    }

    fn get_metallic_roughness(&self, _scene: &Scene, _hit: &Hit) -> (f32, f32) {
        (1.0, 1.0)
    }
}
