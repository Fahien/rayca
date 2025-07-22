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
