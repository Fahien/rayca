// Copyright © 2022-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::{Color, Vec3};

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vertex {
    pub normal: Vec3,
    pub color: Color,
}

impl Vertex {
    pub fn new(normal: Vec3, color: Color) -> Self {
        Self { normal, color }
    }
}

impl Default for Vertex {
    fn default() -> Self {
        Self::new(Vec3::Z_AXIS, Color::WHITE)
    }
}
