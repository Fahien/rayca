// Copyright © 2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::error::Error;
use std::io;
use std::path::{Path, PathBuf};

use bon::Builder;
use serde::*;

use crate::loader::sdtf::SdtfConfig;
use crate::*;

/// Represents a scene that can be serialized and deserialized
/// to and from a GLX file format.
#[derive(Default, Serialize, Deserialize)]
pub struct StoreScene {
    pub name: String,
    pub nodes: Pack<Node>,
    pub models: Pack<ModelSource>,
    pub root: Node,
}

impl StoreScene {
    pub fn new<S: Into<String>>(name: S) -> Self {
        let mut scene = Self::default();
        scene.name = name.into();
        scene
    }

    pub fn load_glx_path<P: AsRef<Path>>(glx_path: P, assets: &Assets) -> io::Result<Self> {
        let asset = assets.load(glx_path.as_ref())?;
        let data = asset.to_string();
        serde_json::from_str(&data).map_err(|e| {
            log::error!(
                "Failed to parse GLX file `{}`: {}",
                glx_path.as_ref().display(),
                e
            );
            io::Error::new(io::ErrorKind::InvalidData, e)
        })
    }
}

/// Scene representation
pub struct Scene {
    pub dir: PathBuf,
    pub name: String,
    pub nodes: Pack<Node>,
    pub models: Pack<Model>,
    pub root: Node,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            dir: PathBuf::from("."),
            name: "Unknown".to_string(),
            nodes: Pack::default(),
            models: Pack::default(),
            root: Node::default(),
        }
    }
}

impl Scene {
    pub fn new<S: Into<String>>(name: S) -> Self {
        let mut scene = Self::default();
        scene.name = name.into();
        scene
    }

    pub fn get_uri(&self) -> PathBuf {
        self.dir.join(format!("{}.gltf", self.name))
    }

    pub fn get_node(&self, handle: Handle<Node>) -> Option<&Node> {
        self.nodes.get(handle)
    }

    pub fn get_node_mut(&mut self, handle: Handle<Node>) -> &mut Node {
        self.nodes.get_mut(handle).unwrap()
    }

    pub fn get_model_node(&self, node: Handle<Node>, model: Handle<Model>) -> Option<&Node> {
        self.models.get(model).and_then(|m| m.get_node(node))
    }

    pub fn get_model(&self, handle: Handle<Model>) -> Option<&Model> {
        self.models.get(handle)
    }

    pub fn get_model_mut(&mut self, handle: Handle<Model>) -> &mut Model {
        self.models.get_mut(handle).unwrap()
    }

    /// Convenience method to push a model into the scene. This is going to create a node for the model
    /// and add it to the root node.
    pub fn push_model(&mut self, model: Model) -> Handle<Node> {
        let model_handle = self.models.push(model);
        let node = Node::builder().model(model_handle).build();
        let node_handle = self.nodes.push(node);
        self.root.children.push(node_handle);
        node_handle
    }

    pub fn push_gltf_from_path<P: AsRef<Path>>(
        &mut self,
        path: P,
        assets: &Assets,
    ) -> Result<Handle<Node>, Box<dyn Error>> {
        let model = Model::load_gltf_path(path, assets)?;
        Ok(self.push_model(model))
    }

    pub fn push_sdtf_from_path<P: AsRef<Path>>(
        &mut self,
        path: P,
    ) -> Result<(Handle<Node>, SdtfConfig), Box<dyn Error>> {
        let (model, config) = Model::load_sdtf_path(path)?;
        let model_handle = self.models.push(model);
        let node = Node::builder().model(model_handle).build();
        let node_handle = self.nodes.push(node);
        self.root.children.push(node_handle);
        Ok((node_handle, config))
    }
}

#[derive(Serialize, Deserialize, Builder)]
pub struct ModelSource {
    pub uri: String,
}

impl Default for ModelSource {
    fn default() -> Self {
        Self {
            uri: "Unknown".to_string(),
        }
    }
}

impl ModelSource {
    pub fn new<S: Into<String>>(uri: S) -> Self {
        Self { uri: uri.into() }
    }
}
