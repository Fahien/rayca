// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::{Color, ColorType, ColorTyped, RGB8};

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, Eq, PartialEq)]
pub struct RGBA8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl ColorTyped for RGBA8 {
    fn color_type() -> ColorType {
        ColorType::RGBA8
    }
}

impl RGBA8 {
    pub const WHITE: Self = Self {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };

    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn white() -> Self {
        Self::new(255, 255, 255, 255)
    }

    /// Over blend this color with another on top of itself
    pub fn over(&mut self, top: Color) {
        let mut self_color: Color = (*self).into();
        self_color.over(top);
        *self = self_color.into();
    }
}

impl From<u32> for RGBA8 {
    fn from(color: u32) -> Self {
        Self::new(
            (color >> 24) as u8,
            (color >> 16) as u8,
            (color >> 8) as u8,
            color as u8,
        )
    }
}

impl From<Color> for RGBA8 {
    fn from(color: Color) -> Self {
        Self::new(
            (color.r * 255.0) as u8,
            (color.g * 255.0) as u8,
            (color.b * 255.0) as u8,
            (color.a * 255.0) as u8,
        )
    }
}
impl From<RGB8> for RGBA8 {
    fn from(color: RGB8) -> Self {
        Self::new(color.r, color.g, color.b, 255)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn construct() {
        let black8 = RGBA8::new(0, 0, 0, 255);
        let black32 = RGBA8::from(0x000000FF);
        assert!(black8 == black32);
    }
}
