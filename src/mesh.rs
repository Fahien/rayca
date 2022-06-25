// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use super::*;

#[derive(Default)]
pub struct PrimitiveBuilder {
    triangles: Triangles,
    material: Option<Handle<Material>>,
}

impl PrimitiveBuilder {
    pub fn new() -> Self {
        Self {
            triangles: Triangles::default(),
            material: None,
        }
    }

    pub fn vertices(mut self, vertices: Vec<Vertex>) -> Self {
        self.triangles.vertices = vertices;
        self
    }

    pub fn indices(mut self, indices: Vec<u8>) -> Self {
        self.triangles.indices = indices;
        self
    }

    pub fn index_size(mut self, index_size_in_bytes: usize) -> Self {
        self.triangles.index_size_in_bytes = index_size_in_bytes;
        self
    }

    pub fn material(mut self, material: Option<Handle<Material>>) -> Self {
        self.material = material;
        self
    }

    pub fn build(self) -> Primitive {
        let mut prim = Primitive::new(self.triangles);
        prim.material = self.material;
        prim
    }
}

#[derive(Default, Clone)]
pub struct Primitive {
    pub triangles: Triangles,
    pub material: Option<Handle<Material>>,
}

impl Primitive {
    pub fn builder() -> PrimitiveBuilder {
        PrimitiveBuilder::new()
    }

    pub fn new(triangles: Triangles) -> Self {
        Self {
            triangles,
            material: None,
        }
    }

    pub fn unit_triangle() -> Self {
        Self::builder()
            .vertices(vec![
                Vertex::new(-1.0, 0.0, 0.0),
                Vertex::new(1.0, 0.0, 0.0),
                Vertex::new(0.0, 1.0, 0.0),
            ])
            .indices(vec![0, 1, 2])
            .build()
    }

    pub fn triangles<'m>(&self, transform: &Trs, model: &'m Model) -> Vec<BvhTriangle<'m>> {
        self.triangles.triangles(transform, self.material, model)
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
