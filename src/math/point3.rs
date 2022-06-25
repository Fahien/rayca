// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::{
    ops::{Add, AddAssign, Index, Mul, MulAssign, Sub},
    simd::{f32x4, SimdFloat, StdFloat},
};

use crate::{Axis3, Dot, Quat, Vec3};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point3 {
    pub simd: f32x4,
}

impl Default for Point3 {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
}

impl Point3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            simd: f32x4::from_array([x, y, z, 1.0]),
        }
    }

    pub fn simd(simd: f32x4) -> Self {
        Self { simd }
    }

    pub fn set_x(&mut self, x: f32) {
        self.simd[0] = x;
    }

    pub fn set_y(&mut self, y: f32) {
        self.simd[1] = y;
    }

    pub fn set_z(&mut self, z: f32) {
        self.simd[2] = z;
    }

    pub fn get_x(&self) -> f32 {
        self.simd[0]
    }

    pub fn get_y(&self) -> f32 {
        self.simd[1]
    }

    pub fn get_z(&self) -> f32 {
        self.simd[2]
    }

    pub fn scale(&mut self, scale: &Vec3) {
        // Make sure we do not scale w
        static B: f32x4 = f32x4::from_slice(&[0.0, 0.0, 0.0, 1.0]);
        self.simd = self.simd.mul_add(scale.simd, B);
    }

    pub fn rotate(&mut self, rotation: &Quat) {
        // Extract the vector part of the quaternion
        let u = rotation.get_xyz();
        let v = Vec3::from(*self);

        // Extract the scalar part of the quaternion
        let s = rotation.get_w();

        // Do the math
        let rotated = 2.0 * u.dot(v) * u + (s * s - u.dot(u)) * v + 2.0 * s * u.cross(&v);
        *self = rotated.into();
    }

    pub fn translate(&mut self, translation: &Vec3) {
        // `translation.w == 0` hence this is fine
        self.simd += translation.simd;
    }

    pub fn min(self, other: &Self) -> Self {
        Self::simd(self.simd.simd_min(other.simd))
    }

    pub fn max(self, other: &Self) -> Self {
        Self::simd(self.simd.simd_max(other.simd))
    }
}

impl From<Vec3> for Point3 {
    fn from(v: Vec3) -> Self {
        // w should be 1.0
        Self::simd(v.simd + f32x4::from_array([0.0, 0.0, 0.0, 1.0]))
    }
}

impl Dot<Point3> for Point3 {
    fn dot(self, rhs: Self) -> f32 {
        let mul4 = self.simd * rhs.simd;
        mul4.reduce_sum()
    }
}

impl Dot<Vec3> for Point3 {
    fn dot(self, rhs: Vec3) -> f32 {
        let mul4 = self.simd * rhs.simd;
        mul4.reduce_sum()
    }
}

impl Add<Vec3> for Point3 {
    type Output = Point3;

    fn add(self, rhs: Vec3) -> Self::Output {
        // `point.w + vec.w = 1.0` hence that's fine
        Self::Output::simd(self.simd + rhs.simd)
    }
}

impl Sub<&Point3> for Point3 {
    type Output = Vec3;

    fn sub(self, rhs: &Point3) -> Self::Output {
        (&self).sub(rhs)
    }
}

impl Sub<&Point3> for &Point3 {
    type Output = Vec3;

    fn sub(self, rhs: &Point3) -> Self::Output {
        // `point.w - point.w = 0.0` hence a vector
        Self::Output::simd(self.simd - rhs.simd)
    }
}

impl Sub<Point3> for &Point3 {
    type Output = Vec3;

    fn sub(self, rhs: Point3) -> Self::Output {
        // `point.w - point.w = 0.0` hence a vector
        Self::Output::simd(self.simd - rhs.simd)
    }
}

impl Sub for Point3 {
    type Output = Vec3;

    fn sub(self, rhs: Self) -> Self::Output {
        self.sub(&rhs)
    }
}

impl Sub<Vec3> for Point3 {
    type Output = Point3;

    fn sub(self, rhs: Vec3) -> Self::Output {
        self.sub(&rhs)
    }
}

impl Sub<&Vec3> for Point3 {
    type Output = Point3;

    fn sub(self, rhs: &Vec3) -> Self::Output {
        self + (-rhs)
    }
}

impl Sub<Vec3> for &Point3 {
    type Output = Point3;

    fn sub(self, rhs: Vec3) -> Self::Output {
        self.sub(&rhs)
    }
}

impl Sub<&Vec3> for &Point3 {
    type Output = Point3;

    fn sub(self, rhs: &Vec3) -> Self::Output {
        *self + (-rhs)
    }
}

impl Index<usize> for Point3 {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.simd[index]
    }
}

impl AddAssign<Vec3> for Point3 {
    fn add_assign(&mut self, rhs: Vec3) {
        // `point.w + vec.w = 1.0` hence that's fine
        self.simd += rhs.simd
    }
}

impl AddAssign<&Vec3> for Point3 {
    fn add_assign(&mut self, rhs: &Vec3) {
        self.simd += rhs.simd
    }
}

impl Mul<f32> for Point3 {
    type Output = Self;

    fn mul(mut self, rhs: f32) -> Self::Output {
        self.simd
            .mul_assign(f32x4::from_array([rhs, rhs, rhs, 1.0]));
        self
    }
}

impl Mul<f32> for &Point3 {
    type Output = Point3;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::Output::simd(self.simd.mul(f32x4::from_array([rhs, rhs, rhs, 1.0])))
    }
}

impl Mul<Vec3> for Point3 {
    type Output = Self;

    fn mul(mut self, rhs: Vec3) -> Self::Output {
        self.scale(&rhs);
        self
    }
}

impl Index<Axis3> for Point3 {
    type Output = f32;

    fn index(&self, index: Axis3) -> &Self::Output {
        match index {
            Axis3::X => &self.simd[0],
            Axis3::Y => &self.simd[1],
            Axis3::Z => &self.simd[2],
        }
    }
}
