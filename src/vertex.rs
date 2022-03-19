// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use super::*;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vertex {
    pub pos: Vec3,
    pub color: RGBA8,
}

impl Vertex {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            pos: Vec3::new(x, y, z),
            color: RGBA8::from(0xFFFFFFFF),
        }
    }
}

impl Default for Vertex {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
}
