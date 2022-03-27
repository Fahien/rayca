// Copyright © 2022-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::{Color, Vec2, Vec3};

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vertex {
    pub normal: Vec3,
    pub color: Color,
    pub uv: Vec2,
}

impl Vertex {
    pub fn new(normal: Vec3, color: Color, uv: Vec2) -> Self {
        Self { normal, color, uv }
    }
}

impl Default for Vertex {
    fn default() -> Self {
        Self::new(Vec3::Z_AXIS, Color::WHITE, Vec2::default())
    }
}
