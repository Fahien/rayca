// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use num_traits::NumCast;

use crate::*;

pub enum BvhGeometry {
    Triangle(BvhTriangle),
    Sphere(Sphere),
}

impl BvhGeometry {
    pub fn get_color(&self, hit: &Hit) -> Color {
        match self {
            BvhGeometry::Triangle(triangle) => triangle.interpolate_colors(&hit.uv),
            BvhGeometry::Sphere(_) => Color::white(),
        }
    }

    pub fn get_uv(&self, hit: &Hit) -> Vec2 {
        match self {
            BvhGeometry::Triangle(triangle) => triangle.interpolate_uvs(&hit.uv),
            // TODO spherical coordinates?
            BvhGeometry::Sphere(_) => Vec2::default(),
        }
    }

    pub fn get_normal(&self, hit: &Hit) -> Vec3 {
        match self {
            BvhGeometry::Triangle(triangle) => triangle.interpolate_normals(&hit.uv),
            BvhGeometry::Sphere(sphere) => sphere.get_normal(&hit.point),
        }
    }

    pub fn get_tangent(&self, hit: &Hit) -> Vec3 {
        match self {
            BvhGeometry::Triangle(triangle) => triangle.interpolate_tangents(&hit.uv),
            BvhGeometry::Sphere(_) => Vec3::default(),
        }
    }

    pub fn get_bitangent(&self, hit: &Hit) -> Vec3 {
        match &self {
            BvhGeometry::Triangle(triangle) => triangle.interpolate_bitangents(&hit.uv),
            BvhGeometry::Sphere(_) => Vec3::default(),
        }
    }
}

pub struct BvhPrimitive {
    pub geometry: BvhGeometry,
    pub node: Handle<Node>,

    /// We store handle and model here as we will anyway need to use the model
    /// when quering material properties such as textures.
    pub material: Handle<Material>,
}

const WHITE_MATERIAL: Material = Material {
    color: Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    },
    albedo_texture: Handle::NONE,
    normal_texture: Handle::NONE,
    metallic_factor: 1.0,
    roughness_factor: 1.0,
    metallic_roughness_texture: Handle::NONE,
};

impl BvhPrimitive {
    pub fn new(geometry: BvhGeometry, node: Handle<Node>, material: Handle<Material>) -> Self {
        Self {
            geometry,
            node,
            material,
        }
    }

    pub fn centroid(&self, model: &Model) -> Point3 {
        let trs = model.solved_trs.get(&self.node).unwrap();
        match &self.geometry {
            BvhGeometry::Triangle(triangle) => triangle.get_centroid(trs).into(),
            BvhGeometry::Sphere(sphere) => sphere.get_center(trs),
        }
    }

    pub fn min(&self, model: &Model) -> Point3 {
        let trs = model.solved_trs.get(&self.node).unwrap();
        match &self.geometry {
            BvhGeometry::Triangle(triangle) => triangle.min(trs),
            BvhGeometry::Sphere(sphere) => sphere.min(trs),
        }
    }

    pub fn max(&self, model: &Model) -> Point3 {
        let trs = model.solved_trs.get(&self.node).unwrap();
        match &self.geometry {
            BvhGeometry::Triangle(triangle) => triangle.max(trs),
            BvhGeometry::Sphere(sphere) => sphere.max(trs),
        }
    }

    pub fn intersects(&self, model: &Model, ray: &Ray) -> Option<Hit> {
        let trs = model.solved_trs.get(&self.node).unwrap();
        match &self.geometry {
            BvhGeometry::Triangle(triangle) => triangle.intersects(trs, ray),
            BvhGeometry::Sphere(sphere) => sphere.intersects(trs, ray),
        }
    }

    pub fn get_material<'m>(&self, model: &'m Model) -> &'m Material {
        let material = model
            .materials
            .get(self.material)
            .unwrap_or(&WHITE_MATERIAL);
        material
    }

    pub fn get_color(&self, model: &Model, hit: &Hit) -> Color {
        let geometry_color = self.geometry.get_color(hit);
        let uv = self.geometry.get_uv(hit);
        let material_color = self.get_material(model).get_color(model, &uv);
        geometry_color * material_color
    }

    pub fn get_normal(&self, model: &Model, hit: &Hit) -> Vec3 {
        match &self.geometry {
            BvhGeometry::Triangle(_) => {
                let uv = self.geometry.get_uv(hit);
                let normal = self.geometry.get_normal(hit);
                let tangent = self.geometry.get_tangent(hit);
                let bitangent = self.geometry.get_bitangent(hit);
                let material = self.get_material(model);
                material.get_normal(model, &uv, normal, tangent, bitangent)
            }
            BvhGeometry::Sphere(sphere) => {
                let trs = model.solved_trs.get(&self.node).unwrap();
                let inverse = trs.get_inversed();
                let hit_point = &inverse * hit.point;
                let normal = sphere.get_normal(&hit_point);
                let normal_matrix = Mat3::from(&inverse).get_transpose();
                (&normal_matrix * normal).get_normalized()
            }
        }
    }

    pub fn get_metallic_roughness(&self, model: &Model, hit: &Hit) -> (f32, f32) {
        match &self.geometry {
            BvhGeometry::Triangle(_) => {
                let material = self.get_material(model);
                let uv = self.geometry.get_uv(hit);
                material.get_metallic_roughness(model, &uv)
            }
            // TODO remember to transform hit point into model sphere
            BvhGeometry::Sphere(_) => (1.0, 1.0),
        }
    }

    /// Calculates the light coming out towards the viewer at a certain intersection
    pub fn get_radiance(&self, model: &Model, ir: &Irradiance) -> Color {
        self.get_material(model).get_radiance(ir, model)
    }

    fn from_triangle_mesh_impl<'m, Index: NumCast>(
        triangles: &TriangleMesh,
        node: Handle<Node>,
        material: Handle<Material>,
        model: &'m Model,
        indices: &[Index],
    ) -> Vec<BvhPrimitive> {
        let mut ret = vec![];

        let trs = model.solved_trs.get(&node).unwrap();
        let tangent_matrix = Mat3::from(&trs.trs);

        let inverse_trs = Inversed::from(&trs.trs);
        let normal_matrix = Mat3::from(&inverse_trs).get_transpose();

        for i in 0..(indices.len() / 3) {
            let mut a = triangles.vertices[indices[i * 3].to_usize().unwrap()].clone();
            a.pos = a.pos;
            a.ext.normal = &normal_matrix * a.ext.normal;
            a.ext.tangent = &tangent_matrix * a.ext.tangent;
            a.ext.bitangent = &tangent_matrix * a.ext.bitangent;

            let mut b = triangles.vertices[indices[i * 3 + 1].to_usize().unwrap()].clone();
            b.pos = b.pos;
            b.ext.normal = &normal_matrix * b.ext.normal;
            b.ext.tangent = &tangent_matrix * b.ext.tangent;
            b.ext.bitangent = &tangent_matrix * b.ext.bitangent;

            let mut c = triangles.vertices[indices[i * 3 + 2].to_usize().unwrap()].clone();
            c.pos = c.pos;
            c.ext.normal = &normal_matrix * c.ext.normal;
            c.ext.tangent = &tangent_matrix * c.ext.tangent;
            c.ext.bitangent = &tangent_matrix * c.ext.bitangent;

            let triangle = Triangle::new([a.pos, b.pos, c.pos]);
            let triangle = BvhTriangle::new(triangle, [a.ext, b.ext, c.ext]);
            let geometry = BvhGeometry::Triangle(triangle);
            let primitive = BvhPrimitive::new(geometry, node, material);
            ret.push(primitive)
        }

        ret
    }

    pub fn from_triangle_mesh(
        triangles: &TriangleMesh,
        node: Handle<Node>,
        material: Handle<Material>,
        model: &Model,
    ) -> Vec<BvhPrimitive> {
        let indices_len = triangles.indices.len() / triangles.index_size_in_bytes;

        match triangles.index_size_in_bytes {
            1 => {
                Self::from_triangle_mesh_impl(triangles, node, material, model, &triangles.indices)
            }
            2 => {
                let indices = unsafe {
                    std::slice::from_raw_parts(
                        triangles.indices.as_ptr() as *const u16,
                        indices_len,
                    )
                };

                Self::from_triangle_mesh_impl(triangles, node, material, model, indices)
            }
            4 => {
                let indices = unsafe {
                    std::slice::from_raw_parts(
                        triangles.indices.as_ptr() as *const u32,
                        indices_len,
                    )
                };

                Self::from_triangle_mesh_impl(triangles, node, material, model, indices)
            }
            _ => panic!("Index size not supported"),
        }
    }

    pub fn from_sphere(
        node: Handle<Node>,
        material: Handle<Material>,
        sphere: &Sphere,
    ) -> Vec<Self> {
        // Transforming a sphere is complicated. The trick is to store transform with sphere,
        // then pre-transform the ray, and post-transform the intersection point.
        let sphere = Sphere::new(sphere.get_model_center(), sphere.get_radius());
        let geometry = BvhGeometry::Sphere(sphere);
        let primitive = BvhPrimitive::new(geometry, node, material);

        vec![primitive]
    }
}
