// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, Eq, PartialEq)]
/// This is the layout expected by the PNG class
pub struct RGBA8 {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl RGBA8 {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
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
