// Copyright © 2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

mod pbr;
mod phong;

pub use pbr::*;
pub use phong::*;

use crate::*;

#[derive(Copy, Clone, Debug)]
pub enum Material {
    Pbr(Handle<PbrMaterial>),
    Phong(Handle<PhongMaterial>),
}

impl Default for Material {
    fn default() -> Self {
        Material::Pbr(Handle::NONE)
    }
}

impl Material {
    pub const DEFAULT: Material = Material::Pbr(Handle::NONE);

    pub fn get_pbr_material_handle(&self) -> Handle<PbrMaterial> {
        if let Material::Pbr(handle) = self {
            *handle
        } else {
            Handle::NONE
        }
    }

    pub fn get_phong_material_handle(&self) -> Handle<PhongMaterial> {
        if let Material::Phong(handle) = self {
            *handle
        } else {
            Handle::NONE
        }
    }

    pub fn get_pbr_material<'m>(&self, model: &'m Model) -> Option<&'m PbrMaterial> {
        match self {
            Material::Pbr(handle) => model.pbr_materials.get(*handle),
            _ => None,
        }
    }

    pub fn get_pbr_material_mut<'m>(&mut self, model: &'m mut Model) -> &'m mut PbrMaterial {
        match self {
            Material::Pbr(handle) => model.pbr_materials.get_mut(*handle).unwrap(),
            _ => panic!("Expected PbrMaterial, found different material type"),
        }
    }

    pub fn get_phong_material<'m>(&self, model: &'m Model) -> Option<&'m PhongMaterial> {
        match self {
            Material::Phong(handle) => model.phong_materials.get(*handle),
            _ => None,
        }
    }

    pub fn get_phong_material_mut<'m>(&mut self, model: &'m mut Model) -> &'m mut PhongMaterial {
        match self {
            Material::Phong(handle) => model.phong_materials.get_mut(*handle).unwrap(),
            _ => panic!("Expected PhongMaterial, found different material type"),
        }
    }

    /// Returns the base color of the material.
    pub fn get_color(&self, model: &Model, uv: Vec2) -> Color {
        match self {
            Material::Pbr(_) => self.get_pbr_material(model).unwrap().get_color(model, uv),
            Material::Phong(_) => self.get_phong_material(model).unwrap().get_color(),
        }
    }

    /// Returns the normal vector based on the material.
    pub fn get_normal(
        &self,
        model: &Model,
        uv: Vec2,
        normal: Vec3,
        tangent: Vec3,
        bitangent: Vec3,
    ) -> Vec3 {
        match self {
            Material::Pbr(_) => self
                .get_pbr_material(model)
                .unwrap()
                .get_normal(model, uv, normal, tangent, bitangent),
            Material::Phong(_) => normal,
        }
    }
}
