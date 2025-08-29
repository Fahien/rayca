// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

pub mod brdf;
pub mod cosine;
pub mod hemisphere;
pub mod nee;
pub mod sample;

use crate::*;

use brdf::*;
use cosine::*;
use hemisphere::*;
use nee::*;

pub trait SoftSampler: Sync {
    /// Returns a random direction in the hemisphere
    fn get_random_dir(&self, hit: &mut HitInfo) -> Vec3;

    /// Returns the radiance for the given parameters.
    /// - `hit`: The hit information.
    /// - `omega_i`: The incoming direction.
    /// - `indirect_sample`: The indirect sample color.
    /// - `weight`: The weight for the sample.
    fn get_radiance(
        &self,
        hit: &mut HitInfo,
        omega_i: Vec3,
        indirect_sample: Color,
        weight: f32,
    ) -> Color;
}

pub trait DirectSampler: Sync {
    fn get_direct_lighting(&self, config: &Config, hit: &mut HitInfo) -> Color;
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SamplerStrategy {
    None,
    Nee,
    Hemisphere,
    Cosine,
    Brdf,
}

impl SamplerStrategy {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn get_direct_sampler(&self) -> &'static dyn DirectSampler {
        match self {
            Self::None => {
                static NONE: NoSampler = NoSampler {};
                &NONE
            }
            Self::Nee => {
                static NEE: NextEventEstimationSampler = NextEventEstimationSampler {};
                &NEE
            }
            _ => panic!("Unsupported direct sampler strategy: {:?}", self),
        }
    }

    pub fn get_indirect_sampler(&self) -> &'static dyn SoftSampler {
        match self {
            Self::Hemisphere => {
                static HEMISPHERE: HemisphereSampler = HemisphereSampler::new();
                &HEMISPHERE
            }
            Self::Cosine => {
                static COSINE: CosineSampler = CosineSampler::new();
                &COSINE
            }
            Self::Brdf => {
                static BRDF: BrdfSampler = BrdfSampler::new();
                &BRDF
            }
            _ => panic!("Unsupported indirect sampler strategy: {:?}", self),
        }
    }
}

impl From<loader::sdtf::SdtfSamplerStrategy> for SamplerStrategy {
    fn from(value: loader::sdtf::SdtfSamplerStrategy) -> Self {
        use loader::sdtf::SdtfSamplerStrategy;
        match value {
            SdtfSamplerStrategy::None => Self::None,
            SdtfSamplerStrategy::Nee => Self::Nee,
            SdtfSamplerStrategy::Hemisphere => Self::Hemisphere,
            SdtfSamplerStrategy::Cosine => Self::Cosine,
            SdtfSamplerStrategy::Brdf => Self::Brdf,
        }
    }
}

pub struct NoSampler;

impl DirectSampler for NoSampler {
    fn get_direct_lighting(&self, _config: &Config, _hit: &mut HitInfo) -> Color {
        // No direct lighting calculation, return black
        Color::BLACK
    }
}
