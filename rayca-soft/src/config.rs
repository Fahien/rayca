// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use bon::Builder;
use rayca_model::loader::sdtf::SdtfConfig;

use crate::*;

#[derive(Copy, Clone, Builder)]
pub struct Config {
    #[builder(default = true)]
    pub bvh: bool,

    /// Number of samples to use for each light.
    #[builder(default = 1)]
    pub light_samples: u32,

    /// Whether light samples should be stratified in a `sqrt(light_samples) * sqrt(light_samples)` grid.
    #[builder(default = false)]
    pub light_stratify: bool,

    /// Number of samples to collect for each pixel.
    #[builder(default = 1)]
    pub samples_per_pixel: u32,

    #[builder(default = IntegratorStrategy::Scratcher)]
    pub integrator: IntegratorStrategy,

    /// Maximum recursion depth for ray tracing.
    #[builder(default = 5)]
    pub max_depth: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self::builder().build()
    }
}

impl Config {
    pub fn apply(&mut self, sdtf_config: SdtfConfig) {
        self.max_depth = sdtf_config.max_depth as u32;
        self.light_samples = sdtf_config.light_samples;
        self.light_stratify = sdtf_config.light_stratify;
        self.samples_per_pixel = sdtf_config.samples_per_pixel;
        self.integrator = sdtf_config.integrator.into();
    }

    pub fn get_strate_count(&self) -> u32 {
        if self.light_stratify {
            (self.light_samples as f32).sqrt() as u32
        } else {
            1
        }
    }
}
