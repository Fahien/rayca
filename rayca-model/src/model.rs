// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use crate::*;

/// Transforms in world space, ready to be used by a renderer
#[repr(transparent)]
pub struct WorldTrs {
    pub trs: Trs,
}

impl WorldTrs {
    pub fn new(trs: Trs) -> Self {
        Self { trs }
    }
}

impl std::ops::Deref for WorldTrs {
    type Target = Trs;

    fn deref(&self) -> &Self::Target {
        &self.trs
    }
}

/// Model representation based on glTF spec
pub struct Model {
    pub dir: PathBuf,
    pub name: String,
    pub root: Node,
    pub buffers: Pack<Buffer>,
    pub buffer_views: Pack<BufferView>,
    pub nodes: Pack<Node>,
    pub meshes: Pack<Mesh>,
    pub primitives: Pack<Primitive>,
    pub geometries: Pack<Geometry>,
    pub pbr_materials: Pack<PbrMaterial>,
    pub phong_materials: Pack<PhongMaterial>,
    pub materials: Pack<Material>,
    pub textures: Pack<Texture>,
    pub images: Pack<Image>,
    pub samplers: Pack<Sampler>,
    pub cameras: Pack<Camera>,
    pub scripts: Pack<Script>,
    pub lights: Pack<Light>,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            dir: PathBuf::from("."),
            name: "Unknown".to_string(),
            root: Node::default(),
            buffers: Pack::default(),
            buffer_views: Pack::default(),
            nodes: Pack::default(),
            meshes: Pack::default(),
            primitives: Pack::default(),
            geometries: Pack::default(),
            pbr_materials: Pack::default(),
            phong_materials: Pack::default(),
            materials: Pack::default(),
            textures: Pack::default(),
            images: Pack::default(),
            samplers: Pack::default(),
            cameras: Pack::default(),
            scripts: Pack::default(),
            lights: Pack::default(),
        }
    }
}

impl Model {
    pub fn new<S: Into<String>>(name: S) -> Self {
        let mut model = Self::default();
        model.name = name.into();
        model
    }

    pub fn get_uri(&self) -> PathBuf {
        self.dir.join(format!("{}.gltf", self.name))
    }

    pub fn find_image_handle_by_uri(&self, uri: &str) -> Handle<Image> {
        self.images
            .iter()
            .position(|image| image.uri == uri)
            .map(Handle::from)
            .unwrap_or_default()
    }

    pub fn get_node(&self, handle: Handle<Node>) -> Option<&Node> {
        self.nodes.get(handle)
    }

    pub fn get_geometry(&self, handle: Handle<Geometry>) -> Option<&Geometry> {
        self.geometries.get(handle)
    }

    pub fn get_geometry_mut(&mut self, handle: Handle<Geometry>) -> Option<&mut Geometry> {
        self.geometries.get_mut(handle)
    }

    pub fn get_material(&self, handle: Handle<Material>) -> Option<&Material> {
        self.materials.get(handle)
    }

    pub fn get_pbr_material(&self, handle: Handle<PbrMaterial>) -> Option<&PbrMaterial> {
        self.pbr_materials.get(handle)
    }

    pub fn get_pbr_material_mut(
        &mut self,
        handle: Handle<PbrMaterial>,
    ) -> Option<&mut PbrMaterial> {
        self.pbr_materials.get_mut(handle)
    }

    pub fn get_phong_material(&self, handle: Handle<PhongMaterial>) -> Option<&PhongMaterial> {
        self.phong_materials.get(handle)
    }

    pub fn get_phong_material_mut(
        &mut self,
        handle: Handle<PhongMaterial>,
    ) -> Option<&mut PhongMaterial> {
        self.phong_materials.get_mut(handle)
    }

    pub fn get_mesh(&self, handle: Handle<Mesh>) -> Option<&Mesh> {
        self.meshes.get(handle)
    }

    pub fn get_primitive(&self, handle: Handle<Primitive>) -> Option<&Primitive> {
        self.primitives.get(handle)
    }

    pub fn get_primitive_mut(&mut self, handle: Handle<Primitive>) -> Option<&mut Primitive> {
        self.primitives.get_mut(handle)
    }

    pub fn get_camera(&self, handle: Handle<Camera>) -> Option<&Camera> {
        self.cameras.get(handle)
    }

    pub fn get_light(&self, handle: Handle<Light>) -> Option<&Light> {
        self.lights.get(handle)
    }

    pub fn push_primitive(&mut self, primitive: Primitive) -> Handle<Primitive> {
        self.primitives.push(primitive)
    }

    pub fn push_camera(&mut self, camera: Camera) -> Handle<Camera> {
        self.cameras.push(camera)
    }

    pub fn push_node(&mut self, node: Node) -> Handle<Node> {
        self.nodes.push(node)
    }
}
