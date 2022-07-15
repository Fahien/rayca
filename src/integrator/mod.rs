// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

pub mod scratcher;
pub use scratcher::*;

use crate::{Bvh, Color, Light, Ray, Trs};

pub trait Integrator: Sync {
    fn trace(&self, ray: Ray, bvh: &Bvh, lights: &[(&Light, &Trs)], depth: u32) -> Option<Color>;
}
