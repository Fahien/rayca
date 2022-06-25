// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use num_traits::NumCast;

use crate::*;

#[derive(Debug, Clone)]
pub struct Triangle {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u8>,

    /// Index size in bytes. This is not index count
    pub index_size: usize,
}

impl Default for Triangle {
    fn default() -> Self {
        Self::new(vec![], vec![])
    }
}

impl Triangle {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u8>) -> Self {
        Self {
            vertices,
            indices,
            index_size: 1,
        }
    }

    fn triangles_impl<'m, Index: NumCast>(
        &self,
        transform: &Trs,
        material: Handle<Material>,
        model: &'m Model,
        indices: &[Index],
    ) -> Vec<BvhTriangle<'m>> {
        let mut ret = vec![];
        let matrix: Mat4 = transform.into();
        let tangent_matrix = Mat3::from(&matrix);

        let mut normal_trs = Inversed::from(transform.clone());
        normal_trs.source.translation = Vec3::default();

        let normal_matrix = Mat4::from(normal_trs).get_transpose();

        for i in 0..(indices.len() / 3) {
            let mut a = self.vertices[indices[i * 3].to_usize().unwrap()].clone();
            a.pos = &matrix * a.pos;
            a.normal = &normal_matrix * a.normal;
            a.tangent = &tangent_matrix * a.tangent;
            a.bitangent = &tangent_matrix * a.bitangent;

            let mut b = self.vertices[indices[i * 3 + 1].to_usize().unwrap()].clone();
            b.pos = &matrix * b.pos;
            b.normal = &normal_matrix * b.normal;
            b.tangent = &tangent_matrix * b.tangent;
            b.bitangent = &tangent_matrix * b.bitangent;

            let mut c = self.vertices[indices[i * 3 + 2].to_usize().unwrap()].clone();
            c.pos = &matrix * c.pos;
            c.normal = &normal_matrix * c.normal;
            c.tangent = &tangent_matrix * c.tangent;
            c.bitangent = &tangent_matrix * c.bitangent;

            ret.push(BvhTriangle::new(a, b, c, material, model));
        }

        ret
    }

    pub fn triangles<'m>(
        &self,
        transform: &Trs,
        material: Handle<Material>,
        model: &'m Model,
    ) -> Vec<BvhTriangle<'m>> {
        let indices_len = self.indices.len() / self.index_size;

        match self.index_size {
            1 => self.triangles_impl(transform, material, model, &self.indices),
            2 => {
                let indices = unsafe {
                    std::slice::from_raw_parts(self.indices.as_ptr() as *const u16, indices_len)
                };

                self.triangles_impl(transform, material, model, indices)
            }
            4 => {
                let indices = unsafe {
                    std::slice::from_raw_parts(self.indices.as_ptr() as *const u32, indices_len)
                };

                self.triangles_impl(transform, material, model, indices)
            }
            _ => panic!("Index size not supported"),
        }
    }
}
