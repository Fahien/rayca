// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use super::*;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vertex {
    pub pos: Point3,
    pub normal: Vec3,
    pub uv: Vec2,
    pub color: Color,
}

impl Vertex {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            pos: Point3::new(x, y, z),
            normal: Vec3::new(0.0, 0.0, 1.0),
            uv: Vec2::default(),
            color: Color::from(0xFFFFFFFF),
        }
    }
}

impl Default for Vertex {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
}
