// Copyright © 2022-2025
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
        Self::new(
            ((color >> 16) & 0xFF) as u8,
            ((color >> 8) & 0xFF) as u8,
            (color & 0xFF) as u8,
        )
    }
}

impl From<Vec3> for RGB8 {
    fn from(v: Vec3) -> Self {
        Self::new(
            ((v.get_x() + 1.0) * 127.5).clamp(0.0, 255.0) as u8,
            ((v.get_y() + 1.0) * 127.5).clamp(0.0, 255.0) as u8,
            ((v.get_z() + 1.0) * 127.5).clamp(0.0, 255.0) as u8,
        )
    }
}

impl From<Point3> for RGB8 {
    fn from(v: Point3) -> Self {
        Self::new(
            ((v.get_x() + 1.0) * 127.5).clamp(0.0, 255.0) as u8,
            ((v.get_y() + 1.0) * 127.5).clamp(0.0, 255.0) as u8,
            ((v.get_z() + 1.0) * 127.5).clamp(0.0, 255.0) as u8,
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

    #[test]
    fn from_u32() {
        let color = 0x123456;
        let rgb = RGB8::from(color);
        assert_eq!(rgb, RGB8::new(0x12, 0x34, 0x56));
    }

    #[test]
    fn from_vec3_normalized() {
        let v = Vec3::new(1.0, 0.0, -1.0);
        let rgb = RGB8::from(v);
        assert_eq!(rgb, RGB8::new(255, 127, 0));
    }

    #[test]
    fn from_vec3_clamping() {
        let v = Vec3::new(2.0, -2.0, 0.5);
        let rgb = RGB8::from(v);
        assert_eq!(rgb, RGB8::new(255, 0, 191));
    }

    #[test]
    fn from_point3() {
        let p = Point3::new(0.0, 1.0, -1.0);
        let rgb = RGB8::from(p);
        assert_eq!(rgb, RGB8::new(127, 255, 0));
    }
}
