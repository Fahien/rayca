// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use super::*;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vertex {
    pub pos: Vec3,
    pub normal: Vec3,
    pub color: Color,
}

impl Vertex {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            pos: Vec3::new(x, y, z),
            normal: Vec3::new(0.0, 0.0, 1.0),
            color: Color::from(0xFFFFFFFF),
        }
    }
}

impl Default for Vertex {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
}
