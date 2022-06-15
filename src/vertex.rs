// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use super::*;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vertex {
    pub pos: Point3,

    pub uv: Vec2,
    pub color: Color,

    pub normal: Vec3,
    pub tangent: Vec3,
    pub bitangent: Vec3,
}

impl Vertex {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            pos: Point3::new(x, y, z),
            uv: Vec2::default(),
            color: Color::from(0xFFFFFFFF),
            normal: Vec3::new(0.0, 0.0, 1.0),
            tangent: Vec3::new(0.0, 0.0, 0.0),
            bitangent: Vec3::new(0.0, 0.0, 0.0),
        }
    }
}

impl Default for Vertex {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
}
