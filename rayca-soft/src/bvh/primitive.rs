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

    /// The node in the scene graph that this primitive belongs to.
    pub node: NodeDrawInfo,

    /// For quering material properties such as textures.
    pub material: Handle<Material>,
}

impl BvhPrimitive {
    pub fn new(geometry: BvhGeometry, node: NodeDrawInfo, material: Handle<Material>) -> Self {
        Self {
            geometry,
            node,
            material,
        }
    }

    pub fn get_centroid(&self, scene: &SceneDrawInfo) -> Point3 {
        let trs = scene.get_world_trs(self.node);
        match &self.geometry {
            BvhGeometry::Triangle(triangle) => triangle.get_centroid(trs).into(),
            BvhGeometry::Sphere(sphere) => sphere.get_center(trs),
        }
    }

    pub fn min(&self, scene: &SceneDrawInfo) -> Point3 {
        let trs = scene.get_world_trs(self.node);
        match &self.geometry {
            BvhGeometry::Triangle(triangle) => triangle.min(trs),
            BvhGeometry::Sphere(sphere) => sphere.min(trs),
        }
    }

    pub fn max(&self, scene: &SceneDrawInfo) -> Point3 {
        let trs = scene.get_world_trs(self.node);
        match &self.geometry {
            BvhGeometry::Triangle(triangle) => triangle.max(trs),
            BvhGeometry::Sphere(sphere) => sphere.max(trs),
        }
    }

    pub fn intersects(&self, scene: &SceneDrawInfo, ray: &Ray) -> Option<Hit> {
        let trs = scene.get_world_trs(self.node);
        match &self.geometry {
            BvhGeometry::Triangle(triangle) => triangle.intersects(trs, ray),
            BvhGeometry::Sphere(sphere) => sphere.intersects(trs, ray),
        }
    }

    pub fn get_material<'m>(&self, scene: &'m SceneDrawInfo) -> &'m Material {
        let model = scene.get_model(self.node.model);
        let material = model
            .materials
            .get(self.material)
            .unwrap_or(&Material::WHITE);
        material
    }

    pub fn get_color(&self, scene: &SceneDrawInfo, hit: &Hit) -> Color {
        let geometry_color = self.geometry.get_color(hit);
        let uv = self.geometry.get_uv(hit);
        let model = scene.get_model(self.node.model);
        let material_color = self.get_material(scene).get_color(model, &uv);
        geometry_color * material_color
    }

    pub fn get_normal(&self, scene_draw_info: &SceneDrawInfo, hit: &Hit) -> Vec3 {
        match &self.geometry {
            BvhGeometry::Triangle(_) => {
                let uv = self.geometry.get_uv(hit);
                let normal = self.geometry.get_normal(hit);
                let tangent = self.geometry.get_tangent(hit);
                let bitangent = self.geometry.get_bitangent(hit);
                let material = self.get_material(scene_draw_info);
                let model = scene_draw_info.get_model(self.node.model);
                material.get_normal(model, &uv, normal, tangent, bitangent)
            }
            BvhGeometry::Sphere(sphere) => {
                let trs = scene_draw_info.get_world_trs(self.node);
                let inverse = trs.get_inversed();
                let hit_point = &inverse * hit.point;
                let normal = sphere.get_normal(&hit_point);
                let normal_matrix = Mat3::from(&inverse).get_transpose();
                (&normal_matrix * normal).get_normalized()
            }
        }
    }

    pub fn get_metallic_roughness(&self, scene: &SceneDrawInfo, hit: &Hit) -> (f32, f32) {
        match &self.geometry {
            BvhGeometry::Triangle(_) => {
                let material = self.get_material(scene);
                let uv = self.geometry.get_uv(hit);
                let model = scene.get_model(self.node.model);
                material.get_metallic_roughness(model, &uv)
            }
            // TODO remember to transform hit point into model sphere
            BvhGeometry::Sphere(_) => (1.0, 1.0),
        }
    }

    /// Calculates the light coming out towards the viewer at a certain intersection
    pub fn get_radiance(&self, scene: &SceneDrawInfo, ir: &Irradiance) -> Color {
        let model = scene.get_model(self.node.model);
        ggx::get_radiance(self.get_material(scene), ir, model)
    }

    fn from_triangle_mesh_impl<'m, Index: NumCast>(
        triangles: &TriangleMesh,
        node: NodeDrawInfo,
        material: Handle<Material>,
        scene: &'m SceneDrawInfo,
        indices: &[Index],
    ) -> Vec<BvhPrimitive> {
        let mut ret = vec![];

        let trs = scene.get_world_trs(node);
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
        node: NodeDrawInfo,
        material: Handle<Material>,
        scene: &SceneDrawInfo,
    ) -> Vec<BvhPrimitive> {
        let index_count = triangles.indices.get_index_count();

        match triangles.indices.index_type {
            ComponentType::I8 | ComponentType::U8 => Self::from_triangle_mesh_impl(
                triangles,
                node,
                material,
                scene,
                &triangles.indices.indices,
            ),
            ComponentType::I16 | ComponentType::U16 => {
                let indices = unsafe {
                    std::slice::from_raw_parts(
                        triangles.indices.indices.as_ptr() as *const u16,
                        index_count,
                    )
                };

                Self::from_triangle_mesh_impl(triangles, node, material, scene, indices)
            }
            ComponentType::U32 => {
                let indices = unsafe {
                    std::slice::from_raw_parts(
                        triangles.indices.indices.as_ptr() as *const u32,
                        index_count,
                    )
                };

                Self::from_triangle_mesh_impl(triangles, node, material, scene, indices)
            }
            _ => panic!("Index type not supported"),
        }
    }

    pub fn from_sphere(
        sphere: &Sphere,
        node: NodeDrawInfo,
        material: Handle<Material>,
    ) -> Vec<Self> {
        // Transforming a sphere is complicated. The trick is to store transform with sphere,
        // then pre-transform the ray, and post-transform the intersection point.
        let sphere = Sphere::new(sphere.get_model_center(), sphere.get_model_radius());
        let geometry = BvhGeometry::Sphere(sphere);
        let primitive = BvhPrimitive::new(geometry, node, material);

        vec![primitive]
    }

    pub fn from_primitive(
        primitive: &Primitive,
        node: NodeDrawInfo,
        scene: &SceneDrawInfo,
    ) -> Vec<BvhPrimitive> {
        let model = scene.get_model(node.model);
        let geometry = model.get_geometry(primitive.geometry).unwrap();
        match geometry {
            Geometry::TriangleMesh(triangles) => {
                BvhPrimitive::from_triangle_mesh(triangles, node, primitive.material, scene)
            }
            Geometry::Sphere(sphere) => BvhPrimitive::from_sphere(sphere, node, primitive.material),
        }
    }

    /// Returns a collection of primitives ready for the BVH structure.
    /// The primitives returned are already transformed into world space.
    /// Sphere are not transformed in world space, as the intersection algorithm
    /// is designed to transform the ray and the intersection point.
    pub fn from_mesh(mesh_draw_info: MeshDrawInfo, scene: &SceneDrawInfo) -> Vec<Self> {
        let mut primitives = vec![];

        // Collect primitives
        let mesh = scene.get_mesh(mesh_draw_info);
        let model = scene.get_model(mesh_draw_info.model);
        for prim_handle in mesh.primitives.iter() {
            let prim = model.primitives.get(*prim_handle).unwrap();
            let prims = Self::from_primitive(prim, mesh_draw_info, scene);
            primitives.extend(prims);
        }

        primitives
    }
}

#[derive(Default)]
pub struct BvhModel {
    pub primitives: Vec<BvhPrimitive>,
}

impl BvhModel {
    pub fn from_model(model_draw_info: &ModelDrawInfo, scene: &SceneDrawInfo) -> Self {
        let mut primitives = vec![];

        for mesh_draw_info in model_draw_info.mesh_draw_infos.iter().copied() {
            let mesh_primitives = BvhPrimitive::from_mesh(mesh_draw_info, scene);
            primitives.extend(mesh_primitives);
        }

        Self { primitives }
    }
}

#[derive(Default)]
pub struct BvhScene {
    pub models: Vec<BvhModel>,
}

impl BvhScene {
    pub fn from_scene(scene_draw_info: &SceneDrawInfo) -> Self {
        let mut models = vec![];

        for model_draw_info in scene_draw_info.model_draw_infos.values() {
            let bvh_model = BvhModel::from_model(model_draw_info, scene_draw_info);
            models.push(bvh_model);
        }

        Self { models }
    }
}
