// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use num_traits::NumCast;

use super::*;

pub struct PrimitiveBuilder {
    vertices: Vec<Vertex>,
    indices: Vec<u8>,
    index_size: usize,
    material: Handle<Material>,
}

impl PrimitiveBuilder {
    pub fn new() -> Self {
        Self {
            vertices: vec![],
            indices: vec![],
            index_size: 1,
            material: Handle::none(),
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

    pub fn material(mut self, material: Handle<Material>) -> Self {
        self.material = material;
        self
    }

    pub fn build(self) -> Primitive {
        let mut prim = Primitive::new(self.vertices);
        prim.indices = self.indices;
        prim.index_size = self.index_size;
        prim.material = self.material;
        prim
    }
}

#[derive(Default)]
pub struct Primitive {
    pub vertices: Vec<Vertex>,
    indices: Vec<u8>,
    index_size: usize,
    pub material: Handle<Material>,
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
            material: Handle::none(),
        }
    }

    fn triangles_impl<'m, Index: NumCast>(
        &self,
        transform: &Trs,
        material: Handle<Material>,
        model: &'m Model,
        indices: &[Index],
    ) -> Vec<Triangle<'m, Vertex>> {
        let mut ret = vec![];
        let matrix = transform.into();

        let mut normal_trs = transform.get_inverse();
        normal_trs.translation = Vec3::default();

        let normal_matrix = Mat4::from(normal_trs).get_transpose();

        for i in 0..(indices.len() / 3) {
            let mut a = self.vertices[indices[i * 3].to_usize().unwrap()];
            a.pos = &matrix * a.pos;
            a.normal = &normal_matrix * a.normal;

            let mut b = self.vertices[indices[i * 3 + 1].to_usize().unwrap()];
            b.pos = &matrix * b.pos;
            b.normal = &normal_matrix * b.normal;

            let mut c = self.vertices[indices[i * 3 + 2].to_usize().unwrap()];
            c.pos = &matrix * c.pos;
            c.normal = &normal_matrix * c.normal;

            ret.push(Triangle::new(a, b, c, material, model));
        }

        ret
    }

    pub fn triangles<'m>(
        &self,
        transform: &Trs,
        material: Handle<Material>,
        model: &'m Model,
    ) -> Vec<Triangle<'m, Vertex>> {
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

#[derive(Default)]
pub struct Mesh {
    pub primitives: Vec<Handle<Primitive>>,
}

impl Mesh {
    pub fn new(primitives: Vec<Handle<Primitive>>) -> Self {
        Self { primitives }
    }
}
