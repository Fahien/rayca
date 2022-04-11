// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::ops::{Add, Mul};

use crate::{Color, Vec3, ColorType};

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, Eq, PartialEq)]
pub struct RGB8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color for RGB8 {
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
            ((v.x + 1.0) * 127.5) as u8,
            ((v.y + 1.0) * 127.5) as u8,
            ((v.z + 1.0) * 127.5) as u8,
        )
    }
}

impl Add for RGB8 {
    type Output = RGB8;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.r += rhs.r;
        self.g += rhs.g;
        self.b += rhs.b;
        self
    }
}

impl Add for &RGB8 {
    type Output = RGB8;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output::new(self.r + rhs.r, self.g + rhs.g, self.b + rhs.b)
    }
}

impl Add<RGB8> for &RGB8 {
    type Output = RGB8;

    fn add(self, rhs: RGB8) -> Self::Output {
        Self::Output::new(self.r + rhs.r, self.g + rhs.g, self.b + rhs.b)
    }
}

impl Add<&RGB8> for RGB8 {
    type Output = RGB8;

    fn add(self, rhs: &RGB8) -> Self::Output {
        Self::Output::new(self.r + rhs.r, self.g + rhs.g, self.b + rhs.b)
    }
}

impl Mul<f32> for RGB8 {
    type Output = RGB8;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::Output::new(
            ((self.r as f32) * rhs) as u8,
            ((self.g as f32) * rhs) as u8,
            ((self.b as f32) * rhs) as u8,
        )
    }
}

impl Mul<f32> for &RGB8 {
    type Output = RGB8;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::Output::new(
            ((self.r as f32) * rhs) as u8,
            ((self.g as f32) * rhs) as u8,
            ((self.b as f32) * rhs) as u8,
        )
    }
}

impl Mul<&RGB8> for f32 {
    type Output = RGB8;

    fn mul(self, rhs: &RGB8) -> Self::Output {
        Self::Output::new(
            (self * rhs.r as f32) as u8,
            (self * rhs.g as f32) as u8,
            (self * rhs.b as f32) as u8,
        )
    }
}

impl Mul<&RGB8> for &f32 {
    type Output = RGB8;

    fn mul(self, rhs: &RGB8) -> Self::Output {
        Self::Output::new(
            (self * rhs.r as f32) as u8,
            (self * rhs.g as f32) as u8,
            (self * rhs.b as f32) as u8,
        )
    }
}

impl Mul<&RGB8> for &RGB8 {
    type Output = RGB8;

    fn mul(self, rhs: &RGB8) -> Self::Output {
        Self::Output::new(
            (((self.r as f32 / 255.0) * (rhs.r as f32 / 255.0)) * 255.0) as u8,
            (((self.g as f32 / 255.0) * (rhs.g as f32 / 255.0)) * 255.0) as u8,
            (((self.b as f32 / 255.0) * (rhs.b as f32 / 255.0)) * 255.0) as u8,
        )
    }
}

impl Mul<&RGB8> for RGB8 {
    type Output = RGB8;

    fn mul(self, rhs: &RGB8) -> Self::Output {
        Self::Output::new(
            (((self.r as f32 / 255.0) * (rhs.r as f32 / 255.0)) * 255.0) as u8,
            (((self.g as f32 / 255.0) * (rhs.g as f32 / 255.0)) * 255.0) as u8,
            (((self.b as f32 / 255.0) * (rhs.b as f32 / 255.0)) * 255.0) as u8,
        )
    }
}

impl Mul<RGB8> for RGB8 {
    type Output = RGB8;

    fn mul(self, rhs: RGB8) -> Self::Output {
        Self::Output::new(
            (((self.r as f32 / 255.0) * (rhs.r as f32 / 255.0)) * 255.0) as u8,
            (((self.g as f32 / 255.0) * (rhs.g as f32 / 255.0)) * 255.0) as u8,
            (((self.b as f32 / 255.0) * (rhs.b as f32 / 255.0)) * 255.0) as u8,
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
