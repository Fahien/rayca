// Copyright © 2022-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::{Handle, Mat4, Triangle, TriangleEx, Vertex};

use super::*;

pub struct GltfPrimitiveBuilder {
    vertices: Vec<GltfVertex>,
    indices: Vec<u8>,
    index_size: usize,
}

impl Default for GltfPrimitiveBuilder {
    fn default() -> Self {
        Self {
            vertices: Default::default(),
            indices: Default::default(),
            index_size: 1,
        }
    }
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

    pub fn index_size(mut self, index_size: usize) -> Self {
        self.index_size = index_size;
        self
    }

    pub fn build(self) -> GltfPrimitive {
        let mut prim = GltfPrimitive::new(self.vertices);
        prim.indices = self.indices;
        prim.index_size = self.index_size;
        prim
    }
}

#[derive(Default)]
pub struct GltfPrimitive {
    pub vertices: Vec<GltfVertex>,
    pub indices: Vec<u8>,
    index_size: usize,
}

impl GltfPrimitive {
    pub fn builder() -> GltfPrimitiveBuilder {
        GltfPrimitiveBuilder::new()
    }

    pub fn new(vertices: Vec<GltfVertex>) -> Self {
        Self {
            vertices,
            indices: vec![],
            index_size: 1,
        }
    }

    pub fn triangles(&self, transform: Mat4) -> (Vec<Triangle>, Vec<TriangleEx>) {
        let mut ret = vec![];
        let mut ret_ex = vec![];

        let indices_len = self.indices.len() / self.index_size;
        match self.index_size {
            1 => {
                for i in 0..(self.indices.len() / 3) {
                    let gltf_a = &self.vertices[self.indices[i * 3] as usize];
                    let a = &transform * gltf_a.pos;
                    let a_ex = Vertex::new(gltf_a.normal, gltf_a.color);

                    let gltf_b = &self.vertices[self.indices[i * 3 + 1] as usize];
                    let b = &transform * gltf_b.pos;
                    let b_ex = Vertex::new(gltf_b.normal, gltf_b.color);

                    let gltf_c = &self.vertices[self.indices[i * 3 + 2] as usize];
                    let c = &transform * gltf_c.pos;
                    let c_ex = Vertex::new(gltf_b.normal, gltf_b.color);

                    ret.push(Triangle::new(a, b, c));
                    ret_ex.push(TriangleEx::new(a_ex, b_ex, c_ex));
                }
            }
            2 => {
                let indices = unsafe {
                    std::slice::from_raw_parts(self.indices.as_ptr() as *const u16, indices_len)
                };

                for i in 0..(indices.len() / 3) {
                    let gltf_a = &self.vertices[indices[i * 3] as usize];
                    let a = &transform * gltf_a.pos;
                    let a_ex = Vertex::new(gltf_a.normal, gltf_a.color);

                    let gltf_b = &self.vertices[indices[i * 3 + 1] as usize];
                    let b = &transform * gltf_b.pos;
                    let b_ex = Vertex::new(gltf_b.normal, gltf_b.color);

                    let gltf_c = &self.vertices[indices[i * 3 + 2] as usize];
                    let c = &transform * gltf_c.pos;
                    let c_ex = Vertex::new(gltf_b.normal, gltf_b.color);

                    ret.push(Triangle::new(a, b, c));
                    ret_ex.push(TriangleEx::new(a_ex, b_ex, c_ex));

                    ret.push(Triangle::new(a, b, c));
                    ret_ex.push(TriangleEx::default());
                }
            }
            4 => {
                let indices = unsafe {
                    std::slice::from_raw_parts(self.indices.as_ptr() as *const u32, indices_len)
                };

                for i in 0..(indices.len() / 3) {
                    let gltf_a = &self.vertices[indices[i * 3] as usize];
                    let a = &transform * gltf_a.pos;
                    let a_ex = Vertex::new(gltf_a.normal, gltf_a.color);

                    let gltf_b = &self.vertices[indices[i * 3 + 1] as usize];
                    let b = &transform * gltf_b.pos;
                    let b_ex = Vertex::new(gltf_b.normal, gltf_b.color);

                    let gltf_c = &self.vertices[indices[i * 3 + 2] as usize];
                    let c = &transform * gltf_c.pos;
                    let c_ex = Vertex::new(gltf_b.normal, gltf_b.color);

                    ret.push(Triangle::new(a, b, c));
                    ret_ex.push(TriangleEx::new(a_ex, b_ex, c_ex));

                    ret.push(Triangle::new(a, b, c));
                    ret_ex.push(TriangleEx::default());
                }
            }
            _ => panic!("Index size not supported"),
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
