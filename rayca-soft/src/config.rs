// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use bon::Builder;

use crate::*;

#[derive(Copy, Clone, Builder)]
pub struct Config {
    #[builder(default = true)]
    pub bvh: bool,

    #[builder(default = IntegratorStrategy::Scratcher)]
    pub integrator: IntegratorStrategy,

    #[builder(default = 5)]
    /// Maximum recursion depth for ray tracing.
    pub max_depth: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self::builder().build()
    }
}
