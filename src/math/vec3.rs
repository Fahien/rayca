// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::ops::{Add, Index, Mul, Neg, Sub};

use crate::{Dot, Point3};

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn cross(&self, rhs: &Self) -> Self {
        Self::new(
            self.y * rhs.z - rhs.y * self.z,
            self.z * rhs.x - rhs.z * self.x,
            self.x * rhs.y - rhs.x * self.y,
        )
    }

    pub fn len(&self) -> f32 {
        self.dot(self).sqrt()
    }

    pub fn normalize(&mut self) {
        let len = self.len();
        self.x /= len;
        self.y /= len;
        self.z /= len;
    }
}

impl Dot<Vec3> for Vec3 {
    fn dot(self, rhs: Vec3) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }
}

impl Dot<&Vec3> for Vec3 {
    fn dot(self, rhs: &Vec3) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }
}

impl Dot<Point3> for Vec3 {
    fn dot(self, rhs: Point3) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }
}

impl Add for Vec3 {
    type Output = Vec3;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
        self
    }
}

impl Add for &Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Add<Vec3> for &Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Vec3) -> Self::Output {
        Self::Output::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Add<&Vec3> for Vec3 {
    type Output = Vec3;

    fn add(self, rhs: &Vec3) -> Self::Output {
        Self::Output::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub for Vec3 {
    type Output = Vec3;

    fn sub(mut self, rhs: Self) -> Self::Output {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
        self
    }
}

impl Sub<Point3> for Vec3 {
    type Output = Vec3;

    fn sub(mut self, rhs: Point3) -> Self::Output {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
        self
    }
}

impl Sub<&Vec3> for Vec3 {
    type Output = Vec3;

    fn sub(mut self, rhs: &Vec3) -> Self::Output {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
        self
    }
}

impl Sub for &Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Sub<Vec3> for &Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Vec3) -> Self::Output {
        Self::Output::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Mul<f32> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::Output::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl Mul<f32> for &Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::Output::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl Mul<Vec3> for f32 {
    type Output = Vec3;

    fn mul(self, mut rhs: Vec3) -> Self::Output {
        rhs.x *= self;
        rhs.y *= self;
        rhs.z *= self;
        rhs
    }
}

impl From<Point3> for Vec3 {
    fn from(p: Point3) -> Self {
        Self::new(p.x, p.y, p.z)
    }
}

impl Neg for Vec3 {
    type Output = Vec3;

    fn neg(mut self) -> Self::Output {
        self.x = -self.x;
        self.y = -self.y;
        self.z = -self.z;
        self
    }
}

impl Neg for &Vec3 {
    type Output = Vec3;

    fn neg(self) -> Self::Output {
        Self::Output::new(-self.x, -self.y, -self.z)
    }
}

impl Index<usize> for Vec3 {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("Index out of bounds"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn normalize() {
        let mut v = Vec3::new(2.0, 0.0, 0.0);
        v.normalize();
        assert!(v == Vec3::new(1.0, 0.0, 0.0));
    }
}
