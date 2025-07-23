// Copyright © 2022-2024
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

#[derive(Debug, Clone)]
pub enum Geometry {
    TriangleMesh(TriangleMesh),
    Sphere(Sphere),
}

impl Default for Geometry {
    fn default() -> Self {
        Self::TriangleMesh(TriangleMesh::default())
    }
}

impl Geometry {
    pub fn as_triangle_mesh(&self) -> Option<&TriangleMesh> {
        if let Geometry::TriangleMesh(mesh) = self {
            Some(mesh)
        } else {
            None
        }
    }
}
