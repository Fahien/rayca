// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

pub mod sphere;
pub mod triangle;

pub use sphere::*;
pub use triangle::*;

#[derive(Debug, Clone)]
pub enum Geometry {
    Triangles(Triangle),
    Sphere(Sphere),
}

impl Default for Geometry {
    fn default() -> Self {
        Self::Triangles(Triangle::default())
    }
}
