// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

pub mod rgb8;
pub mod rgba8;

pub use rgb8::*;
pub use rgba8::*;

use crate::Vec3;

use std::ops::{Add, AddAssign, Div, Mul};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ColorType {
    RGB8,
    RGBA8,
}

impl ColorType {
    pub fn channels(&self) -> usize {
        match self {
            ColorType::RGB8 => 3,
            ColorType::RGBA8 => 4,
        }
    }
}

impl Default for ColorType {
    fn default() -> Self {
        Self::RGBA8
    }
}

pub trait ColorTyped: Copy {
    fn color_type() -> ColorType;
}

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

    pub fn black() -> Self {
        Self::new(0.0, 0.0, 0.0, 1.0)
    }

    pub fn white() -> Self {
        Self::new(1.0, 1.0, 1.0, 1.0)
    }

    pub fn over(&mut self, top: Color) {
        self.r = top.r * top.a + self.r * (1.0 - top.a);
        self.g = top.g * top.a + self.g * (1.0 - top.a);
        self.b = top.b * top.a + self.b * (1.0 - top.a);
        self.a = 1.0;
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

impl AddAssign<Color> for Color {
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

impl Mul<&Vec3> for &Color {
    type Output = Color;

    fn mul(self, rhs: &Vec3) -> Self::Output {
        Self::Output::new(self.r * rhs.x, self.g * rhs.y, self.b * rhs.z, self.a)
    }
}

impl Mul<&Vec3> for Color {
    type Output = Color;

    fn mul(self, rhs: &Vec3) -> Self::Output {
        Self::Output::new(self.r * rhs.x, self.g * rhs.y, self.b * rhs.z, self.a)
    }
}

impl Mul<Vec3> for Color {
    type Output = Color;

    fn mul(self, rhs: Vec3) -> Self::Output {
        Self::Output::new(self.r * rhs.x, self.g * rhs.y, self.b * rhs.z, self.a)
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
