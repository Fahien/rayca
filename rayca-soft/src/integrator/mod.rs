// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

mod analyticdirect;
mod flat;
mod raytracer;
mod scratcher;

pub use analyticdirect::*;
pub use flat::*;
pub use raytracer::*;
pub use scratcher::*;

use crate::*;

pub trait Integrator: Sync {
    fn trace(
        &self,
        config: &Config,
        scene: &SceneDrawInfo,
        ray: Ray,
        tlas: &Tlas,
        depth: u32,
    ) -> Option<Color>;
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum IntegratorStrategy {
    Scratcher,
    Raytracer,
    Flat,
    AnalyticDirect,
}

impl IntegratorStrategy {
    pub fn get_integrator(&self) -> &'static dyn Integrator {
        match self {
            Self::Scratcher => {
                static SCRATCHER: Scratcher = Scratcher::new();
                &SCRATCHER
            }
            Self::Raytracer => {
                static RAYTRACER: Raytracer = Raytracer::new();
                &RAYTRACER
            }
            Self::Flat => {
                static FLAT: Flat = Flat::new();
                &FLAT
            }
            Self::AnalyticDirect => {
                static ANALYTIC_DIRECT: AnalyticDirect = AnalyticDirect::new();
                &ANALYTIC_DIRECT
            }
        }
    }
}

impl From<loader::sdtf::SdtfIntegratorStrategy> for IntegratorStrategy {
    fn from(value: loader::sdtf::SdtfIntegratorStrategy) -> Self {
        match value {
            loader::sdtf::SdtfIntegratorStrategy::Raytracer => IntegratorStrategy::Raytracer,
            loader::sdtf::SdtfIntegratorStrategy::AnalyticDirect => {
                IntegratorStrategy::AnalyticDirect
            }
        }
    }
}
