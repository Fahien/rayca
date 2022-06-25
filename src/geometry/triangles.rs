// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use num_traits::NumCast;

use crate::*;

/// Triangle mesh
#[derive(Debug, Clone)]
pub struct Triangles {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u8>,

    /// Index size in bytes. This is not index count
    pub index_size_in_bytes: usize,
}

impl Default for Triangles {
    fn default() -> Self {
        Self::new(vec![], vec![])
    }
}

impl Triangles {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u8>) -> Self {
        Self {
            vertices,
            indices,
            index_size_in_bytes: 1,
        }
    }

    fn primitives_impl<'m, Index: NumCast>(
        &self,
        node: Handle<Node>,
        material: Handle<Material>,
        model: &'m Model,
        indices: &[Index],
    ) -> Vec<BvhPrimitive> {
        let mut ret = vec![];

        let trs = model.nodes.get(node).unwrap().trs.clone();
        let tangent_matrix = Mat3::from(&trs);

        let inverse_trs = Inversed::from(&trs);
        let normal_matrix = Mat3::from(&inverse_trs).get_transpose();

        for i in 0..(indices.len() / 3) {
            let mut a = self.vertices[indices[i * 3].to_usize().unwrap()];
            a.pos = &trs * a.pos;
            a.normal = &normal_matrix * a.normal;
            a.tangent = &tangent_matrix * a.tangent;
            a.bitangent = &tangent_matrix * a.bitangent;

            let mut b = self.vertices[indices[i * 3 + 1].to_usize().unwrap()];
            b.pos = &trs * b.pos;
            b.normal = &normal_matrix * b.normal;
            b.tangent = &tangent_matrix * b.tangent;
            b.bitangent = &tangent_matrix * b.bitangent;

            let mut c = self.vertices[indices[i * 3 + 2].to_usize().unwrap()];
            c.pos = &trs * c.pos;
            c.normal = &normal_matrix * c.normal;
            c.tangent = &tangent_matrix * c.tangent;
            c.bitangent = &tangent_matrix * c.bitangent;

            let triangle = Box::new(BvhTriangle::new(a, b, c));
            let geometry = BvhGeometry::Triangle(triangle);
            let primitive = BvhPrimitive::new(geometry, node, material);
            ret.push(primitive)
        }

        ret
    }

    pub fn primitives(
        &self,
        node: Handle<Node>,
        material: Handle<Material>,
        model: &Model,
    ) -> Vec<BvhPrimitive> {
        let indices_len = self.indices.len() / self.index_size_in_bytes;

        match self.index_size_in_bytes {
            1 => self.primitives_impl(node, material, model, &self.indices),
            2 => {
                let indices = unsafe {
                    std::slice::from_raw_parts(self.indices.as_ptr() as *const u16, indices_len)
                };

                self.primitives_impl(node, material, model, indices)
            }
            4 => {
                let indices = unsafe {
                    std::slice::from_raw_parts(self.indices.as_ptr() as *const u32, indices_len)
                };

                self.primitives_impl(node, material, model, indices)
            }
            _ => panic!("Index size not supported"),
        }
    }
}
