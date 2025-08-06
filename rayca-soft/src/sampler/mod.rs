// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

pub mod cosine;
pub mod hemisphere;

use crate::*;

use cosine::*;
use hemisphere::*;

pub trait SoftSampler: Sync {
    /// Returns a random direction in the hemisphere
    fn get_random_dir(&self, n: Vec3) -> Vec3;

    /// Returns the radiance for the given parameters.
    /// - `brdf`: The BRDF color.
    /// - `n`: The normal vector at the point of intersection.
    /// - `omega_i`: The incoming direction.
    /// - `indirect_sample`: The indirect sample color.
    /// - `weight`: The weight for the sample.
    fn get_radiance(
        &self,
        brdf: Color,
        n: Vec3,
        omega_i: Vec3,
        indirect_sample: Color,
        weight: f32,
    ) -> Color;
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SamplerStrategy {
    Hemisphere,
    Cosine,
}

impl SamplerStrategy {
    pub fn get_sampler(&self) -> &'static dyn SoftSampler {
        match self {
            Self::Hemisphere => {
                static HEMISPHERE: HemisphereSampler = HemisphereSampler::new();
                &HEMISPHERE
            }
            Self::Cosine => {
                static COSINE: CosineSampler = CosineSampler::new();
                &COSINE
            }
        }
    }
}

impl From<loader::sdtf::SdtfSamplerStrategy> for SamplerStrategy {
    fn from(value: loader::sdtf::SdtfSamplerStrategy) -> Self {
        use loader::sdtf::SdtfSamplerStrategy;
        match value {
            SdtfSamplerStrategy::Hemisphere => Self::Hemisphere,
            SdtfSamplerStrategy::Cosine => Self::Cosine,
        }
    }
}
