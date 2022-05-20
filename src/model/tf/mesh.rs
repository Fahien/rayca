// Copyright © 2022-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use num_traits::NumCast;

use crate::{
    GgxMaterial, GltfVertex, Handle, Inversed, Mat4, Point3, Triangle, TriangleEx, Trs, Vec3,
    Vertex,
};

pub struct GltfPrimitiveBuilder {
    vertices: Vec<GltfVertex>,
    indices: Vec<u8>,
    index_size: usize,
    material: Handle<GgxMaterial>,
}

impl Default for GltfPrimitiveBuilder {
    fn default() -> Self {
        Self {
            vertices: Default::default(),
            indices: Default::default(),
            index_size: 1,
            material: Handle::none(),
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

    pub fn material(mut self, material: Handle<GgxMaterial>) -> Self {
        self.material = material;
        self
    }

    pub fn build(self) -> GltfPrimitive {
        let mut prim = GltfPrimitive::new(self.vertices, self.indices);
        prim.index_size = self.index_size;
        prim.material = self.material;
        prim
    }
}

#[derive(Default)]
pub struct GltfPrimitive {
    pub vertices: Vec<GltfVertex>,
    pub indices: Vec<u8>,
    index_size: usize,
    pub material: Handle<GgxMaterial>,
}

impl GltfPrimitive {
    pub fn builder() -> GltfPrimitiveBuilder {
        GltfPrimitiveBuilder::new()
    }

    pub fn unit_triangle() -> Self {
        let mut a = GltfVertex::default();
        a.pos.x -= 1.0;
        let mut b = GltfVertex::default();
        b.pos.x += 1.0;
        let mut c = GltfVertex::default();
        c.pos.y += 1.0;
        Self::new(vec![a, b, c], vec![0, 1, 2])
    }

    pub fn new(vertices: Vec<GltfVertex>, indices: Vec<u8>) -> Self {
        Self {
            vertices,
            indices,
            index_size: 1,
            material: Handle::none(),
        }
    }

    fn process_vertex(&self, matrix: &Mat4, normal_matrix: &Mat4, i: usize) -> (Point3, Vertex) {
        let gltf_vertex = &self.vertices[i];
        let pos = matrix * gltf_vertex.pos;
        let normal = normal_matrix * gltf_vertex.normal;
        let ex = Vertex::new(normal, gltf_vertex.color, gltf_vertex.uv);
        (pos, ex)
    }

    pub fn triangles_impl<Index: NumCast>(
        &self,
        transform: &Trs,
        indices: &[Index],
    ) -> (Vec<Triangle>, Vec<TriangleEx>) {
        let mut ret = vec![];
        let mut ret_ex = vec![];
        let matrix = Mat4::from(transform);

        let mut normal_trs = Inversed::from(transform.clone());
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
            ret_ex.push(TriangleEx::new(a_ex, b_ex, c_ex, self.material));
        }

        (ret, ret_ex)
    }

    pub fn triangles(&self, transform: &Trs) -> (Vec<Triangle>, Vec<TriangleEx>) {
        let indices_len = self.indices.len() / self.index_size;

        match self.index_size {
            1 => self.triangles_impl(transform, &self.indices),
            2 => {
                let indices = unsafe {
                    std::slice::from_raw_parts(self.indices.as_ptr() as *const u16, indices_len)
                };

                self.triangles_impl(transform, indices)
            }
            4 => {
                let indices = unsafe {
                    std::slice::from_raw_parts(self.indices.as_ptr() as *const u32, indices_len)
                };

                self.triangles_impl(transform, indices)
            }
            _ => panic!("Index size not supported"),
        }
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
