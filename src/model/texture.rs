// Copyright © 2022-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::{Handle, Image, Sampler};

#[derive(Debug, Default)]
pub struct Texture {
    pub image: Handle<Image>,
    pub sampler: Handle<Sampler>,
}

impl Texture {
    pub fn new(image: Handle<Image>, sampler: Handle<Sampler>) -> Self {
        Self { image, sampler }
    }
}
