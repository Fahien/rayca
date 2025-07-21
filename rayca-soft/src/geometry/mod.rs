// Copyright © 2022-2024
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

pub mod sphere;
pub mod triangles;
pub mod vertex;

pub use sphere::*;
pub use triangles::*;
pub use vertex::*;

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
