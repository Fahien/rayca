// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::ops::{Add, Mul};

use crate::Vec3;

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

impl From<Vec3> for RGBA8 {
    fn from(v: Vec3) -> Self {
        Self::new(
            (v.x * 255.0) as u8,
            (v.y * 255.0) as u8,
            (v.z * 255.0) as u8,
            255,
        )
    }
}

impl Add for RGBA8 {
    type Output = RGBA8;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.r += rhs.r;
        self.g += rhs.g;
        self.b += rhs.b;
        self.a += rhs.a;
        self
    }
}

impl Add for &RGBA8 {
    type Output = RGBA8;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output::new(
            self.r + rhs.r,
            self.g + rhs.g,
            self.b + rhs.b,
            self.a + rhs.a,
        )
    }
}

impl Add<RGBA8> for &RGBA8 {
    type Output = RGBA8;

    fn add(self, rhs: RGBA8) -> Self::Output {
        Self::Output::new(
            self.r + rhs.r,
            self.g + rhs.g,
            self.b + rhs.b,
            self.a + rhs.a,
        )
    }
}

impl Add<&RGBA8> for RGBA8 {
    type Output = RGBA8;

    fn add(self, rhs: &RGBA8) -> Self::Output {
        Self::Output::new(
            self.r + rhs.r,
            self.g + rhs.g,
            self.b + rhs.b,
            self.a + rhs.a,
        )
    }
}

impl Mul<f32> for RGBA8 {
    type Output = RGBA8;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::Output::new(
            ((self.r as f32) * rhs) as u8,
            ((self.g as f32) * rhs) as u8,
            ((self.b as f32) * rhs) as u8,
            ((self.a as f32) * rhs) as u8,
        )
    }
}

impl Mul<f32> for &RGBA8 {
    type Output = RGBA8;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::Output::new(
            ((self.r as f32) * rhs) as u8,
            ((self.g as f32) * rhs) as u8,
            ((self.b as f32) * rhs) as u8,
            ((self.a as f32) * rhs) as u8,
        )
    }
}

impl Mul<&RGBA8> for f32 {
    type Output = RGBA8;

    fn mul(self, rhs: &RGBA8) -> Self::Output {
        Self::Output::new(
            (self * rhs.r as f32) as u8,
            (self * rhs.g as f32) as u8,
            (self * rhs.b as f32) as u8,
            (self * rhs.a as f32) as u8,
        )
    }
}

impl Mul<&RGBA8> for &f32 {
    type Output = RGBA8;

    fn mul(self, rhs: &RGBA8) -> Self::Output {
        Self::Output::new(
            (self * rhs.r as f32) as u8,
            (self * rhs.g as f32) as u8,
            (self * rhs.b as f32) as u8,
            (self * rhs.a as f32) as u8,
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
