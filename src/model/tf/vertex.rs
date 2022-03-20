// Copyright © 2022-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::{Color, Point3, Vec3};

#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct GltfVertex {
    pub pos: Point3,
    pub normal: Vec3,
    pub color: Color,
}

impl GltfVertex {
    pub fn new(pos: Point3, normal: Vec3, color: Color) -> Self {
        Self { pos, normal, color }
    }
}

impl Default for GltfVertex {
    fn default() -> Self {
        Self::new(Point3::default(), Vec3::Z_AXIS, Color::WHITE)
    }
}
