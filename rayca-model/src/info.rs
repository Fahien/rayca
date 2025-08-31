// Copyright © 2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

#[derive(Clone, Copy, PartialEq)]
pub struct NodeInfo {
    pub model: Handle<Model>,
    pub node: Handle<Node>,
}

pub type CameraInfo = NodeInfo;
