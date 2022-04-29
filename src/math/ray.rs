// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use super::*;

#[derive(Debug)]
pub struct Ray {
    pub origin: Point3,
    pub dir: Vec3,
}

impl Ray {
    pub fn new(origin: Point3, dir: Vec3) -> Self {
        Self { origin, dir }
    }

    pub fn translate(&mut self, translation: &Vec3) {
        self.origin += translation;
    }

    pub fn rotate(&mut self, rotation: &Quat) {
        self.dir.rotate(rotation);
    }
}

impl Default for Ray {
    fn default() -> Self {
        Self::new(Default::default(), Vec3::new(0.0, 0.0, -1.0))
    }
}

pub struct Hit<'m> {
    /// Primitive hit in this intersection
    pub primitive: &'m dyn Intersect<'m>,

    pub depth: f32,
    pub point: Vec3,

    /// Barycentric coordinates expressing the hit point in terms of the primitive.
    /// Useful to interpolate vertex data of such a primitive
    pub uv: Vec2,
}

impl<'m> Hit<'m> {
    pub fn new(primitive: &'m dyn Intersect<'m>, depth: f32, point: Vec3, uv: Vec2) -> Self {
        Self {
            primitive,
            depth,
            point,
            uv,
        }
    }
}

pub trait Intersect<'m> {
    fn intersects(&'m self, ray: &Ray) -> Option<Hit<'m>>;
    fn get_color(&self, hit: &Hit) -> Color;
    fn get_normal(&self, hit: &Hit) -> Vec3;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn rotate() {
        let mut ray = Ray::default();
        let rot = Quat::new(-0.383, 0.0, 0.0, 0.924);
        ray.rotate(&rot);
        println!("{} {} {}", ray.dir.x, ray.dir.y, ray.dir.z);
        assert!(ray.dir.close(&Vec3::new(0.0, -0.707, -0.707)));
    }
}
