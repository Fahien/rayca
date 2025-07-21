// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::hash::Hash;

use super::*;

pub struct NodeBuilder {
    pub id: usize,
    pub name: String,
    pub trs: Trs,
    pub children: Vec<Handle<Node>>,
    pub mesh: Handle<Mesh>,
    pub camera: Handle<Camera>,
    pub light: Handle<Light>,
}

impl NodeBuilder {
    pub fn new() -> Self {
        Self {
            id: 0,
            name: "Unknown".to_string(),
            trs: Trs::default(),
            children: vec![],
            mesh: Handle::NONE,
            camera: Handle::NONE,
            light: Handle::NONE,
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

    pub fn trs(mut self, trs: Trs) -> Self {
        self.trs = trs;
        self
    }

    pub fn translation(mut self, translation: Vec3) -> Self {
        self.trs.translation = translation;
        self
    }

    pub fn rotation(mut self, rotation: Quat) -> Self {
        self.trs.rotation = rotation;
        self
    }

    pub fn scale(mut self, scale: Vec3) -> Self {
        self.trs.scale = scale;
        self
    }

    pub fn matrix(mut self, matrix: Mat4) -> Self {
        self.trs.scale = matrix.get_scale();
        self.trs.rotation = matrix.get_rotation();
        self.trs.translation = matrix.get_translation();
        self
    }

    pub fn children(mut self, children: Vec<Handle<Node>>) -> Self {
        self.children = children;
        self
    }

    pub fn mesh(mut self, mesh: Handle<Mesh>) -> Self {
        self.mesh = mesh;
        self
    }

    pub fn camera(mut self, camera: Handle<Camera>) -> Self {
        self.camera = camera;
        self
    }

    pub fn light(mut self, light: Handle<Light>) -> Self {
        self.light = light;
        self
    }

    pub fn build(self) -> Node {
        let mut node = Node::new();
        node.id = self.id;
        node.name = self.name;

        node.trs = self.trs;

        node.children = self.children;
        node.mesh = self.mesh;
        node.camera = self.camera;
        node.light = self.light;

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
    pub camera: Handle<Camera>,
    pub light: Handle<Light>,
    pub mesh: Handle<Mesh>,
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
