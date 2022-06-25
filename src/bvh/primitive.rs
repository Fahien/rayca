// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

pub enum BvhGeometry {
    Triangle(Box<BvhTriangle>),
    Sphere(BvhSphere),
}

pub struct BvhPrimitive<'m> {
    pub geometry: BvhGeometry,
    pub trs: &'m Trs,
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

    pub fn intersects(&self, ray: &Ray) -> Option<Hit> {
        match &self.geometry {
            BvhGeometry::Triangle(triangle) => triangle.intersects(ray),
            BvhGeometry::Sphere(sphere) => {
                let ray = ray.clone();
                let inverse = Inversed::from(self.trs);
                let inverse_ray = &inverse * ray;
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
        let material = self.get_material();

        match &self.geometry {
            BvhGeometry::Triangle(triangle) => triangle.get_color(material, self.model, hit),
            BvhGeometry::Sphere(_) => material.color,
        }
    }

    pub fn get_normal(&self, hit: &Hit) -> Vec3 {
        match &self.geometry {
            BvhGeometry::Triangle(triangle) => {
                let material = self.get_material();
                triangle.get_normal(material, self.model, hit)
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

    pub fn get_metallic_roughness(&self, hit: &Hit) -> (f32, f32) {
        match &self.geometry {
            BvhGeometry::Triangle(triangle) => {
                let material = self.get_material();
                triangle.get_metallic_roughness(material, self.model, hit)
            }
            BvhGeometry::Sphere(_) => (1.0, 1.0),
        }
    }
}
