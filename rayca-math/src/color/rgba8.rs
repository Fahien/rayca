// Copyright © 2022-2025
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
    pub const BLACK: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };

    pub const WHITE: Self = Self {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };

    pub const RED: Self = Self {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    };

    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn white() -> Self {
        Self::new(255, 255, 255, 255)
    }

    pub fn black() -> Self {
        Self::new(0, 0, 0, 255)
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
            ((color >> 24) & 0xFF) as u8,
            ((color >> 16) & 0xFF) as u8,
            ((color >> 8) & 0xFF) as u8,
            (color & 0xFF) as u8,
        )
    }
}

impl From<Color> for RGBA8 {
    fn from(color: Color) -> Self {
        Self::new(
            (color.r * 255.0).clamp(0.0, 255.0) as u8,
            (color.g * 255.0).clamp(0.0, 255.0) as u8,
            (color.b * 255.0).clamp(0.0, 255.0) as u8,
            (color.a * 255.0).clamp(0.0, 255.0) as u8,
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

    #[test]
    fn from_u32() {
        let color = 0x12345678;
        let rgba = RGBA8::from(color);
        assert_eq!(rgba, RGBA8::new(0x12, 0x34, 0x56, 0x78));
    }

    #[test]
    fn from_color_normalized() {
        let c = Color {
            r: 1.0,
            g: 0.0,
            b: 0.5,
            a: 1.0,
        };
        let rgba = RGBA8::from(c);
        assert_eq!(rgba, RGBA8::new(255, 0, 127, 255));
    }

    #[test]
    fn from_color_clamping() {
        let c = Color {
            r: 2.0,
            g: -1.0,
            b: 0.5,
            a: 1.5,
        };
        let rgba = RGBA8::from(c);
        assert_eq!(rgba, RGBA8::new(255, 0, 127, 255));
    }

    #[test]
    fn from_rgb8() {
        let rgb = RGB8::new(10, 20, 30);
        let rgba = RGBA8::from(rgb);
        assert_eq!(rgba, RGBA8::new(10, 20, 30, 255));
    }
}
