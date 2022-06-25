// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

pub mod scratcher;
pub use scratcher::*;

use crate::{Bvh, Color, Light, Node, Pack, Ray};

pub trait Integrator: Sync {
    fn trace(
        &self,
        ray: Ray,
        bvh: &Bvh,
        light_nodes: &[Node],
        lights: &Pack<Light>,
        depth: u32,
    ) -> Option<Color>;
}
