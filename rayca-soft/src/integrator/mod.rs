// Copyright © 2022-2024
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

pub mod scratcher;
pub use scratcher::*;

use crate::*;

pub trait Integrator: Sync {
    fn trace(&self, scene: &SceneDrawInfo, ray: Ray, bvh: &Bvh, depth: u32) -> Option<Color>;
}
