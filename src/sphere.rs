// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::ray::Intersect;

use super::*;

pub struct Sphere {
    center: Vec3,
    _radius: f32,
    radius2: f32,
    pub color: RGBA8,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32, color: RGBA8) -> Self {
        let radius2 = radius * radius;
        Self {
            center,
            _radius: radius,
            radius2,
            color,
        }
    }
}

impl Default for Sphere {
    fn default() -> Self {
        Self {
            center: Default::default(),
            _radius: 1.0,
            radius2: 1.0,
            color: RGBA8::from(0xFFFFFFFFu32),
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
        let hit = Hit::new(t0, point, Vec2::default());

        Some(hit)
    }

    fn get_color(&self, hit: &Hit) -> Color {
        let normal = self.get_normal(&hit);
        Color::new(normal.x, normal.y, normal.z, 1.0)
    }

    fn get_normal(&self, hit: &Hit) -> Vec3 {
        let mut normal = hit.point - self.center;
        normal.normalize();
        normal
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn intersect() {
        let orig = Vec3::new(0.0, 0.0, 0.0);
        let sphere = Sphere::new(orig, 1.0, RGBA8::from(0xFFFFFFFFu32));

        let right = Vec3::new(1.0, 0.0, 0.0);
        let ray = Ray::new(orig, right);
        assert!(sphere.intersects(&ray).is_some());

        let ray = Ray::new(Vec3::new(2.0, 0.0, 0.0), right);
        assert!(sphere.intersects(&ray).is_none());

        let sphere = Sphere::new(Vec3::new(4.0, 0.0, 0.0), 1.0, RGBA8::from(0xFFFFFFFFu32));
        let forward = Vec3::new(0.0, 0.0, -1.0);
        let ray = Ray::new(orig, forward);
        assert!(sphere.intersects(&ray).is_none());
    }
}
