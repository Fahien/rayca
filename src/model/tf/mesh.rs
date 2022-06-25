// Copyright © 2022-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::{
    Bvh, GgxMaterial, GltfSpheres, GltfTriangles, GltfVertex, Handle, Point3, SolvedTrs, Sphere,
    SphereEx, Triangle, TriangleEx,
};

pub struct GltfPrimitiveBuilder {
    triangles: Option<GltfTriangles>,
    spheres: Option<GltfSpheres>,
    material: Handle<GgxMaterial>,
}

impl Default for GltfPrimitiveBuilder {
    fn default() -> Self {
        Self {
            triangles: Some(GltfTriangles::default()),
            spheres: None,
            material: Handle::none(),
        }
    }
}

impl GltfPrimitiveBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn vertices(mut self, vertices: Vec<GltfVertex>) -> Self {
        if self.triangles.is_none() {
            self.triangles.replace(GltfTriangles::default());
        }
        self.triangles.as_mut().unwrap().vertices = vertices;
        self
    }

    pub fn indices(mut self, indices: Vec<u8>) -> Self {
        if self.triangles.is_none() {
            self.triangles.replace(GltfTriangles::default());
        }
        self.triangles.as_mut().unwrap().indices = indices;
        self
    }

    pub fn index_size(mut self, index_size_in_bytes: usize) -> Self {
        if self.triangles.is_none() {
            self.triangles.replace(GltfTriangles::default());
        }
        self.triangles.as_mut().unwrap().index_size_in_bytes = index_size_in_bytes;
        self
    }

    pub fn material(mut self, material: Handle<GgxMaterial>) -> Self {
        self.material = material;
        self
    }

    pub fn build(self) -> GltfPrimitive {
        GltfPrimitive {
            triangles: self.triangles,
            spheres: self.spheres,
            material: self.material,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GltfPrimitive {
    pub triangles: Option<GltfTriangles>,
    pub spheres: Option<GltfSpheres>,
    pub material: Handle<GgxMaterial>,
}

impl GltfPrimitive {
    pub fn builder() -> GltfPrimitiveBuilder {
        GltfPrimitiveBuilder::new()
    }

    pub fn triangles(triangles: GltfTriangles) -> Self {
        Self {
            triangles: Some(triangles),
            spheres: None,
            material: Handle::none(),
        }
    }

    pub fn sphere(spheres: GltfSpheres) -> Self {
        Self {
            triangles: None,
            spheres: Some(spheres),
            material: Handle::none(),
        }
    }

    pub fn unit_triangle() -> Self {
        Self::builder()
            .vertices(vec![
                GltfVertex::from_position(Point3::new(-1.0, 0.0, 0.0)),
                GltfVertex::from_position(Point3::new(1.0, 0.0, 0.0)),
                GltfVertex::from_position(Point3::new(0.0, 1.0, 0.0)),
            ])
            .indices(vec![0, 1, 2])
            .build()
    }

    pub fn primitives(
        &self,
        bvh: &Bvh,
        trs: Handle<SolvedTrs>,
    ) -> (Vec<Triangle>, Vec<TriangleEx>, Vec<Sphere>, Vec<SphereEx>) {
        let (triangles, triangles_ex) = if let Some(triangles) = &self.triangles {
            let trs = bvh.get_trs(trs);
            triangles.primitives(trs, self.material)
        } else {
            (Default::default(), Default::default())
        };

        let (sphere, sphere_ex) = if let Some(sphere) = &self.spheres {
            sphere.primitives(trs, self.material)
        } else {
            (Default::default(), Default::default())
        };

        (triangles, triangles_ex, sphere, sphere_ex)
    }
}

#[derive(Debug, Default)]
pub struct GltfMesh {
    pub primitives: Vec<Handle<GltfPrimitive>>,
}

impl GltfMesh {
    pub fn new(primitives: Vec<Handle<GltfPrimitive>>) -> Self {
        Self { primitives }
    }
}
