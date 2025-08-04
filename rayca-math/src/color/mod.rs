// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign};

use serde::Deserialize;

use crate::{Point3, Vec3};

pub mod rgb8;
pub mod rgba32f;
pub mod rgba8;

pub use rgb8::*;
pub use rgba8::*;
pub use rgba32f::*;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorType {
    RGB8,
    RGBA8,
    RGBA32F,
}

impl ColorType {
    pub fn channels(&self) -> u32 {
        match self {
            ColorType::RGB8 => 3,
            ColorType::RGBA8 => 4,
            ColorType::RGBA32F => 4,
        }
    }

    pub fn depth(&self) -> u32 {
        match self {
            ColorType::RGB8 | ColorType::RGBA8 => std::mem::size_of::<u8>() as u32,
            ColorType::RGBA32F => std::mem::size_of::<f32>() as u32,
        }
    }

    /// Size in bytes of this color type
    pub fn size(&self) -> u32 {
        self.channels() * self.depth()
    }
}

impl Default for ColorType {
    fn default() -> Self {
        Self::RGBA8
    }
}

pub trait ColorTyped: Copy + Default {
    fn color_type() -> ColorType;
}

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[ {}, {}, {}, {} ]", self.r, self.g, self.b, self.a)
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::WHITE
    }
}

impl Color {
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    pub const RED: Self = Self {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    pub const GREEN: Self = Self {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };

    pub const BLUE: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };

    pub const YELLOW: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };

    pub const MAGENTA: Self = Self {
        r: 1.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };

    pub const CYAN: Self = Self {
        r: 0.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_rgb(r: f32, g: f32, b: f32) -> Self {
        Self::new(r, g, b, 1.0)
    }

    pub fn random() -> Self {
        Color::new(fastrand::f32(), fastrand::f32(), fastrand::f32(), 1.0)
    }

    pub fn over(&mut self, top: Color) {
        self.r = top.r * top.a + self.r * (1.0 - top.a);
        self.g = top.g * top.a + self.g * (1.0 - top.a);
        self.b = top.b * top.a + self.b * (1.0 - top.a);
        self.a = 1.0;
    }

    pub fn is_transparent(&self) -> bool {
        self.a < 1.0 - f32::EPSILON
    }

    pub fn close(&self, other: Self) -> bool {
        let diff_r = (self.r - other.r).abs();
        let diff_g = (self.g - other.g).abs();
        let diff_b = (self.b - other.b).abs();
        let diff_a = (self.a - other.a).abs();
        diff_r < f32::EPSILON
            && diff_g < f32::EPSILON
            && diff_b < f32::EPSILON
            && diff_a < f32::EPSILON
    }

    pub fn get_rgb(&self) -> Vec3 {
        Vec3::new(self.r, self.g, self.b)
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
        Self::new(v.simd[0], v.simd[1], v.simd[2], 1.0)
    }
}

impl From<Point3> for Color {
    fn from(v: Point3) -> Self {
        Self::new(v.simd[0], v.simd[1], v.simd[2], 1.0)
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

impl From<RGBA32F> for Color {
    fn from(color: RGBA32F) -> Self {
        Self::new(
            color.r / 255.0,
            color.g / 255.0,
            color.b / 255.0,
            color.a / 255.0,
        )
    }
}

impl From<&[f32; 3]> for Color {
    fn from(value: &[f32; 3]) -> Self {
        Self::new(value[0], value[1], value[2], 1.0)
    }
}

impl Add for Color {
    type Output = Color;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.r += rhs.r * rhs.a;
        self.g += rhs.g * rhs.a;
        self.b += rhs.b * rhs.a;
        self
    }
}

impl Add for &Color {
    type Output = Color;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output::new(
            self.r + rhs.r * rhs.a,
            self.g + rhs.g * rhs.a,
            self.b + rhs.b * rhs.a,
            self.a,
        )
    }
}

impl Add<&Color> for Color {
    type Output = Color;

    fn add(mut self, rhs: &Color) -> Self::Output {
        self.r += rhs.r * rhs.a;
        self.g += rhs.g * rhs.a;
        self.b += rhs.b * rhs.a;
        self
    }
}

impl AddAssign for Color {
    fn add_assign(&mut self, rhs: Color) {
        self.r += rhs.r * rhs.a;
        self.g += rhs.g * rhs.a;
        self.b += rhs.b * rhs.a;
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

impl MulAssign<f32> for Color {
    fn mul_assign(&mut self, rhs: f32) {
        self.r *= rhs;
        self.g *= rhs;
        self.b *= rhs;
    }
}

impl Mul<Color> for f32 {
    type Output = Color;

    fn mul(self, mut rhs: Color) -> Self::Output {
        rhs.r *= self;
        rhs.g *= self;
        rhs.b *= self;
        rhs
    }
}

impl Mul<Color> for &f32 {
    type Output = Color;

    fn mul(self, mut rhs: Color) -> Self::Output {
        rhs.r *= self;
        rhs.g *= self;
        rhs.b *= self;
        rhs
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

impl MulAssign<Color> for Color {
    fn mul_assign(&mut self, rhs: Color) {
        self.r *= rhs.r;
        self.g *= rhs.g;
        self.b *= rhs.b;
        self.a *= rhs.a;
    }
}

impl Div<f32> for Color {
    type Output = Color;

    fn div(self, rhs: f32) -> Self::Output {
        Self::Output::new(self.r / rhs, self.g / rhs, self.b / rhs, self.a)
    }
}

impl Div<f32> for &Color {
    type Output = Color;

    fn div(self, rhs: f32) -> Self::Output {
        Self::Output::new(self.r / rhs, self.g / rhs, self.b / rhs, self.a)
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let arr = <[f32; 4]>::deserialize(deserializer)?;
        Ok(Self::new(arr[0], arr[1], arr[2], arr[3]))
    }

    fn deserialize_in_place<D>(deserializer: D, place: &mut Self) -> Result<(), D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let arr = <[f32; 4]>::deserialize(deserializer)?;
        place.r = arr[0];
        place.g = arr[1];
        place.b = arr[2];
        place.a = arr[3];
        Ok(())
    }
}

impl DivAssign<f32> for Color {
    fn div_assign(&mut self, rhs: f32) {
        self.r /= rhs;
        self.g /= rhs;
        self.b /= rhs;
    }
}
