// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use super::*;

#[derive(Debug)]
pub struct Ray {
    pub origin: FVec3,
    pub dir: FVec3,

    // Reciprocal of direction
    pub rdir: FVec3,
}

impl Ray {
    pub fn new(origin: FVec3, dir: FVec3) -> Self {
        let rdir = 1.0 / dir;
        Self {
            origin: origin,
            dir: dir,
            rdir: rdir,
        }
    }

    pub fn translate(&mut self, translation: &Vec3) {
        self.origin += FVec3::from(translation);
    }

    pub fn rotate(&mut self, rotation: &Quat) {
        self.dir.rotate(rotation);
    }
}

impl Default for Ray {
    fn default() -> Self {
        Self::new(FVec3::default(), FVec3::new(0.0, 0.0, -1.0))
    }
}

pub struct Hit {
    pub depth: f32,
    pub point: FVec3,
    pub uv: Vec2,
}

impl Hit {
    pub fn new(depth: f32, point: FVec3, uv: Vec2) -> Self {
        Self { depth, point, uv }
    }
}

pub trait Intersect {
    fn intersects(&self, ray: &Ray) -> Option<Hit>;
    fn get_color(&self, hit: &Hit) -> Color;
    fn get_normal(&self, hit: &Hit) -> FVec3;
    fn get_metallic_roughness(&self, hit: &Hit) -> (f32, f32);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn rotate() {
        let mut ray = Ray::default();
        let rot = Quat::new(-0.383, 0.0, 0.0, 0.924);
        ray.rotate(&rot);
        println!("{:?}", ray.dir);
        assert!(ray.dir.close(&FVec3::new(0.0, -0.707, -0.707)));
    }
}
