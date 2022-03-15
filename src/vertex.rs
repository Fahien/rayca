// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use super::*;

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Vertex {
    pub point: Point3,
    pub color: Color,
}

impl Vertex {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            point: Point3::new(x, y, z),
            ..Default::default()
        }
    }
}
