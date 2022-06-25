// Copyright © 2022-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::{Integrator, Scratcher};

pub struct Config {
    pub bvh: bool,
    pub integrator: Box<dyn Integrator>,
}

impl Default for Config {
    fn default() -> Self {
        Self::new(true, Box::<Scratcher>::default())
    }
}

impl Config {
    pub fn new(bvh: bool, integrator: Box<dyn Integrator>) -> Self {
        Self { bvh, integrator }
    }
}
