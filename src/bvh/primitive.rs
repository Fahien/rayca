// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

pub enum BvhGeometry {
    Triangle(BvhTriangle),
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

pub struct BvhPrimitive<'m> {
    pub geometry: BvhGeometry,
    pub trs: &'m Trs,

    /// We store handle and model here as we will anyway need to use the model
    /// when quering material properties such as textures.
    pub material: Handle<Material>,
    pub model: &'m Model,
}

const WHITE_MATERIAL: Material = Material {
    color: Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    },
    albedo_texture: None,
    normal_texture: None,
    metallic_factor: 1.0,
    roughness_factor: 1.0,
    metallic_roughness_texture: None,
};

impl<'m> BvhPrimitive<'m> {
    pub fn new(
        geometry: BvhGeometry,
        trs: &'m Trs,
        material: Handle<Material>,
        model: &'m Model,
    ) -> Self {
        Self {
            geometry,
            trs,
            material,
            model,
        }
    }

    pub fn centroid(&self) -> &Vec3 {
        match &self.geometry {
            BvhGeometry::Triangle(triangle) => &triangle.centroid,
            BvhGeometry::Sphere(sphere) => &sphere.center,
        }
    }

    pub fn min(&self) -> Vec3 {
        match &self.geometry {
            BvhGeometry::Triangle(triangle) => triangle.min(),
            BvhGeometry::Sphere(sphere) => sphere.min(),
        }
    }

    pub fn max(&self) -> Vec3 {
        match &self.geometry {
            BvhGeometry::Triangle(triangle) => triangle.max(),
            BvhGeometry::Sphere(sphere) => sphere.max(),
        }
    }

    pub fn intersects(&self, ray: &Ray) -> Option<Hit> {
        match &self.geometry {
            BvhGeometry::Triangle(triangle) => triangle.intersects(&ray),
            BvhGeometry::Sphere(sphere) => {
                let ray = ray.clone();
                let inverse = Inversed::from(self.trs);
                let inverse_ray = &inverse * ray.clone();
                let mut hit = sphere.intersects(&inverse_ray);
                if let Some(hit) = hit.as_mut() {
                    let transformed_point = hit.point;
                    hit.point = self.trs * transformed_point;
                }
                hit
            }
        }
    }

    pub fn get_material(&self) -> &Material {
        let material = self
            .model
            .materials
            .get(self.material)
            .unwrap_or(&WHITE_MATERIAL);
        material
    }

    pub fn get_color(&self, hit: &Hit) -> Color {
        let geometry_color = self.geometry.get_color(hit);
        let uv = self.geometry.get_uv(hit);
        let material_color = self.get_material().get_color(&uv, self.model);
        geometry_color * material_color
    }

    pub fn get_normal(&self, hit: &Hit) -> Vec3 {
        match &self.geometry {
            BvhGeometry::Triangle(_) => {
                let uv = self.geometry.get_uv(hit);
                let normal = self.geometry.get_normal(hit);
                let tangent = self.geometry.get_tangent(hit);
                let bitangent = self.geometry.get_bitangent(hit);
                let material = self.get_material();
                material.get_normal(&uv, normal, tangent, bitangent, self.model)
            }
            BvhGeometry::Sphere(sphere) => {
                let inverse = self.trs.get_inversed();
                let hit_point = &inverse * hit.point;
                let normal = sphere.get_normal(&hit_point);
                let normal_matrix = Mat3::from(&inverse).get_transpose();
                (&normal_matrix * normal).get_normalized()
            }
        }
    }

    /// Calculates the light coming out towards the viewer at a certain intersection
    pub fn get_radiance(&self, light_intersection: &LightIntersection) -> Color {
        self.get_material()
            .get_radiance(light_intersection, self.model)
    }
}
