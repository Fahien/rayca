// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

pub mod sphere;
pub mod triangles;

pub use sphere::*;
pub use triangles::*;

#[derive(Debug, Clone)]
pub enum Geometry {
    Triangles(Triangles),
    Sphere(Sphere),
}

impl Default for Geometry {
    fn default() -> Self {
        Self::Triangles(Triangles::default())
    }
}
