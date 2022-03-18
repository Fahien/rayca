// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use super::*;

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Vertex {
    pub point: Vec3,
    pub color: RGBA8,
}

impl Vertex {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            point: Vec3::new(x, y, z),
            ..Default::default()
        }
    }
}
