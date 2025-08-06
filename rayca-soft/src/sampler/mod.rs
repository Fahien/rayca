// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

pub mod hemisphere;

use crate::*;
use hemisphere::*;

pub trait SoftSampler: Sync {
    /// Generates a sample from this distribution
    fn get_indirect_lighting(
        &self,
        config: &Config,
        scene: &SceneDrawInfo,
        tlas: &Tlas,
        hit: &Hit,
        n: Vec3,
        r: Vec3,
        material: &PhongMaterial,
        depth: u32,
    ) -> Color;
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SamplerStrategy {
    Hemisphere,
}

impl SamplerStrategy {
    pub fn get_sampler(&self) -> &'static dyn SoftSampler {
        match self {
            Self::Hemisphere => {
                static HEMISPHERE: HemisphereSampler = HemisphereSampler::new();
                &HEMISPHERE
            }
        }
    }
}

impl From<loader::sdtf::SdtfSamplerStrategy> for SamplerStrategy {
    fn from(value: loader::sdtf::SdtfSamplerStrategy) -> Self {
        use loader::sdtf::SdtfSamplerStrategy;
        match value {
            SdtfSamplerStrategy::Hemisphere => Self::Hemisphere,
        }
    }
}
