// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use bon::Builder;

use crate::*;

#[derive(Builder, Debug, Default, Clone)]
pub struct Primitive {
    pub geometry: Handle<Geometry>,
    #[builder(default)]
    pub material: Handle<Material>,
}
