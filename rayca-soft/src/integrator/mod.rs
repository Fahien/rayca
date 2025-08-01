// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

mod flat;
mod raytracer;
mod scratcher;

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
pub enum IntegratorType {
    Scratcher,
    Raytracer,
    Flat,
}

impl IntegratorType {
    pub fn get_integrator(&self) -> &'static dyn Integrator {
        match self {
            IntegratorType::Scratcher => {
                static SCRATCHER: Scratcher = Scratcher::new();
                &SCRATCHER
            }
            IntegratorType::Raytracer => {
                static RAYTRACER: Raytracer = Raytracer::new();
                &RAYTRACER
            }
            IntegratorType::Flat => {
                static FLAT: Flat = Flat::new();
                &FLAT
            }
        }
    }
}
