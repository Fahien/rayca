// Copyright © 2022-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::{Color, Point3, Vec2, Vec3};

#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct GltfVertex {
    pub pos: Point3,
    pub color: Color,
    pub uv: Vec2,

    pub normal: Vec3,
    pub tangent: Vec3,
    pub bitangent: Vec3,
}

impl GltfVertex {
    pub fn new(
        pos: Point3,
        color: Color,
        uv: Vec2,
        normal: Vec3,
        tangent: Vec3,
        bitangent: Vec3,
    ) -> Self {
        Self {
            pos,
            color,
            uv,
            normal,
            tangent,
            bitangent,
        }
    }

    pub fn from_position(pos: Point3) -> Self {
        Self {
            pos,
            ..Default::default()
        }
    }
}

impl Default for GltfVertex {
    fn default() -> Self {
        Self::new(
            Point3::default(),
            Color::WHITE,
            Vec2::default(),
            Vec3::Z_AXIS,
            Vec3::default(),
            Vec3::default(),
        )
    }
}
