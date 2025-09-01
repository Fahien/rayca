// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::simd::{f32x4, num::SimdFloat};

use crate::*;

#[repr(C)]
#[derive(Default, Clone)]
pub struct AABB {
    pub a: Point3,
    pub b: Point3,
}

impl AABB {
    pub fn new(a: Point3, b: Point3) -> Self {
        Self { a, b }
    }

    pub fn area(&self) -> f32 {
        let e = self.b - self.a; // box extent
        e.simd[0] * e.simd[1] + e.simd[1] * e.simd[2] + e.simd[2] * e.simd[0]
    }

    fn grow(&mut self, p: Point3) {
        self.a = self.a.min(p);
        self.b = self.b.max(p);
    }

    fn grow_triangle(&mut self, triangle: &BvhTriangle, trs: &Trs) {
        self.grow(triangle.get_vertex(0, trs));
        self.grow(triangle.get_vertex(1, trs));
        self.grow(triangle.get_vertex(2, trs));
    }

    fn grow_sphere(&mut self, sphere: &Sphere, trs: &Trs) {
        let radius = sphere.get_radius(trs);
        let center = sphere.get_center(trs);
        self.grow(center + Vec3::new(-radius, 0.0, 0.0));
        self.grow(center + Vec3::new(radius, 0.0, 0.0));
        self.grow(center + Vec3::new(0.0, -radius, 0.0));
        self.grow(center + Vec3::new(0.0, radius, 0.0));
        self.grow(center + Vec3::new(0.0, 0.0, -radius));
        self.grow(center + Vec3::new(0.0, 0.0, radius));
    }

    pub fn grow_range(
        &mut self,
        blas: &Blas,
        range: BvhRange<BvhPrimitive>,
        scene: &SceneDrawInfo,
    ) {
        // Visits each primitive to find the lowest and highest x, y, and z
        for i in range.to_range() {
            let prim = &blas.model.primitives[i];
            self.a = self.a.min(prim.min(scene));
            self.b = self.b.max(prim.max(scene));
        }
    }

    pub fn grow_primitive(&mut self, scene: &SceneDrawInfo, primitive: &BvhPrimitive) {
        let trs = scene.get_world_trs(primitive.node);
        match &primitive.geometry {
            BvhGeometry::Triangle(triangle) => {
                self.grow_triangle(triangle, &trs.trs);
            }
            BvhGeometry::Sphere(sphere) => {
                self.grow_sphere(sphere, &trs.trs);
            }
        }
    }

    /// Slab test. We do not care where we hit the box; only info we need is a yes/no answer.
    pub fn intersects(&self, ray: &Ray) -> f32 {
        let origin_vec = Vec3::from(ray.origin);
        let t1 = (self.a - origin_vec) * ray.rdir;
        let t2 = (self.b - origin_vec) * ray.rdir;

        static WMAX: f32x4 = f32x4::from_array([1.0, 1.0, 1.0, f32::MAX]);
        static WMIN: f32x4 = f32x4::from_array([1.0, 1.0, 1.0, f32::MIN]);

        let vmax = t1.max(t2).simd * WMAX;
        let vmin = t1.min(t2).simd * WMIN;

        let tmax = vmax.reduce_min();
        let tmin = vmin.reduce_max();

        if tmax >= tmin && tmax > 0.0 {
            tmin
        } else {
            f32::MAX
        }
    }

    pub fn get_centroid(&self) -> Point3 {
        ((self.b - self.a) / 2.0).into()
    }
}
