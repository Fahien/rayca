// Copyright © 2022-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

pub mod scratcher;
pub use scratcher::*;

use crate::{Color, Hit, Scene};

pub trait Integrator {
    fn get_color(&self, scene: &Scene, hit: Hit) -> Color;
}
