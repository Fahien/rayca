// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use super::*;

#[derive(Default)]
pub struct PrimitiveBuilder {
    vertices: Vec<Vertex>,
    indices: Vec<u8>,
    index_size: usize,
}

impl PrimitiveBuilder {
    pub fn new() -> Self {
        Self {
            vertices: vec![],
            indices: vec![],
            index_size: 1,
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

    pub fn index_size(mut self, index_size: usize) -> Self {
        self.index_size = index_size;
        self
    }

    pub fn build(self) -> Primitive {
        let mut prim = Primitive::new(self.vertices);
        prim.indices = self.indices;
        prim.index_size = self.index_size;
        prim
    }
}

#[derive(Default)]
pub struct Primitive {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u8>,
    index_size: usize,
}

impl Primitive {
    pub fn builder() -> PrimitiveBuilder {
        PrimitiveBuilder::new()
    }

    pub fn new(vertices: Vec<Vertex>) -> Self {
        Self {
            vertices,
            indices: vec![],
            index_size: 1,
        }
    }

    pub fn triangles(&self, trs: &Trs) -> Vec<Triangle> {
        let mut ret = vec![];

        let indices_len = self.indices.len() / self.index_size;
        match self.index_size {
            1 => {
                for i in 0..(self.indices.len() / 3) {
                    let mut a = self.vertices[self.indices[i * 3] as usize];
                    a.pos = trs * a.pos;

                    let mut b = self.vertices[self.indices[i * 3 + 1] as usize];
                    b.pos = trs * b.pos;

                    let mut c = self.vertices[self.indices[i * 3 + 2] as usize];
                    c.pos = trs * c.pos;

                    ret.push(Triangle::new(a, b, c))
                }
            }
            2 => {
                let indices = unsafe {
                    std::slice::from_raw_parts(self.indices.as_ptr() as *const u16, indices_len)
                };

                for i in 0..(indices.len() / 3) {
                    let mut a = self.vertices[indices[i * 3] as usize];
                    a.pos = trs * a.pos;

                    let mut b = self.vertices[indices[i * 3 + 1] as usize];
                    b.pos = trs * b.pos;

                    let mut c = self.vertices[indices[i * 3 + 2] as usize];
                    c.pos = trs * c.pos;

                    ret.push(Triangle::new(a, b, c))
                }
            }
            4 => {
                let indices = unsafe {
                    std::slice::from_raw_parts(self.indices.as_ptr() as *const u16, indices_len)
                };

                for i in 0..(indices.len() / 3) {
                    let mut a = self.vertices[indices[i * 3] as usize];
                    a.pos = trs * a.pos;

                    let mut b = self.vertices[indices[i * 3 + 1] as usize];
                    b.pos = trs * b.pos;

                    let mut c = self.vertices[indices[i * 3 + 2] as usize];
                    c.pos = trs * c.pos;

                    ret.push(Triangle::new(a, b, c))
                }
            }
            _ => panic!("Index size not supported"),
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
