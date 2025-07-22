// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

#[derive(Debug, Default)]
pub struct Sampler {}

#[derive(Default)]

pub struct TextureBuilder {
    pub image: Handle<Image>,
    pub sampler: Handle<Sampler>,
}

impl TextureBuilder {
    pub fn image(mut self, image: Handle<Image>) -> Self {
        self.image = image;
        self
    }

    pub fn sampler(mut self, sampler: Handle<Sampler>) -> Self {
        self.sampler = sampler;
        self
    }

    pub fn build(self) -> Texture {
        Texture {
            image: self.image,
            sampler: self.sampler,
        }
    }
}

#[derive(Debug, Default)]
pub struct Texture {
    pub image: Handle<Image>,
    pub sampler: Handle<Sampler>,
}

impl Texture {
    pub fn builder() -> TextureBuilder {
        TextureBuilder::default()
    }

    pub fn new(image: Handle<Image>, sampler: Handle<Sampler>) -> Self {
        Self { image, sampler }
    }
}

impl std::fmt::Display for Texture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        let mut need_comma = false;
        if self.image.is_valid() {
            write!(f, "\"source\": {}", self.image.id)?;
            need_comma = true;
        }
        if self.sampler.is_valid() {
            if need_comma {
                write!(f, ", ")?;
            }
            write!(f, "\"sampler\": {}", self.sampler.id)?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}
