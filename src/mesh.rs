// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use super::*;

pub struct PrimitiveBuilder {
    triangles: Option<Triangles>,
    sphere: Option<Sphere>,
    material: Option<Handle<Material>>,
}

impl Default for PrimitiveBuilder {
    fn default() -> Self {
        Self {
            triangles: Some(Triangles::default()),
            sphere: None,
            material: None,
        }
    }
}

impl PrimitiveBuilder {
    pub fn new() -> Self {
        Self {
            triangles: None,
            sphere: None,
            material: None,
        }
    }

    pub fn vertices(mut self, vertices: Vec<Vertex>) -> Self {
        if self.triangles.is_none() {
            self.triangles.replace(Triangles::default());
        }
        self.triangles.as_mut().unwrap().vertices = vertices;
        self
    }

    pub fn indices(mut self, indices: Vec<u8>) -> Self {
        if self.triangles.is_none() {
            self.triangles.replace(Triangles::default());
        }
        self.triangles.as_mut().unwrap().indices = indices;
        self
    }

    pub fn index_size(mut self, index_size_in_bytes: usize) -> Self {
        if self.triangles.is_none() {
            self.triangles.replace(Triangles::default());
        }
        self.triangles.as_mut().unwrap().index_size_in_bytes = index_size_in_bytes;
        self
    }

    pub fn material(mut self, material: Option<Handle<Material>>) -> Self {
        self.material = material;
        self
    }

    pub fn build(self) -> Primitive {
        Primitive {
            triangles: self.triangles,
            sphere: self.sphere,
            material: self.material,
        }
    }
}

#[derive(Default, Clone)]
pub struct Primitive {
    pub triangles: Option<Triangles>,
    pub sphere: Option<Sphere>,
    pub material: Option<Handle<Material>>,
}

impl Primitive {
    pub fn builder() -> PrimitiveBuilder {
        PrimitiveBuilder::new()
    }

    pub fn triangles(triangles: Triangles) -> Self {
        Self {
            triangles: Some(triangles),
            sphere: None,
            material: None,
        }
    }

    pub fn sphere(sphere: Sphere) -> Self {
        Self {
            triangles: None,
            sphere: Some(sphere),
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

    pub fn primitives<'m>(
        &self,
        trs: &'m Trs,
        model: &'m Model,
    ) -> (Vec<BvhTriangle<'m>>, Vec<BvhSphere<'m>>) {
        let triangles = if let Some(triangles) = &self.triangles {
            triangles.primitives(trs, self.material, model)
        } else {
            vec![]
        };

        let sphere = if let Some(sphere) = &self.sphere {
            sphere.primitives(trs, self.material, model)
        } else {
            vec![]
        };

        (triangles, sphere)
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
