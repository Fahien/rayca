// Copyright © 2022-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::{ray::Intersect, Color, Dot, Hit, Point3, Ray, Scene, Shade, Vec2, Vec3, RGBA8};

pub struct Sphere {
    center: Point3,
    _radius: f32,
    radius2: f32,
}

impl Sphere {
    pub fn new(center: Point3, radius: f32) -> Self {
        let radius2 = radius * radius;
        Self {
            center,
            _radius: radius,
            radius2,
        }
    }
}

impl Default for Sphere {
    fn default() -> Self {
        Self {
            center: Default::default(),
            _radius: 1.0,
            radius2: 1.0,
        }
    }
}

impl Intersect for Sphere {
    /// [Ray-sphere intersection](https://www.scratchapixel.com/lessons/3d-basic-rendering/minimal-ray-tracer-rendering-simple-shapes/ray-sphere-intersection)
    fn intersects(&self, ray: &Ray) -> Option<Hit> {
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

#[cfg(test)]
mod test {
    use crate::Vec3;

    use super::*;

    #[test]
    fn intersect() {
        let orig = Point3::new(0.0, 0.0, 0.0);
        let sphere = Sphere::new(orig, 1.0);

        let right = Vec3::new(1.0, 0.0, 0.0);
        let ray = Ray::new(orig, right);
        assert!(sphere.intersects(&ray).is_some());

        let ray = Ray::new(Point3::new(2.0, 0.0, 0.0), right);
        assert!(sphere.intersects(&ray).is_none());

        let sphere = Sphere::new(Point3::new(4.0, 0.0, 0.0), 1.0);
        let forward = Vec3::new(0.0, 0.0, -1.0);
        let ray = Ray::new(orig, forward);
        assert!(sphere.intersects(&ray).is_none());
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
    fn get_color(&self, _scene: &Scene, _hit: &Hit) -> Color {
        self.color.into()
    }

    fn get_normal(&self, scene: &Scene, hit: &Hit) -> Vec3 {
        // TODO: Make it onliner: scene.get_sphere(hit);
        let blas_node = &scene.tlas.blas_nodes[hit.blas as usize];
        let bvh = scene.tlas.bvhs.get(blas_node.bvh).unwrap();
        let sphere = bvh.get_sphere(hit.primitive);
        let mut normal = hit.point - sphere.center;
        normal.normalize();
        normal
    }

    fn get_metalness(&self, _scene: &Scene, _hit: &Hit) -> f32 {
        1.0
    }
}
