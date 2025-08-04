// Copyright © 2022-2025
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
            BvhGeometry::Sphere(sphere) => sphere.get_model_normal(&hit.point),
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
            .unwrap_or(&Material::DEFAULT);
        material
    }

    pub fn is_emissive(&self, scene: &SceneDrawInfo) -> bool {
        let model = scene.get_model(self.node.model);
        let material = self.get_material(scene);
        match material {
            Material::Pbr(handle) => model.pbr_materials.get(*handle).unwrap().is_emissive(),
            Material::Phong(handle) => model.phong_materials.get(*handle).unwrap().is_emissive(),
        }
    }

    pub fn get_color(&self, scene: &SceneDrawInfo, hit: &Hit) -> Color {
        let geometry_color = self.geometry.get_color(hit);
        let uv = self.geometry.get_uv(hit);
        let model = scene.get_model(self.node.model);
        let material_color = self.get_material(scene).get_color(model, uv);
        geometry_color * material_color
    }

    pub fn get_diffuse(&self, scene: &SceneDrawInfo, hit: &Hit, uv: Vec2) -> Color {
        let model = scene.get_model(self.node.model);
        let geometry_color = self.geometry.get_color(hit);
        let material_color = self.get_material(scene).get_diffuse(model, uv);
        geometry_color * material_color
    }

    pub fn get_specular(&self, scene: &SceneDrawInfo) -> Color {
        let model = scene.get_model(self.node.model);
        self.get_material(scene).get_specular(model)
    }

    pub fn get_shininess(&self, scene: &SceneDrawInfo) -> f32 {
        let model = scene.get_model(self.node.model);
        self.get_material(scene).get_shininess(model)
    }

    pub fn get_uv(&self, hit: &Hit) -> Vec2 {
        self.geometry.get_uv(hit)
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
                material.get_normal(model, uv, normal, tangent, bitangent)
            }
            BvhGeometry::Sphere(sphere) => {
                let trs = scene_draw_info.get_world_trs(self.node);
                let inverse = trs.get_inversed();
                let hit_point = &inverse * hit.point;
                let normal = sphere.get_model_normal(&hit_point);
                let normal_matrix = Mat3::from(&inverse).get_transpose();
                (&normal_matrix * normal).get_normalized()
            }
        }
    }

    /// Calculates the light coming out towards the viewer at a certain intersection
    pub fn get_radiance(&self, scene: &SceneDrawInfo, ir: &Irradiance) -> Color {
        let model = scene.get_model(self.node.model);
        let material = self.get_material(scene);
        match material {
            Material::Pbr(_) => {
                let pbr_material = material.get_pbr_material(model);
                ggx::get_radiance(pbr_material, ir, model)
            }
            Material::Phong(_) => {
                let phong_material = material.get_phong_material(model).unwrap();
                lambertian::get_radiance(phong_material, ir)
            }
        }
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
        match triangles.indices.index_type {
            ComponentType::I8 | ComponentType::U8 => {
                let indices = triangles.indices.get_indices::<u8>();
                Self::from_triangle_mesh_impl(triangles, node, material, scene, indices)
            }
            ComponentType::I16 | ComponentType::U16 => {
                let indices = triangles.indices.get_indices::<u16>();
                Self::from_triangle_mesh_impl(triangles, node, material, scene, indices)
            }
            ComponentType::U32 => {
                let indices = triangles.indices.get_indices::<u32>();
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

    pub fn from_quad_light(light_draw_info: LightDrawInfo, scene: &SceneDrawInfo) -> Vec<Self> {
        let quad_light = scene.get_quad_light(light_draw_info);

        let quad_a: Point3 = Point3::default();
        let normal = quad_light.get_normal();

        let a = Vertex::builder().position(quad_a).normal(normal).build();
        let b = Vertex::builder()
            .position(quad_a + quad_light.ab)
            .normal(normal)
            .build();
        let d = Vertex::builder()
            .position(quad_a + quad_light.ab + quad_light.ac)
            .normal(normal)
            .build();
        let c = Vertex::builder()
            .position(quad_a + quad_light.ac)
            .normal(normal)
            .build();

        let triangle1 = Triangle::new([a.pos, d.pos, b.pos]);
        let triangle2 = Triangle::new([a.pos, c.pos, d.pos]);

        let ext1 = [a.ext, d.ext, b.ext];
        let ext2 = [a.ext, c.ext, d.ext];

        let t1 = BvhTriangle::new(triangle1, ext1);
        let t2 = BvhTriangle::new(triangle2, ext2);

        let geometry1 = BvhGeometry::Triangle(t1);
        let geometry2 = BvhGeometry::Triangle(t2);

        let primitive1 = BvhPrimitive::new(geometry1, light_draw_info, quad_light.material);
        let primitive2 = BvhPrimitive::new(geometry2, light_draw_info, quad_light.material);

        vec![primitive1, primitive2]
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

        // We also consider area lights, as they should be detected by primary rays
        for light_draw_info in model_draw_info.light_draw_infos.iter().copied() {
            let light_primitives = BvhPrimitive::from_quad_light(light_draw_info, scene);
            primitives.extend(light_primitives);
        }

        Self { primitives }
    }

    pub fn get_primitive(&self, index: u32) -> &BvhPrimitive {
        self.primitives
            .get(index as usize)
            .expect("Primitive index out of bounds")
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
