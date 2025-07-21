// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use super::*;

#[derive(Default)]
pub struct Texture {
    pub image: Handle<Image>,
    pub sampler: Handle<Sampler>,
}

impl Texture {
    pub fn new(image: Handle<Image>, sampler: Handle<Sampler>) -> Self {
        Self { image, sampler }
    }
}
