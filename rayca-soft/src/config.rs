// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::IntegratorType;

#[derive(Copy, Clone)]
pub struct Config {
    pub bvh: bool,
    pub integrator: IntegratorType,
}

impl Default for Config {
    fn default() -> Self {
        Self::new(true, IntegratorType::Scratcher)
    }
}

impl Config {
    pub fn new(bvh: bool, integrator: IntegratorType) -> Self {
        Self { bvh, integrator }
    }
}
