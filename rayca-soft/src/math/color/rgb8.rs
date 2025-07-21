// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::{ColorType, ColorTyped, Point3, Vec3};

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, Eq, PartialEq)]
pub struct RGB8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl ColorTyped for RGB8 {
    fn color_type() -> ColorType {
        ColorType::RGB8
    }
}

impl RGB8 {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn white() -> Self {
        Self::new(255, 255, 255)
    }
}

impl From<u32> for RGB8 {
    fn from(color: u32) -> Self {
        Self::new((color >> 16) as u8, (color >> 8) as u8, color as u8)
    }
}

impl From<Vec3> for RGB8 {
    fn from(v: Vec3) -> Self {
        Self::new(
            ((v.get_x() + 1.0) * 127.5) as u8,
            ((v.get_y() + 1.0) * 127.5) as u8,
            ((v.get_z() + 1.0) * 127.5) as u8,
        )
    }
}

impl From<Point3> for RGB8 {
    fn from(v: Point3) -> Self {
        Self::new(
            ((v.get_x() + 1.0) * 127.5) as u8,
            ((v.get_y() + 1.0) * 127.5) as u8,
            ((v.get_z() + 1.0) * 127.5) as u8,
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn construct() {
        let black8 = RGB8::new(0, 0, 0);
        let black32 = RGB8::from(0x000000);
        assert!(black8 == black32);
    }
}
