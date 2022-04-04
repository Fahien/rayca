// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use super::*;

#[derive(Debug)]
pub struct Ray {
    pub origin: Vec3,
    pub dir: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, dir: Vec3) -> Self {
        Self { origin, dir }
    }

    pub fn translate(&mut self, translation: &Vec3) {
        self.origin += translation;
    }

    pub fn rotate(&mut self, rotation: &Quat) {
        self.dir.rotate(rotation);
    }
}

pub struct Hit {
    pub depth: f32,
    pub point: Vec3,
    pub uv: Vec2,
}

impl Hit {
    pub fn new(depth: f32, point: Vec3, uv: Vec2) -> Self {
        Self {
            depth,
            point,
            uv,
        }
    }
}

pub trait Intersect {
    fn intersects(&self, ray: &Ray) -> Option<Hit>;
    fn get_color(&self, hit: &Hit) -> RGBA8;
    fn get_normal(&self, hit: &Hit) -> Vec3;
}
