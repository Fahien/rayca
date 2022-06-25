// Copyright © 2022-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use num_traits::NumCast;

use crate::*;

#[derive(Debug, Clone)]
pub struct GltfTriangles {
    pub vertices: Vec<GltfVertex>,
    pub indices: Vec<u8>,

    /// Index size in bytes. This is not index count
    pub index_size_in_bytes: usize,
}

impl Default for GltfTriangles {
    fn default() -> Self {
        Self {
            vertices: Default::default(),
            indices: Default::default(),
            index_size_in_bytes: 1,
        }
    }
}

impl GltfTriangles {
    pub fn new(vertices: Vec<GltfVertex>, indices: Vec<u8>) -> Self {
        Self {
            vertices,
            indices,
            index_size_in_bytes: 1,
        }
    }

    pub fn unit_triangle() -> Self {
        let mut a = GltfVertex::default();
        a.pos.set_x(-1.0);
        let mut b = GltfVertex::default();
        b.pos.set_x(1.0);
        let mut c = GltfVertex::default();
        c.pos.set_y(1.0);
        Self::new(vec![a, b, c], vec![0, 1, 2])
    }

    fn process_vertex(&self, matrix: &Mat4, normal_matrix: &Mat4, i: usize) -> (Point3, Vertex) {
        let gltf_vertex = &self.vertices[i];
        let pos = matrix * gltf_vertex.pos;
        let normal = normal_matrix * gltf_vertex.normal;
        // Vector are not affected by the translation part,
        // so it is fine to use the transform matrix as tangent matrix
        let tangent = matrix * gltf_vertex.tangent;
        let bitangent = matrix * gltf_vertex.bitangent;
        let ex = Vertex::new(
            gltf_vertex.color,
            gltf_vertex.uv,
            normal,
            tangent,
            bitangent,
        );
        (pos, ex)
    }

    pub fn primitives_impl<Index: NumCast>(
        &self,
        trs: &Trs,
        indices: &[Index],
        material: Handle<GgxMaterial>,
    ) -> (Vec<Triangle>, Vec<TriangleEx>) {
        let mut ret = vec![];
        let mut ret_ex = vec![];
        let matrix = Mat4::from(trs);

        let mut normal_trs = Inversed::from(trs.clone());
        normal_trs.source.translation = Vec3::default();

        let normal_matrix = Mat4::from(normal_trs).get_transpose();

        for i in 0..(indices.len() / 3) {
            let a_i = indices[i * 3].to_usize().unwrap();
            let (a, a_ex) = self.process_vertex(&matrix, &normal_matrix, a_i);

            let b_i = indices[i * 3 + 1].to_usize().unwrap();
            let (b, b_ex) = self.process_vertex(&matrix, &normal_matrix, b_i);

            let c_i = indices[i * 3 + 2].to_usize().unwrap();
            let (c, c_ex) = self.process_vertex(&matrix, &normal_matrix, c_i);

            ret.push(Triangle::new(a, b, c));
            ret_ex.push(TriangleEx::new(a_ex, b_ex, c_ex, material));
        }

        (ret, ret_ex)
    }

    pub fn primitives(
        &self,
        trs: &Trs,
        material: Handle<GgxMaterial>,
    ) -> (Vec<Triangle>, Vec<TriangleEx>) {
        let indices_len = self.indices.len() / self.index_size_in_bytes;

        match self.index_size_in_bytes {
            1 => self.primitives_impl(trs, &self.indices, material),
            2 => {
                let indices = unsafe {
                    std::slice::from_raw_parts(self.indices.as_ptr() as *const u16, indices_len)
                };

                self.primitives_impl(trs, indices, material)
            }
            4 => {
                let indices = unsafe {
                    std::slice::from_raw_parts(self.indices.as_ptr() as *const u32, indices_len)
                };

                self.primitives_impl(trs, indices, material)
            }
            _ => panic!("Index size not supported"),
        }
    }
}
