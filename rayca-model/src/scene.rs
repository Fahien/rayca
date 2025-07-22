// Copyright © 2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::io;
use std::path::Path;

use bon::Builder;
use serde::*;

use crate::*;

/// Scene representation
#[derive(Serialize, Deserialize)]
pub struct Scene {
    pub name: String,
    pub nodes: Pack<Node>,
    pub models: Pack<ModelSource>,
    pub root: Node,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
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

    pub fn get_node(&self, handle: Handle<Node>) -> Option<&Node> {
        self.nodes.get(handle)
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
