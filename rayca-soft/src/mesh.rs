// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use super::*;

#[derive(Default)]
pub struct PrimitiveBuilder {
    geometry: Geometry,
    material: Option<Handle<Material>>,
}

impl PrimitiveBuilder {
    pub fn new() -> Self {
        Self {
            geometry: Geometry::default(),
            material: None,
        }
    }

    pub fn vertices(mut self, vertices: Vec<Vertex>) -> Self {
        match &mut self.geometry {
            Geometry::TriangleMesh(triangles) => {
                triangles.vertices = vertices;
            }
            _ => self.geometry = Geometry::TriangleMesh(TriangleMesh::new(vertices, vec![])),
        }
        self
    }

    pub fn indices(mut self, indices: Vec<u8>) -> Self {
        match &mut self.geometry {
            Geometry::TriangleMesh(triangles) => {
                triangles.indices = indices;
            }
            _ => self.geometry = Geometry::TriangleMesh(TriangleMesh::new(vec![], indices)),
        }
        self
    }

    pub fn index_size(mut self, index_size_in_bytes: usize) -> Self {
        match &mut self.geometry {
            Geometry::TriangleMesh(triangles) => {
                triangles.index_size_in_bytes = index_size_in_bytes;
            }
            _ => {
                let mut triangle = TriangleMesh::new(vec![], vec![]);
                triangle.index_size_in_bytes = index_size_in_bytes;
                self.geometry = Geometry::TriangleMesh(triangle);
            }
        }
        self
    }
    pub fn sphere(mut self, center: Point3, radius: f32) -> Self {
        match &mut self.geometry {
            Geometry::Sphere(sphere) => {
                sphere.set_center(center);
                sphere.set_radius(radius);
            }
            _ => {
                let sphere = Sphere::new(center, radius);
                self.geometry = Geometry::Sphere(sphere);
            }
        }
        self
    }

    pub fn material(mut self, material: Handle<Material>) -> Self {
        self.material = Some(material);
        self
    }

    pub fn build(self) -> Primitive {
        let mut prim = Primitive::new(self.geometry);
        if let Some(material) = self.material {
            prim.material = material;
        }
        prim
    }
}

#[derive(Default, Clone)]
pub struct Primitive {
    pub geometry: Geometry,
    pub material: Handle<Material>,
}

impl Primitive {
    pub fn builder() -> PrimitiveBuilder {
        PrimitiveBuilder::new()
    }

    pub fn new(geometry: Geometry) -> Self {
        Self {
            geometry,
            material: Handle::NONE,
        }
    }

    pub fn unit_triangle() -> Self {
        Self::builder()
            .vertices(vec![
                Vertex::builder()
                    .position(Point3::new(-1.0, 0.0, 0.0))
                    .build(),
                Vertex::builder()
                    .position(Point3::new(1.0, 0.0, 0.0))
                    .build(),
                Vertex::builder()
                    .position(Point3::new(0.0, 1.0, 0.0))
                    .build(),
            ])
            .indices(vec![0, 1, 2])
            .build()
    }

    pub fn unit_sphere() -> Self {
        Self::builder().sphere(Point3::default(), 1.0).build()
    }

    pub fn primitives(
        &self,
        node: Handle<Node>,
        material: Handle<Material>,
        model: &Model,
    ) -> Vec<BvhPrimitive> {
        match &self.geometry {
            Geometry::TriangleMesh(triangles) => {
                BvhPrimitive::from_triangle_mesh(triangles, node, material, model)
            }
            Geometry::Sphere(sphere) => BvhPrimitive::from_sphere(node, material, sphere),
        }
    }
}

#[derive(Default)]
pub struct MeshBuilder {
    pub primitives: Vec<Handle<Primitive>>,
}

impl MeshBuilder {
    pub fn primitives(mut self, primitives: Vec<Handle<Primitive>>) -> Self {
        self.primitives = primitives;
        self
    }

    pub fn build(self) -> Mesh {
        Mesh {
            primitives: self.primitives,
        }
    }
}

#[derive(Default)]
pub struct Mesh {
    pub primitives: Vec<Handle<Primitive>>,
}

impl Mesh {
    pub fn builder() -> MeshBuilder {
        MeshBuilder::default()
    }

    pub fn new(primitives: Vec<Handle<Primitive>>) -> Self {
        Self { primitives }
    }
}
