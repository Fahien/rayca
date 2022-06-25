// Copyright © 2022-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::{GgxMaterial, GltfTriangles, GltfVertex, Handle, Triangle, TriangleEx, Trs};

#[derive(Default)]
pub struct GltfPrimitiveBuilder {
    triangles: GltfTriangles,
    material: Handle<GgxMaterial>,
}

impl GltfPrimitiveBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn vertices(mut self, vertices: Vec<GltfVertex>) -> Self {
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

    pub fn material(mut self, material: Handle<GgxMaterial>) -> Self {
        self.material = material;
        self
    }

    pub fn build(self) -> GltfPrimitive {
        let mut prim = GltfPrimitive::new(self.triangles);
        prim.material = self.material;
        prim
    }
}

#[derive(Default, Clone)]
pub struct GltfPrimitive {
    pub triangles: GltfTriangles,
    pub material: Handle<GgxMaterial>,
}

impl GltfPrimitive {
    pub fn builder() -> GltfPrimitiveBuilder {
        GltfPrimitiveBuilder::new()
    }

    pub fn unit_triangle() -> Self {
        Self::new(GltfTriangles::unit_triangle())
    }

    pub fn new(triangles: GltfTriangles) -> Self {
        Self {
            triangles,
            material: Handle::none(),
        }
    }

    pub fn triangles(&self, transform: &Trs) -> (Vec<Triangle>, Vec<TriangleEx>) {
        self.triangles.primitives(transform, self.material)
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
