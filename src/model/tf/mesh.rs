// Copyright © 2022-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::{Handle, Triangle, TriangleEx};

use super::*;

#[derive(Default)]
pub struct GltfPrimitiveBuilder {
    vertices: Vec<GltfVertex>,
    indices: Vec<u8>,
}

impl GltfPrimitiveBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn vertices(mut self, vertices: Vec<GltfVertex>) -> Self {
        self.vertices = vertices;
        self
    }

    pub fn indices(mut self, indices: Vec<u8>) -> Self {
        self.indices = indices;
        self
    }

    pub fn build(self) -> GltfPrimitive {
        let mut prim = GltfPrimitive::new(self.vertices);
        prim.indices = self.indices;
        prim
    }
}

#[derive(Default)]
pub struct GltfPrimitive {
    pub vertices: Vec<GltfVertex>,
    pub indices: Vec<u8>,
}

impl GltfPrimitive {
    pub fn builder() -> GltfPrimitiveBuilder {
        GltfPrimitiveBuilder::new()
    }

    pub fn new(vertices: Vec<GltfVertex>) -> Self {
        Self {
            vertices,
            indices: vec![],
        }
    }

    pub fn triangles(&self) -> (Vec<Triangle>, Vec<TriangleEx>) {
        let mut ret = vec![];
        let mut ret_ex = vec![];

        for i in 0..(self.indices.len() / 3) {
            let ai = self.indices[i * 3] as usize;
            let bi = self.indices[i * 3 + 1] as usize;
            let ci = self.indices[i * 3 + 2] as usize;

            let a = self.vertices[ai].pos;
            let b = self.vertices[bi].pos;
            let c = self.vertices[ci].pos;

            ret.push(Triangle::new(a, b, c));
            ret_ex.push(TriangleEx::default());
        }

        (ret, ret_ex)
    }
}

#[derive(Default)]
pub struct GltfMesh {
    pub primitives: Vec<Handle<GltfPrimitive>>,
}

impl GltfMesh {
    pub fn new(primitives: Vec<Handle<GltfPrimitive>>) -> Self {
        Self { primitives }
    }
}
