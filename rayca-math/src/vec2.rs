// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::ops::{Add, Div, Mul, Neg, Sub};

use crate::{EPS, Vec3};

#[repr(C, align(8))]
#[derive(Clone, Copy, Default, Debug, PartialEq, PartialOrd)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub fn new(x: f32, y: f32) -> Vec2 {
        Vec2 { x, y }
    }

    pub fn min(&self, other: &Self) -> Self {
        Self::new(self.x.min(other.x), self.y.min(other.y))
    }

    pub fn max(&self, other: &Self) -> Self {
        Self::new(self.x.max(other.x), self.y.max(other.y))
    }

    /// Returns true if all components are within `eps` of the other Vec2
    pub fn close(&self, other: &Self) -> bool {
        (self.x - other.x).abs() < EPS && (self.y - other.y).abs() < EPS
    }

    pub fn extend(self, z: f32) -> Vec3 {
        Vec3::new(self.x, self.y, z)
    }
}

impl Add for Vec2 {
    type Output = Vec2;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.x += rhs.x;
        self.y += rhs.y;
        self
    }
}

impl Sub for Vec2 {
    type Output = Vec2;

    fn sub(mut self, rhs: Self) -> Self::Output {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self
    }
}

impl Mul<f32> for Vec2 {
    type Output = Vec2;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::Output::new(self.x * rhs, self.y * rhs)
    }
}

impl Mul<f32> for &Vec2 {
    type Output = Vec2;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::Output::new(self.x * rhs, self.y * rhs)
    }
}

impl Div<f32> for Vec2 {
    type Output = Vec2;

    fn div(self, rhs: f32) -> Self::Output {
        Vec2::new(self.x / rhs, self.y / rhs)
    }
}

impl Mul for Vec2 {
    type Output = Vec2;

    fn mul(self, rhs: Self) -> Self::Output {
        Vec2::new(self.x * rhs.x, self.y * rhs.y)
    }
}

impl Mul<&Vec2> for Vec2 {
    type Output = Vec2;

    fn mul(self, rhs: &Vec2) -> Self::Output {
        Vec2::new(self.x * rhs.x, self.y * rhs.y)
    }
}

impl Mul<Vec2> for &Vec2 {
    type Output = Vec2;

    fn mul(self, rhs: Vec2) -> Self::Output {
        Vec2::new(self.x * rhs.x, self.y * rhs.y)
    }
}

impl Mul<&Vec2> for &Vec2 {
    type Output = Vec2;

    fn mul(self, rhs: &Vec2) -> Self::Output {
        Vec2::new(self.x * rhs.x, self.y * rhs.y)
    }
}

impl Add<&Vec2> for Vec2 {
    type Output = Vec2;
    fn add(self, rhs: &Vec2) -> Self::Output {
        Vec2::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Add<Vec2> for &Vec2 {
    type Output = Vec2;
    fn add(self, rhs: Vec2) -> Self::Output {
        Vec2::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Add<&Vec2> for &Vec2 {
    type Output = Vec2;
    fn add(self, rhs: &Vec2) -> Self::Output {
        Vec2::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub<&Vec2> for Vec2 {
    type Output = Vec2;
    fn sub(self, rhs: &Vec2) -> Self::Output {
        Vec2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Sub<Vec2> for &Vec2 {
    type Output = Vec2;
    fn sub(self, rhs: Vec2) -> Self::Output {
        Vec2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Sub<&Vec2> for &Vec2 {
    type Output = Vec2;
    fn sub(self, rhs: &Vec2) -> Self::Output {
        Vec2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Neg for Vec2 {
    type Output = Vec2;
    fn neg(self) -> Self::Output {
        Vec2::new(-self.x, -self.y)
    }
}

impl Neg for &Vec2 {
    type Output = Vec2;
    fn neg(self) -> Self::Output {
        Vec2::new(-self.x, -self.y)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_add() {
        let a = Vec2::new(1.0, 2.0);
        let b = Vec2::new(3.0, 4.0);
        assert_eq!(a + b, Vec2::new(4.0, 6.0));
    }

    #[test]
    fn test_sub() {
        let a = Vec2::new(5.0, 7.0);
        let b = Vec2::new(2.0, 3.0);
        assert_eq!(a - b, Vec2::new(3.0, 4.0));
    }

    #[test]
    fn test_mul_scalar() {
        let a = Vec2::new(2.0, 3.0);
        assert_eq!(a * 2.0, Vec2::new(4.0, 6.0));
    }

    #[test]
    fn test_mul_elementwise() {
        let a = Vec2::new(2.0, 3.0);
        let b = Vec2::new(4.0, 5.0);
        assert_eq!(a * b, Vec2::new(8.0, 15.0));
    }

    #[test]
    fn test_div_scalar() {
        let a = Vec2::new(8.0, 6.0);
        assert_eq!(a / 2.0, Vec2::new(4.0, 3.0));
    }

    #[test]
    fn test_neg() {
        let a = Vec2::new(1.0, -2.0);
        assert_eq!(-a, Vec2::new(-1.0, 2.0));
    }

    #[test]
    fn test_min_max() {
        let a = Vec2::new(1.0, 5.0);
        let b = Vec2::new(3.0, 2.0);
        assert_eq!(a.min(&b), Vec2::new(1.0, 2.0));
        assert_eq!(a.max(&b), Vec2::new(3.0, 5.0));
    }

    #[test]
    fn test_close() {
        let a = Vec2::new(1.00001, 2.00001);
        let b = Vec2::new(1.00002, 2.00002);
        assert!(a.close(&b));
    }
}
