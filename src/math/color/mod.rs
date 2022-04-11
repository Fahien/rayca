// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

pub mod rgb8;
pub mod rgba8;

pub use rgb8::*;
pub use rgba8::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ColorType {
    RGB8,
    RGBA8,
}

impl ColorType {
    pub fn channels(&self) -> usize {
        match self {
            ColorType::RGB8 => 3,
            ColorType::RGBA8 => 4,
        }
    }
}

impl Default for ColorType {
    fn default() -> Self {
        Self::RGBA8
    }
}

pub trait Color: Copy {
    fn color_type() -> ColorType;
}
