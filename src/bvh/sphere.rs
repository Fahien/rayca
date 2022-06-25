// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

pub struct BvhSphere {
    pub center: Point3,
    radius: f32,
    radius2: f32,
}

impl BvhSphere {
    pub fn new(mut center: Point3, radius: f32) -> Self {
        center.simd[3] = 1.0;
        Self {
            center,
            radius,
            radius2: radius * radius,
        }
    }

    pub fn get_radius(&self) -> f32 {
        self.radius
    }

    /// This will return a point outside of the sphere, useful for the AABB
    pub fn min(&self) -> Point3 {
        let rad3 = Vec3::splat(self.radius);
        self.center - rad3
    }

    pub fn max(&self) -> Point3 {
        let rad3 = Vec3::splat(self.radius);
        self.center + rad3
    }

    /// [Ray-sphere intersection](https://www.scratchapixel.com/lessons/3d-basic-rendering/minimal-ray-tracer-rendering-simple-shapes/ray-sphere-intersection)
    pub fn intersects(&self, ray: &Ray) -> Option<Hit> {
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
}
