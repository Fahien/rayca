// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use serde::*;

use crate::*;

#[derive(Clone, Default)]
pub struct NodeBuilder {
    name: Option<String>,
    trs: Trs,
    children: Vec<Handle<Node>>,
    mesh: Option<Handle<Mesh>>,
    camera: Option<Handle<Camera>>,
    script: Option<Handle<Script>>,
    model: Option<Handle<ModelSource>>,
}

impl NodeBuilder {
    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn trs(mut self, trs: Trs) -> Self {
        self.trs = trs;
        self
    }

    pub fn children(mut self, children: Vec<Handle<Node>>) -> Self {
        self.children = children;
        self
    }

    pub fn mesh(mut self, mesh: Handle<Mesh>) -> Self {
        self.mesh.replace(mesh);
        self
    }

    pub fn camera(mut self, camera: Handle<Camera>) -> Self {
        self.camera.replace(camera);
        self
    }

    pub fn script(mut self, script: Handle<Script>) -> Self {
        self.script.replace(script);
        self
    }

    pub fn model(mut self, model: Handle<ModelSource>) -> Self {
        self.model.replace(model);
        self
    }

    pub fn build(self) -> Node {
        Node {
            name: self.name,
            trs: self.trs,
            children: self.children,
            mesh: self.mesh,
            camera: self.camera,
            script: self.script,
            model: self.model,
            ..Default::default()
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Node {
    pub name: Option<String>,

    #[serde(flatten)]
    pub trs: Trs,

    #[serde(default)]
    pub children: Vec<Handle<Node>>,

    pub mesh: Option<Handle<Mesh>>,

    pub camera: Option<Handle<Camera>>,

    pub script: Option<Handle<Script>>,

    pub model: Option<Handle<ModelSource>>,
}

impl Default for Node {
    fn default() -> Self {
        Self {
            name: None,
            trs: Trs::default(),
            children: Vec::new(),
            mesh: None,
            camera: None,
            script: None,
            model: None,
        }
    }
}

impl Node {
    pub fn builder() -> NodeBuilder {
        NodeBuilder::default()
    }

    pub fn new() -> Self {
        Self::default()
    }
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{ \"translation\": {}, \"rotation\": {}, \"scale\": {}",
            self.trs.translation, self.trs.rotation, self.trs.scale
        )?;
        if let Some(name) = &self.name {
            write!(f, ", \"name\": \"{}\"", name)?;
        }
        if let Some(camera) = &self.camera {
            write!(f, ", \"camera\": {}", camera.id)?;
        }
        if let Some(mesh) = &self.mesh {
            write!(f, ", \"mesh\": {}", mesh.id)?;
        }
        if let Some(model) = &self.model {
            write!(f, ", \"model\": {}", model.id)?;
        }
        if !self.children.is_empty() {
            write!(f, ", \"children\": [")?;
            for (i, child) in self.children.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", child.id)?;
            }
            write!(f, "]")?;
        }
        write!(f, " }}")?;
        Ok(())
    }
}
