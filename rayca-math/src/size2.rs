// Copyright © 2023-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::ops::Mul;

use crate::Vec2;

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Size2 {
    pub width: u32,
    pub height: u32,
}

impl Size2 {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

impl Mul<u32> for Size2 {
    type Output = Self;

    fn mul(self, rhs: u32) -> Self::Output {
        Self::new(self.width * rhs, self.height * rhs)
    }
}

impl From<Size2> for Vec2 {
    fn from(v: Size2) -> Self {
        Self::new(v.width as f32, v.height as f32)
    }
}

impl std::fmt::Display for Size2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}x{}", self.width, self.height))
    }
}
