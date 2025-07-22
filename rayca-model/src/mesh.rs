// Copyright © 2021-2024
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

#[derive(Clone, Default)]
pub struct MeshBuilder {
    primitives: Vec<Handle<Primitive>>,
}

impl MeshBuilder {
    pub fn primitive(mut self, primitive: Handle<Primitive>) -> Self {
        self.primitives.clear();
        self.primitives.push(primitive);
        self
    }

    pub fn primitives(mut self, primitives: Vec<Handle<Primitive>>) -> Self {
        self.primitives = primitives;
        self
    }

    pub fn build(self) -> Mesh {
        Mesh {
            primitives: self.primitives,
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct Mesh {
    pub primitives: Vec<Handle<Primitive>>,
}

impl Mesh {
    pub fn builder() -> MeshBuilder {
        MeshBuilder::default()
    }
}
