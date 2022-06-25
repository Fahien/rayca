// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::ray::Intersect;

use crate::*;

#[derive(Debug, Clone)]
pub struct Sphere {
    pub center: Point3,
    radius: f32,
    radius2: f32,
}

impl Sphere {
    pub fn new(center: Point3, radius: f32) -> Self {
        let radius2 = radius * radius;
        Self {
            center,
            radius,
            radius2,
        }
    }

    pub fn set_radius(&mut self, radius: f32) {
        self.radius = radius;
        self.radius2 = radius * radius;
    }

    pub fn get_radius(&self) -> f32 {
        self.radius
    }

    pub fn primitives(&self, node: Handle<Node>, material: Handle<Material>) -> Vec<BvhPrimitive> {
        // Transforming a sphere is complicated. The trick is to store transform with sphere,
        // then pre-transform the ray, and post-transform the intersection point.
        let sphere = BvhSphere::new(self.center, self.radius);
        let geometry = BvhGeometry::Sphere(sphere);
        let primitive = BvhPrimitive::new(geometry, node, material);

        vec![primitive]
    }
}

impl Default for Sphere {
    fn default() -> Self {
        Self {
            center: Default::default(),
            radius: 1.0,
            radius2: 1.0,
        }
    }
}

impl Intersect for Sphere {
    /// [Ray-sphere intersection](https://www.scratchapixel.com/lessons/3d-basic-rendering/minimal-ray-tracer-rendering-simple-shapes/ray-sphere-intersection)
    fn intersects(&self, ray: &Ray) -> Option<Hit> {
        let l = self.center - ray.origin;
        // angle between sphere-center-to-ray-origin and ray-direction
        let tca = l.dot(ray.dir);
        if tca < 0.0 {
            return None;
        }

        let d2 = l.dot(l) - tca * tca;
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

    fn get_color(&self, material: &Material, model: &Model, hit: &Hit) -> Color {
        let normal = self.get_normal(material, model, hit);
        Color::from(normal)
    }

    fn get_normal(&self, _material: &Material, _model: &Model, hit: &Hit) -> Vec3 {
        let mut normal = hit.point - self.center;
        normal.normalize();
        normal
    }

    fn get_metallic_roughness(
        &self,
        _material: &Material,
        _model: &Model,
        _hit: &Hit,
    ) -> (f32, f32) {
        (1.0, 1.0) // glTF default values
    }
}

#[cfg(test)]
mod test {
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
