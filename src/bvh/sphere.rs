// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

pub struct BvhSphere<'m> {
    center: Point3,
    radius: f32,
    radius2: f32,

    pub trs: &'m Trs,
    pub material: Handle<Material>,
    pub model: &'m Model,
}

impl<'m> BvhSphere<'m> {
    pub fn new(
        center: Point3,
        radius: f32,
        trs: &'m Trs,
        material: Handle<Material>,
        model: &'m Model,
    ) -> Self {
        Self {
            center,
            radius,
            radius2: radius * radius,
            trs,
            material,
            model,
        }
    }

    pub fn get_center(&self) -> Point3 {
        self.trs * self.center
    }

    pub fn get_radius(&self) -> f32 {
        self.radius
    }

    /// This will return a point outside of the sphere, useful for the AABB
    pub fn min(&self) -> Point3 {
        let rad3 = Vec3::splat(self.radius);
        self.get_center() - rad3
    }

    pub fn max(&self) -> Point3 {
        let rad3 = Vec3::splat(self.radius);
        self.get_center() + rad3
    }

    pub fn get_material(&self) -> &Material {
        let material = self
            .model
            .materials
            .get(self.material)
            .unwrap_or(&Material::WHITE);
        material
    }

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
        let hit = Hit::new(self, t0, point, Vec2::default());

        Some(hit)
    }
}

impl<'m> Intersect<'m> for BvhSphere<'m> {
    /// [Ray-sphere intersection](https://www.scratchapixel.com/lessons/3d-basic-rendering/minimal-ray-tracer-rendering-simple-shapes/ray-sphere-intersection)
    fn intersects(&self, ray: &Ray) -> Option<Hit> {
        let ray = ray.clone();
        let inverse = Inversed::from(self.trs);
        let inverse_ray = &inverse * ray;
        let mut hit = self.intersects_impl(&inverse_ray)?;
        let transformed_point = hit.point;
        hit.point = self.trs * transformed_point;
        Some(hit)
    }

    fn get_color(&self, _hit: &Hit) -> Color {
        self.get_material().color
    }

    fn get_metallic_roughness(&self, _hit: &Hit) -> (f32, f32) {
        let uv = Vec2::default();
        self.get_material().get_metallic_roughness(&uv, self.model)
    }

    fn get_normal(&self, hit: &Hit) -> Vec3 {
        (hit.point - self.trs * self.center).get_normalized()
    }
}
