// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use bon::Builder;
use serde::*;

use crate::*;

#[derive(Builder, Clone, Serialize, Deserialize, Default)]
pub struct Node {
    pub name: Option<String>,

    #[serde(flatten)]
    #[builder(default)]
    pub trs: Trs,

    #[serde(default)]
    #[builder(default)]
    pub children: Vec<Handle<Node>>,

    pub mesh: Option<Handle<Mesh>>,

    pub camera: Option<Handle<Camera>>,

    pub light: Option<Handle<Light>>,

    pub script: Option<Handle<Script>>,

    pub model: Option<Handle<Model>>,
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
