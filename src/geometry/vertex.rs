// Copyright © 2022-2024
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Vertex {
    pub pos: Point3,
    pub ext: VertexExt,
}

impl Vertex {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            pos: Point3::new(x, y, z),
            ext: Default::default(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct VertexExt {
    pub uv: Vec2,
    pub color: Color,

    pub normal: Vec3,
    pub tangent: Vec3,
    pub bitangent: Vec3,
}

impl Default for VertexExt {
    fn default() -> Self {
        Self {
            uv: Vec2::default(),
            color: Color::from(0xFFFFFFFF),
            normal: Vec3::new(0.0, 0.0, 1.0),
            tangent: Vec3::new(0.0, 0.0, 0.0),
            bitangent: Vec3::new(0.0, 0.0, 0.0),
        }
    }
}
