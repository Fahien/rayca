// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

pub enum BvhGeometry {
    Triangle(Box<BvhTriangle>),
    Sphere(BvhSphere),
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

    pub fn centroid(&self) -> &Point3 {
        match &self.geometry {
            BvhGeometry::Triangle(triangle) => &triangle.centroid,
            BvhGeometry::Sphere(sphere) => &sphere.center,
        }
    }

    pub fn min(&self) -> Point3 {
        match &self.geometry {
            BvhGeometry::Triangle(triangle) => triangle.min(),
            BvhGeometry::Sphere(sphere) => sphere.min(),
        }
    }

    pub fn max(&self) -> Point3 {
        match &self.geometry {
            BvhGeometry::Triangle(triangle) => triangle.max(),
            BvhGeometry::Sphere(sphere) => sphere.max(),
        }
    }

    pub fn intersects(&self, model: &Model, ray: &Ray) -> Option<Hit> {
        match &self.geometry {
            BvhGeometry::Triangle(triangle) => triangle.intersects(ray),
            BvhGeometry::Sphere(sphere) => {
                let ray = ray.clone();
                let trs = model.solved_trs.get(&self.node).unwrap();
                let inverse = Inversed::from(&trs.trs);
                let inverse_ray = &inverse * ray;
                let mut hit = sphere.intersects(&inverse_ray);
                if let Some(hit) = hit.as_mut() {
                    let transformed_point = hit.point;
                    hit.point = &trs.trs * transformed_point;
                }
                hit
            }
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
}
