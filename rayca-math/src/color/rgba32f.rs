// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::{Color, ColorType, ColorTyped, RGB8};

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct RGBA32F {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl ColorTyped for RGBA32F {
    fn color_type() -> ColorType {
        ColorType::RGBA32F
    }
}

impl RGBA32F {
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 255.0,
    };

    pub const WHITE: Self = Self {
        r: 255.0,
        g: 255.0,
        b: 255.0,
        a: 255.0,
    };

    pub const RED: Self = Self {
        r: 255.0,
        g: 0.0,
        b: 0.0,
        a: 255.0,
    };

    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn white() -> Self {
        Self::new(255.0, 255.0, 255.0, 255.0)
    }

    pub fn black() -> Self {
        Self::new(0.0, 0.0, 0.0, 255.0)
    }

    /// Over blend this color with another on top of itself
    pub fn over(&mut self, top: Color) {
        let mut self_color: Color = (*self).into();
        self_color.over(top);
        *self = self_color.into();
    }
}

impl From<u32> for RGBA32F {
    fn from(color: u32) -> Self {
        Self::new(
            ((color >> 24) & 0xFF) as f32,
            ((color >> 16) & 0xFF) as f32,
            ((color >> 8) & 0xFF) as f32,
            (color & 0xFF) as f32,
        )
    }
}

impl From<Color> for RGBA32F {
    fn from(color: Color) -> Self {
        Self::new(
            (color.r * 255.0).clamp(0.0, 255.0),
            (color.g * 255.0).clamp(0.0, 255.0),
            (color.b * 255.0).clamp(0.0, 255.0),
            (color.a * 255.0).clamp(0.0, 255.0),
        )
    }
}

impl From<RGB8> for RGBA32F {
    fn from(color: RGB8) -> Self {
        Self::new(color.r as f32, color.g as f32, color.b as f32, 255.0)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn construct() {
        let black32f = RGBA32F::new(0.0, 0.0, 0.0, 255.0);
        let black32 = RGBA32F::from(0x000000FF);
        assert!(black32f == black32);
    }

    #[test]
    fn from_u32() {
        let color = 0x12345678;
        let rgba = RGBA32F::from(color);
        assert_eq!(
            rgba,
            RGBA32F::new(0x12 as f32, 0x34 as f32, 0x56 as f32, 0x78 as f32)
        );
    }

    #[test]
    fn from_color_normalized() {
        let c = Color {
            r: 1.0,
            g: 0.0,
            b: 0.5,
            a: 1.0,
        };
        let rgba = RGBA32F::from(c);
        assert_eq!(rgba, RGBA32F::new(255.0, 0.0, 127.5, 255.0));
    }

    #[test]
    fn from_color_clamping() {
        let c = Color {
            r: 2.0,
            g: -1.0,
            b: 0.5,
            a: 1.5,
        };
        let rgba = RGBA32F::from(c);
        assert_eq!(rgba, RGBA32F::new(255.0, 0.0, 127.5, 255.0));
    }

    #[test]
    fn from_rgb8() {
        let rgb = RGB8::new(10, 20, 30);
        let rgba = RGBA32F::from(rgb);
        assert_eq!(rgba, RGBA32F::new(10.0, 20.0, 30.0, 255.0));
    }
}
