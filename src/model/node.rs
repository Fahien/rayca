// Copyright © 2022-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::hash::Hash;

use crate::{Camera, GltfMesh, Handle, Light, Mat4, Quat, Trs, Vec3};

pub struct NodeBuilder {
    pub id: usize,
    pub name: String,
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    pub children: Vec<Handle<Node>>,
    pub mesh: Option<Handle<GltfMesh>>,
    pub camera: Option<Handle<Camera>>,
    pub light: Option<Handle<Light>>,
}

impl NodeBuilder {
    pub fn new() -> Self {
        Self {
            id: 0,
            name: "Unknown".to_string(),
            translation: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::default(),
            scale: Vec3::new(1.0, 1.0, 1.0),
            children: vec![],
            mesh: None,
            camera: None,
            light: None,
        }
    }

    pub fn id(mut self, id: usize) -> Self {
        self.id = id;
        self
    }

    pub fn name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn translation(mut self, translation: Vec3) -> Self {
        self.translation = translation;
        self
    }

    pub fn rotation(mut self, rotation: Quat) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn scale(mut self, scale: Vec3) -> Self {
        self.scale = scale;
        self
    }

    pub fn matrix(mut self, matrix: Mat4) -> Self {
        self.scale = matrix.get_scale();
        self.rotation = matrix.get_rotation();
        self.translation = matrix.get_translation();
        self
    }

    pub fn children(mut self, children: Vec<Handle<Node>>) -> Self {
        self.children = children;
        self
    }

    pub fn mesh(mut self, mesh: Handle<GltfMesh>) -> Self {
        self.mesh = Some(mesh);
        self
    }

    pub fn camera(mut self, camera: Handle<Camera>) -> Self {
        self.camera = Some(camera);
        self
    }

    pub fn light(mut self, light: Handle<Light>) -> Self {
        self.light = Some(light);
        self
    }

    pub fn build(self) -> Node {
        let mut node = Node::new();
        node.id = self.id;
        node.name = self.name;

        node.trs.scale = self.scale;
        node.trs.rotation = self.rotation;
        node.trs.translation = self.translation;

        node.children = self.children;
        node.camera = self.camera;
        node.light = self.light;
        node.mesh = self.mesh;

        node
    }
}

impl Default for NodeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Default, Clone)]
pub struct Node {
    pub id: usize,
    pub name: String,
    pub camera: Option<Handle<Camera>>,
    pub light: Option<Handle<Light>>,
    pub mesh: Option<Handle<GltfMesh>>,
    pub trs: Trs,
    pub children: Vec<Handle<Node>>,
}

impl Node {
    pub fn builder() -> NodeBuilder {
        NodeBuilder::new()
    }

    pub fn new() -> Self {
        Self {
            name: String::from("Unknown"),
            mesh: None,
            ..Default::default()
        }
    }
}

impl Hash for Node {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Node {}
