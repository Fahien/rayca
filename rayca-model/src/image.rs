// Copyright © 2024-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

#[derive(Clone, Default)]
pub struct ImageBuilder {
    uri: String,
}

impl ImageBuilder {
    pub fn uri<S: Into<String>>(mut self, uri: S) -> Self {
        self.uri = uri.into();
        self
    }

    pub fn build(self) -> Image {
        Image { uri: self.uri }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Image {
    pub uri: String,
}

impl Image {
    pub fn builder() -> ImageBuilder {
        ImageBuilder::default()
    }
}

impl std::fmt::Display for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ \"uri\": \"{}\" }}", self.uri)
    }
}
