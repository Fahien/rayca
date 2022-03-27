// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::Vec3;

use std::ops::{Add, Mul};

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn white() -> Self {
        Self::new(1.0, 1.0, 1.0, 1.0)
    }
}

impl From<u32> for Color {
    fn from(color: u32) -> Self {
        Self::new(
            (color >> 24) as u8 as f32 / 255.0,
            (color >> 16) as u8 as f32 / 255.0,
            (color >> 8) as u8 as f32 / 255.0,
            color as u8 as f32 / 255.0,
        )
    }
}

impl From<Vec3> for Color {
    fn from(v: Vec3) -> Self {
        Self::new((v.x + 1.0) / 2.0, (v.y + 1.0) / 2.0, (v.z + 1.0) / 2.0, 1.0)
    }
}

impl From<RGBA8> for Color {
    fn from(color: RGBA8) -> Self {
        Self::new(
            color.r as f32 / 255.0,
            color.g as f32 / 255.0,
            color.b as f32 / 255.0,
            color.a as f32 / 255.0,
        )
    }
}

impl Add for Color {
    type Output = Color;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.r += rhs.r;
        self.g += rhs.g;
        self.b += rhs.b;
        self.a += rhs.a;
        self
    }
}

impl Add for &Color {
    type Output = Color;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output::new(
            self.r + rhs.r,
            self.g + rhs.g,
            self.b + rhs.b,
            self.a + rhs.a,
        )
    }
}

impl Add<Color> for &Color {
    type Output = Color;

    fn add(self, rhs: Color) -> Self::Output {
        Self::Output::new(
            self.r + rhs.r,
            self.g + rhs.g,
            self.b + rhs.b,
            self.a + rhs.a,
        )
    }
}

impl Add<&Color> for Color {
    type Output = Color;

    fn add(self, rhs: &Color) -> Self::Output {
        Self::Output::new(
            self.r + rhs.r,
            self.g + rhs.g,
            self.b + rhs.b,
            self.a + rhs.a,
        )
    }
}

impl Mul<f32> for Color {
    type Output = Color;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::Output::new(self.r * rhs, self.g * rhs, self.b * rhs, self.a)
    }
}

impl Mul<f32> for &Color {
    type Output = Color;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::Output::new(self.r * rhs, self.g * rhs, self.b * rhs, self.a)
    }
}

impl Mul<&Color> for f32 {
    type Output = Color;

    fn mul(self, rhs: &Color) -> Self::Output {
        Self::Output::new(self * rhs.r, self * rhs.g, self * rhs.b, rhs.a)
    }
}

impl Mul<&Color> for &f32 {
    type Output = Color;

    fn mul(self, rhs: &Color) -> Self::Output {
        Self::Output::new(self * rhs.r, self * rhs.g, self * rhs.b, rhs.a)
    }
}

impl Mul<&Color> for &Color {
    type Output = Color;

    fn mul(self, rhs: &Color) -> Self::Output {
        Self::Output::new(
            self.r * rhs.r,
            self.g * rhs.g,
            self.b * rhs.b,
            self.a * rhs.a,
        )
    }
}

impl Mul<&Color> for Color {
    type Output = Color;

    fn mul(self, rhs: &Color) -> Self::Output {
        Self::Output::new(
            self.r * rhs.r,
            self.g * rhs.g,
            self.b * rhs.b,
            self.a * rhs.a,
        )
    }
}

impl Mul<Color> for Color {
    type Output = Color;

    fn mul(self, rhs: Color) -> Self::Output {
        Self::Output::new(
            self.r * rhs.r,
            self.g * rhs.g,
            self.b * rhs.b,
            self.a * rhs.a,
        )
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, Eq, PartialEq)]
/// This is the layout expected by the PNG class
pub struct RGBA8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
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
