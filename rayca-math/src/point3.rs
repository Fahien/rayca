// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::{
    ops::{Add, AddAssign, Index, Mul, MulAssign, Sub},
    simd::{StdFloat, f32x4, num::SimdFloat},
};

use crate::{Axis3, Dot, EPS, Quat, Vec3};

#[repr(C, align(16))]
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

    pub fn rotate(&mut self, rotation: Quat) {
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

    pub fn close(&self, b: &Self) -> bool {
        let diff = (self - b).abs();
        diff < Vec3::splat(EPS)
    }
}

impl From<&[f32; 3]> for Point3 {
    fn from(value: &[f32; 3]) -> Self {
        Self::new(value[0], value[1], value[2])
    }
}

impl From<[f32; 3]> for Point3 {
    fn from(value: [f32; 3]) -> Self {
        Self::new(value[0], value[1], value[2])
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new_and_getters() {
        let p = Point3::new(1.0, 2.0, 3.0);
        assert_eq!(p.get_x(), 1.0);
        assert_eq!(p.get_y(), 2.0);
        assert_eq!(p.get_z(), 3.0);
    }

    #[test]
    fn setters() {
        let mut p = Point3::default();
        p.set_x(4.0);
        p.set_y(5.0);
        p.set_z(6.0);
        assert_eq!(p.get_x(), 4.0);
        assert_eq!(p.get_y(), 5.0);
        assert_eq!(p.get_z(), 6.0);
    }

    #[test]
    fn add_and_sub_vec3() {
        let p = Point3::new(1.0, 2.0, 3.0);
        let v = Vec3::new(1.0, 1.0, 1.0);
        let p2 = p + v;
        assert_eq!(p2.get_x(), 2.0);
        assert_eq!(p2.get_y(), 3.0);
        assert_eq!(p2.get_z(), 4.0);
        let p3 = p2 - v;
        assert_eq!(p3, p);
    }

    #[test]
    fn scale_and_mul() {
        let mut p = Point3::new(2.0, 3.0, 4.0);
        let v = Vec3::new(2.0, 3.0, 4.0);
        p.scale(&v);
        assert_eq!(p.get_x(), 4.0);
        assert_eq!(p.get_y(), 9.0);
        assert_eq!(p.get_z(), 16.0);
        let p2 = Point3::new(1.0, 2.0, 3.0) * 2.0;
        assert_eq!(p2.get_x(), 2.0);
        assert_eq!(p2.get_y(), 4.0);
        assert_eq!(p2.get_z(), 6.0);
    }

    #[test]
    fn min_max() {
        let a = Point3::new(1.0, 5.0, 3.0);
        let b = Point3::new(2.0, 4.0, 6.0);
        let min = a.min(&b);
        let max = a.max(&b);
        assert_eq!(min, Point3::new(1.0, 4.0, 3.0));
        assert_eq!(max, Point3::new(2.0, 5.0, 6.0));
    }

    #[test]
    fn index_axis3() {
        let p = Point3::new(7.0, 8.0, 9.0);
        assert_eq!(p[Axis3::X], 7.0);
        assert_eq!(p[Axis3::Y], 8.0);
        assert_eq!(p[Axis3::Z], 9.0);
    }
}
