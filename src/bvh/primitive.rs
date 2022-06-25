// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

pub enum BvhGeometry {
    Triangle(Box<BvhTriangle>),
    Sphere(BvhSphere),
}

pub struct BvhPrimitive {
    pub geometry: BvhGeometry,
    pub node: Handle<Node>,
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
        let material = self.get_material(model);

        match &self.geometry {
            BvhGeometry::Triangle(triangle) => triangle.get_color(material, model, hit),
            BvhGeometry::Sphere(_) => material.color,
        }
    }

    pub fn get_normal(&self, model: &Model, hit: &Hit) -> Vec3 {
        match &self.geometry {
            BvhGeometry::Triangle(triangle) => {
                let material = self.get_material(model);
                triangle.get_normal(material, model, hit)
            }
            BvhGeometry::Sphere(sphere) => (sphere.center - hit.point).get_normalized(),
        }
    }

    pub fn get_metallic_roughness(&self, model: &Model, hit: &Hit) -> (f32, f32) {
        match &self.geometry {
            BvhGeometry::Triangle(triangle) => {
                let material = self.get_material(model);
                triangle.get_metallic_roughness(material, model, hit)
            }
            BvhGeometry::Sphere(_) => (1.0, 1.0),
        }
    }
}
