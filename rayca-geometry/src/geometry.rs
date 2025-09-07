// Copyright © 2022-2024
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

#[derive(Debug, Clone)]
pub enum Geometry {
    TriangleMesh(Handle<TriangleMesh>),
    Sphere(Handle<Sphere>),
}

impl Default for Geometry {
    fn default() -> Self {
        Self::TriangleMesh(Handle::NONE)
    }
}

impl Geometry {
    pub fn get_triangle_mesh(&self) -> Handle<TriangleMesh> {
        if let Geometry::TriangleMesh(handle) = self {
            *handle
        } else {
            Handle::NONE
        }
    }

    pub fn get_sphere(&self) -> Handle<Sphere> {
        if let Geometry::Sphere(handle) = self {
            *handle
        } else {
            Handle::NONE
        }
    }
}
