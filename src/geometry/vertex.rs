// Copyright © 2022-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::{Color, Vec2, Vec3};

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vertex {
    pub color: Color,
    pub uv: Vec2,

    pub normal: Vec3,
    pub tangent: Vec3,
    pub bitangent: Vec3,
}

impl Vertex {
    pub fn new(color: Color, uv: Vec2, normal: Vec3, tangent: Vec3, bitangent: Vec3) -> Self {
        Self {
            color,
            uv,
            normal,
            tangent,
            bitangent,
        }
    }
}

impl Default for Vertex {
    fn default() -> Self {
        Self::new(
            Color::WHITE,
            Vec2::default(),
            Vec3::Z_AXIS,
            Vec3::default(),
            Vec3::default(),
        )
    }
}
