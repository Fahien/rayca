// Copyright © 2022-2024
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

pub struct BvhTriangle {
    pub triangle: Triangle,
    pub ext: TriangleExt,
}

impl BvhTriangle {
    pub fn new(triangle: Triangle, ext: TriangleExt) -> Self {
        Self { triangle, ext }
    }

    pub fn min(&self, trs: &Trs) -> Point3 {
        self.triangle.min(trs)
    }

    pub fn max(&self, trs: &Trs) -> Point3 {
        self.triangle.max(trs)
    }

    pub fn get_centroid(&self, trs: &Trs) -> Vec3 {
        self.triangle.get_centroid(trs)
    }

    pub fn get_vertex(&self, i: usize, trs: &Trs) -> Point3 {
        self.triangle.get_vertex(i, trs)
    }

    /// Returns the interpolation of the vertices colors
    pub fn interpolate_colors(&self, hit_uv: &Vec2) -> Color {
        self.ext.vertices[2].color * (1.0 - hit_uv.x - hit_uv.y)
            + self.ext.vertices[0].color * hit_uv.x
            + self.ext.vertices[1].color * hit_uv.y
    }

    /// Returns the interpolation of the vertices uvs
    pub fn interpolate_uvs(&self, hit_uv: &Vec2) -> Vec2 {
        self.ext.vertices[2].uv * (1.0 - hit_uv.x - hit_uv.y)
            + self.ext.vertices[0].uv * hit_uv.x
            + self.ext.vertices[1].uv * hit_uv.y
    }

    /// Returns the interpolation of the vertices normals
    pub fn interpolate_normals(&self, hit_uv: &Vec2) -> Vec3 {
        let n = self.ext.vertices[2].normal * (1.0 - hit_uv.x - hit_uv.y)
            + self.ext.vertices[0].normal * hit_uv.x
            + self.ext.vertices[1].normal * hit_uv.y;
        n.get_normalized()
    }

    /// Returns the interpolation of the vertices tangents
    pub fn interpolate_tangents(&self, hit_uv: &Vec2) -> Vec3 {
        let mut t = self.ext.vertices[2].tangent * (1.0 - hit_uv.x - hit_uv.y)
            + self.ext.vertices[0].tangent * hit_uv.x
            + self.ext.vertices[1].tangent * hit_uv.y;
        t.normalize();
        t
    }

    /// Returns the interpolation of the vertices bitangents
    pub fn interpolate_bitangents(&self, hit_uv: &Vec2) -> Vec3 {
        let mut b = self.ext.vertices[2].bitangent * (1.0 - hit_uv.x - hit_uv.y)
            + self.ext.vertices[0].bitangent * hit_uv.x
            + self.ext.vertices[1].bitangent * hit_uv.y;
        b.normalize();
        b
    }

    /// [Ray-triangle intersection](https://www.scratchapixel.com/lessons/3d-basic-rendering/ray-tracing-rendering-a-triangle/ray-triangle-intersection-geometric-solution)
    pub fn intersects(&self, trs: &Trs, ray: &Ray) -> Option<Hit> {
        self.triangle.intersects(trs, ray)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn intersect() {
        let mut model = Model::default();
        let geometry_handle = model
            .geometries
            .push(Geometry::TriangleMesh(TriangleMesh::unit()));
        let triangle_prim = model
            .primitives
            .push(Primitive::builder().geometry(geometry_handle).build());
        let mesh = model
            .meshes
            .push(Mesh::builder().primitive(triangle_prim).build());
        let node = model.nodes.push(Node::builder().mesh(mesh).build());
        model.root.children.push(node);

        let mut scene = Scene::default();
        let model_handle = scene.models.push(model);
        let node = Node::builder().model(model_handle).build();
        let node_handle = scene.nodes.push(node);
        scene.root.children.push(node_handle);

        let scene_draw_info = SceneDrawInfo::new(&scene);

        let triangles =
            BvhPrimitive::from_mesh(scene_draw_info.mesh_draw_infos[0], &scene_draw_info);
        let triangle_ref = &triangles[0];

        let ray = Ray::new(Point3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, -1.0));
        assert!(triangle_ref.intersects(&scene_draw_info, &ray).is_some());
        let ray = Ray::new(Point3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, 1.0));
        assert!(triangle_ref.intersects(&scene_draw_info, &ray).is_none());
    }
}
