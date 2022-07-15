// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::{Light, Trs};

pub struct BvhLight<'m> {
    pub light: &'m Light,

    /// Transform in world space
    pub trs: &'m Trs,
}

impl<'m> BvhLight<'m> {
    pub fn new(light: &'m Light, trs: &'m Trs) -> Self {
        Self { light, trs }
    }
}
