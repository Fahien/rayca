// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::ray::Intersect;

use super::*;

pub struct Sphere {
    center: Vec3,
    _radius: f32,
    radius2: f32,
    pub color: u32,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32, color: u32) -> Self {
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
            color: 0xFFFFFFFFu32,
        }
    }
}

impl Intersect for Sphere {
    /// [Ray-sphere intersection](https://www.scratchapixel.com/lessons/3d-basic-rendering/minimal-ray-tracer-rendering-simple-shapes/ray-sphere-intersection)
    fn intersects(&self, ray: &Ray) -> bool {
        let l = self.center - ray.origin;
        // angle between sphere-center-to-ray-origin and ray-direction
        let tca = l.dot(&ray.dir);
        if tca < 0.0 {
            return false;
        }

        let d2 = l.dot(&l) - tca * tca;
        if d2 > self.radius2 {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn intersect() {
        let orig = Vec3::new(0.0, 0.0, 0.0);
        let sphere = Sphere::new(orig, 1.0, 0xFFFFFFFFu32);

        let right = Vec3::new(1.0, 0.0, 0.0);
        let ray = Ray::new(orig, right);
        assert!(sphere.intersects(&ray));

        let ray = Ray::new(Vec3::new(2.0, 0.0, 0.0), right);
        assert!(!sphere.intersects(&ray));

        let sphere = Sphere::new(Vec3::new(4.0, 0.0, 0.0), 1.0, 0xFFFFFFFFu32);
        let forward = Vec3::new(0.0, 0.0, -1.0);
        let ray = Ray::new(orig, forward);
        assert!(!sphere.intersects(&ray));
    }
}
