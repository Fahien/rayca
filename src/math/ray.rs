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
}

pub struct Hit {
    pub point: Vec3,
    pub uv: Vec2,
}

impl Hit {
    pub fn new(point: Vec3, uv: Vec2) -> Self {
        Self { point, uv }
    }
}

pub trait Intersect {
    fn intersects(&self, ray: &Ray) -> Option<Hit>;
    fn get_color(&self, hit: &Hit) -> Color;
    fn get_normal(&self, hit: &Hit) -> Vec3;
}
