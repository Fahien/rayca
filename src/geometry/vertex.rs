// Copyright © 2022-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::Color;

#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct Vertex {
    pub color: Color,
}

impl Vertex {
    pub fn new(color: Color) -> Self {
        Self { color }
    }
}

impl Default for Vertex {
    fn default() -> Self {
        Self::new(Color::WHITE)
    }
}
