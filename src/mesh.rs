// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use super::*;

pub struct PrimitiveBuilder {
    vertices: Vec<Vertex>,
    indices: Vec<u8>,
}

impl PrimitiveBuilder {
    pub fn new() -> Self {
        Self {
            vertices: vec![],
            indices: vec![],
        }
    }

    pub fn vertices(mut self, vertices: Vec<Vertex>) -> Self {
        self.vertices = vertices;
        self
    }

    pub fn indices(mut self, indices: Vec<u8>) -> Self {
        self.indices = indices;
        self
    }

    pub fn build(self) -> Primitive {
        let mut prim = Primitive::new(self.vertices);
        prim.indices = self.indices;
        prim
    }
}

#[derive(Default)]
pub struct Primitive {
    vertices: Vec<Vertex>,
    indices: Vec<u8>,
}

impl Primitive {
    pub fn builder() -> PrimitiveBuilder {
        PrimitiveBuilder::new()
    }

    pub fn new(vertices: Vec<Vertex>) -> Self {
        Self {
            vertices,
            indices: vec![],
        }
    }

    pub fn triangles(&self, transform: Mat4) -> Vec<Triangle<Vertex>> {
        let mut ret = vec![];

        for i in 0..(self.indices.len() / 3) {
            let mut a = self.vertices[self.indices[i * 3] as usize];
            a.pos = &transform * a.pos;

            let mut b = self.vertices[self.indices[i * 3 + 1] as usize];
            b.pos = &transform * b.pos;

            let mut c = self.vertices[self.indices[i * 3 + 2] as usize];
            c.pos = &transform * c.pos;

            ret.push(Triangle::new(a, b, c))
        }
        ret
    }
}

#[derive(Default)]
pub struct Mesh {
    pub primitives: Vec<Handle<Primitive>>,
}

impl Mesh {
    pub fn new(primitives: Vec<Handle<Primitive>>) -> Self {
        Self { primitives }
    }
}