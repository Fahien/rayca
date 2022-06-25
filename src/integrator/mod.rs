// Copyright © 2022-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

pub mod scratcher;
pub use scratcher::*;

use crate::{Color, Ray, Scene};

pub trait Integrator: Sync {
    fn trace(&self, scene: &Scene, ray: Ray, depth: u8) -> Color;
}
