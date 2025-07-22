// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use core::simd::*;
use std::{
    ops::{Add, AddAssign, Div, Index, Mul, MulAssign, Neg, Sub, SubAssign},
    simd::{StdFloat, num::SimdFloat},
};

use num_traits::MulAdd;
use serde::*;

use crate::{Color, EPS, Point3, Quat};

use crate::Dot;

#[derive(Clone, Copy)]
pub enum Axis3 {
    X,
    Y,
    Z,
}

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Vec3 {
    pub simd: f32x4,
}

impl std::fmt::Display for Vec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[ {}, {}, {} ]",
            self.get_x(),
            self.get_y(),
            self.get_z()
        )
    }
}

impl Vec3 {
    pub const ZERO: Self = Self {
        simd: f32x4::from_array([0.0, 0.0, 0.0, 0.0]),
    };

    pub const ONE: Self = Self {
        simd: f32x4::from_array([1.0, 1.0, 1.0, 0.0]),
    };

    pub const X_AXIS: Self = Self {
        simd: f32x4::from_array([1.0, 0.0, 0.0, 0.0]),
    };

    pub const Y_AXIS: Self = Self {
        simd: f32x4::from_array([0.0, 1.0, 0.0, 0.0]),
    };

    pub const Z_AXIS: Self = Self {
        simd: f32x4::from_array([0.0, 0.0, 1.0, 0.0]),
    };

    pub fn unit() -> Self {
        Self::ONE
    }

    pub fn new(x: f32, y: f32, z: f32) -> Vec3 {
        Self {
            simd: f32x4::from_array([x, y, z, 0.0]),
        }
    }

    pub fn splat(d: f32) -> Vec3 {
        Self::new(d, d, d)
    }

    pub fn simd(mut simd: f32x4) -> Self {
        simd[3] = 0.0;
        Self { simd }
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

    pub fn set_x(&mut self, x: f32) {
        self.simd[0] = x;
    }
    pub fn set_y(&mut self, y: f32) {
        self.simd[1] = y;
    }
    pub fn set_z(&mut self, z: f32) {
        self.simd[2] = z;
    }

    pub fn abs(mut self) -> Vec3 {
        self.simd = self.simd.abs();
        self
    }

    pub fn close(&self, b: &Vec3) -> bool {
        let diff = (self - b).abs();
        diff < Vec3::splat(EPS)
    }

    pub fn min(self, other: &Vec3) -> Vec3 {
        Self::simd(self.simd.simd_min(other.simd))
    }

    pub fn max(self, other: &Vec3) -> Vec3 {
        Self::simd(self.simd.simd_max(other.simd))
    }

    /// [SIMD cross-product](https://geometrian.com/programming/tutorials/cross-product/index.php) method 5
    pub fn cross(&self, rhs: &Self) -> Self {
        let tmp0 = simd_swizzle!(self.simd, [1, 2, 0, 3]);
        let tmp1 = simd_swizzle!(rhs.simd, [2, 0, 1, 3]);
        let tmp2 = tmp0 * rhs.simd;
        let tmp3 = tmp0 * tmp1;
        let tmp4 = simd_swizzle!(tmp2, [1, 2, 0, 3]);
        let res = tmp3 - tmp4;
        Self::simd(res)
    }

    pub fn scale(&mut self, scale: &Vec3) {
        self.simd *= scale.simd;
    }

    pub fn rotate(&mut self, rotation: Quat) {
        // Extract the vector part of the quaternion
        let u = Vec3::simd(rotation.simd * f32x4::from_array([1.0, 1.0, 1.0, 0.0]));

        let v = *self;

        // Extract the scalar part of the quaternion
        let s = rotation.get_w();

        // Do the math
        *self = 2.0 * u.dot(&v) * u + (s * s - u.dot(&u)) * v + 2.0 * s * u.cross(&v);
    }

    pub fn get_rotated(&self, rotation: Quat) -> Self {
        let mut ret = self.clone();
        ret.rotate(rotation);
        ret
    }

    pub fn translate(&mut self, translation: &Vec3) {
        self.simd += translation.simd;
    }

    pub fn norm(&self) -> f32 {
        self.dot(self)
    }

    pub fn len(&self) -> f32 {
        self.norm().sqrt()
    }

    pub fn is_normalized(&self) -> bool {
        (self.norm() - 1.0).abs() < EPS
    }

    pub fn normalize(&mut self) {
        let len = self.len();
        if len > EPS {
            self.simd /= f32x4::from_array([len, len, len, 1.0]);
        }
    }

    pub fn get_normalized(mut self) -> Self {
        self.normalize();
        self
    }

    pub fn get_reciprocal(&self) -> Self {
        static ONE_X: f32x4 = f32x4::from_array([1.0, 0.0, 0.0, 0.0]);
        static ONE_Y: f32x4 = f32x4::from_array([0.0, 1.0, 0.0, 0.0]);
        static ONE_Z: f32x4 = f32x4::from_array([0.0, 0.0, 1.0, 0.0]);
        static ONE_W: f32x4 = f32x4::from_array([0.0, 0.0, 0.0, 1.0]);
        let mut num = f32x4::from_array([1.0, 1.0, 1.0, 0.0]);
        let mut den = self.simd + ONE_W;
        // Avoid division by zero
        if self.simd[0] == 0.0 {
            num -= ONE_X;
            den += ONE_X;
        }
        if self.simd[1] == 0.0 {
            num -= ONE_Y;
            den += ONE_Y;
        }
        if self.simd[2] == 0.0 {
            num -= ONE_Z;
            den += ONE_Z;
        }
        Self::simd(num / den)
    }

    /// Returns the reflection of this vector around a surface normal
    pub fn reflect(&self, normal: &Vec3) -> Self {
        self - 2.0 * self.dot(normal) * normal
    }

    pub fn to_array(self) -> [f32; 3] {
        self.simd.to_array()[0..3].try_into().unwrap()
    }
}

impl Dot<Vec3> for Vec3 {
    fn dot(self, rhs: Vec3) -> f32 {
        self.dot(&rhs)
    }
}

impl Dot<&Vec3> for Vec3 {
    fn dot(self, rhs: &Vec3) -> f32 {
        (self.simd * rhs.simd).reduce_sum()
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(mut self, rhs: Self) -> Self::Output {
        self.simd -= rhs.simd;
        self
    }
}

impl Sub for &Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: &Vec3) -> Self::Output {
        *self - rhs
    }
}

impl Sub<Vec3> for &Vec3 {
    type Output = Vec3;

    fn sub(self, mut rhs: Vec3) -> Self::Output {
        rhs.simd = self.simd - rhs.simd;
        rhs
    }
}

impl Sub<&Vec3> for Vec3 {
    type Output = Vec3;

    fn sub(mut self, rhs: &Vec3) -> Self::Output {
        self.simd -= rhs.simd;
        self
    }
}

impl Sub<f32> for Vec3 {
    type Output = Vec3;

    fn sub(mut self, rhs: f32) -> Self::Output {
        self.simd -= f32x4::splat(rhs);
        self
    }
}

impl SubAssign<&Vec3> for Vec3 {
    fn sub_assign(&mut self, rhs: &Vec3) {
        self.simd -= rhs.simd;
    }
}

impl Default for Vec3 {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
}

impl Add for Vec3 {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.simd += rhs.simd;
        self
    }
}

impl Add<Vec3> for f32 {
    type Output = Vec3;

    fn add(self, mut rhs: Vec3) -> Self::Output {
        rhs += self;
        rhs
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        self.simd += rhs.simd;
    }
}

impl AddAssign<&Vec3> for Vec3 {
    fn add_assign(&mut self, rhs: &Self) {
        self.simd += rhs.simd;
    }
}

impl AddAssign<f32> for Vec3 {
    fn add_assign(&mut self, rhs: f32) {
        self.simd += f32x4::splat(rhs)
    }
}

impl Mul for Vec3 {
    type Output = Self;

    fn mul(mut self, rhs: Self) -> Self::Output {
        self.simd *= rhs.simd;
        self
    }
}

impl Mul<Vec3> for &Vec3 {
    type Output = Vec3;

    fn mul(self, mut rhs: Vec3) -> Self::Output {
        rhs.simd *= self.simd;
        rhs
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;

    fn mul(mut self, rhs: f32) -> Self::Output {
        self.simd *= f32x4::splat(rhs);
        self
    }
}
impl Mul<f32> for &Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: f32) -> Self::Output {
        self * Vec3::splat(rhs)
    }
}

impl Mul<Vec3> for f32 {
    type Output = Vec3;

    fn mul(self, mut rhs: Vec3) -> Self::Output {
        rhs.simd *= f32x4::splat(self);
        rhs
    }
}

impl From<&[f32; 3]> for Vec3 {
    fn from(value: &[f32; 3]) -> Self {
        Self::new(value[0], value[1], value[2])
    }
}

impl From<&Point3> for Vec3 {
    fn from(p: &Point3) -> Self {
        // Set w to 0.0
        Self::simd(p.simd - f32x4::from_array([0.0, 0.0, 0.0, 1.0]))
    }
}

impl From<Point3> for Vec3 {
    fn from(p: Point3) -> Self {
        Self::from(&p)
    }
}

impl From<&Color> for Vec3 {
    fn from(c: &Color) -> Self {
        Vec3::new(c.r * c.a, c.g * c.a, c.b * c.a)
    }
}

impl From<Color> for Vec3 {
    fn from(c: Color) -> Self {
        Self::from(&c)
    }
}

impl Mul<&Vec3> for f32 {
    type Output = Vec3;

    fn mul(self, rhs: &Vec3) -> Self::Output {
        *rhs * self
    }
}

impl MulAssign<f32> for Vec3 {
    fn mul_assign(&mut self, rhs: f32) {
        self.simd *= f32x4::splat(rhs);
    }
}

impl MulAssign<Vec3> for Vec3 {
    fn mul_assign(&mut self, rhs: Vec3) {
        self.simd *= rhs.simd;
    }
}

impl MulAdd for Vec3 {
    type Output = Self;

    fn mul_add(mut self, a: Self, b: Self) -> Self::Output {
        self.simd = self.simd.mul_add(a.simd, b.simd);
        self
    }
}

impl Div<f32> for Vec3 {
    type Output = Vec3;

    fn div(mut self, rhs: f32) -> Self::Output {
        self.simd /= f32x4::splat(rhs);
        self
    }
}

impl Div<Vec3> for f32 {
    type Output = Vec3;

    fn div(self, mut rhs: Vec3) -> Self::Output {
        rhs.simd = f32x4::splat(self) / rhs.simd;
        rhs
    }
}

impl Neg for Vec3 {
    type Output = Vec3;

    fn neg(mut self) -> Self::Output {
        self.simd = -self.simd;
        self
    }
}

impl Neg for &Vec3 {
    type Output = Vec3;

    fn neg(self) -> Self::Output {
        -*self
    }
}

impl Index<Axis3> for Vec3 {
    type Output = f32;

    fn index(&self, index: Axis3) -> &Self::Output {
        match index {
            Axis3::X => &self.simd[0],
            Axis3::Y => &self.simd[1],
            Axis3::Z => &self.simd[2],
        }
    }
}

impl Serialize for Vec3 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_array().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Vec3 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let arr: [f32; 3] = Deserialize::deserialize(deserializer)?;
        Ok(Self::from(&arr))
    }
}

#[cfg(test)]
mod test {
    mod vec3 {
        use crate::*;

        #[test]
        fn normalize() {
            let mut v = Vec3::new(2.0, 0.0, 0.0);
            v.normalize();
            assert!(v.close(&Vec3::new(1.0, 0.0, 0.0)));
        }

        #[test]
        fn rotate() {
            let mut v = Vec3::new(1.0, 0.0, 0.0);
            let y180 = Quat::new(0.0, 1.0, 0.0, 0.0);
            v.rotate(y180);
            assert!(v.close(&Vec3::new(-1.0, 0.0, 0.0)));

            let mut v = Vec3::new(1.0, 0.0, 0.0);
            let y90 = Quat::new(0.0, 0.707, 0.0, 0.707);
            v.rotate(y90);
            assert!(v.close(&Vec3::new(0.0, 0.0, -1.0)));

            let mut v = Vec3::new(1.0, 0.0, 0.0);
            let z180 = Quat::new(0.0, 0.0, 1.0, 0.0);
            v.rotate(z180);
            assert!(v.close(&Vec3::new(-1.0, 0.0, 0.0)));

            let mut v = Vec3::new(1.0, 0.0, 0.0);
            let z90 = Quat::new(0.0, 0.0, 0.707, 0.707);
            v.rotate(z90);
            assert!(v.close(&Vec3::new(0.0, 1.0, 0.0)));

            let mut v = Vec3::new(0.0, 0.0, 1.0);
            // x: -45 degrees
            let rot = Quat::new(-0.383, 0.0, 0.0, 0.924);
            v.rotate(rot);
            assert!(v.close(&Vec3::new(0.0, 0.707, 0.707)));
        }

        #[test]
        fn arithmetic() {
            let a = Vec3::new(1.0, 2.0, 3.0);
            let b = Vec3::new(4.0, 5.0, 6.0);
            assert_eq!(a + b, Vec3::new(5.0, 7.0, 9.0));
            assert_eq!(b - a, Vec3::new(3.0, 3.0, 3.0));
            assert_eq!(a * 2.0, Vec3::new(2.0, 4.0, 6.0));
            assert_eq!(b / 2.0, Vec3::new(2.0, 2.5, 3.0));
            assert_eq!(-a, Vec3::new(-1.0, -2.0, -3.0));
        }

        #[test]
        fn min_max() {
            let a = Vec3::new(1.0, 5.0, 3.0);
            let b = Vec3::new(4.0, 2.0, 6.0);
            assert_eq!(a.min(&b), Vec3::new(1.0, 2.0, 3.0));
            assert_eq!(a.max(&b), Vec3::new(4.0, 5.0, 6.0));
        }

        #[test]
        fn reciprocal() {
            let a = Vec3::new(2.0, 4.0, 0.0);
            let r = a.get_reciprocal();
            assert!((r.get_x() - 0.5).abs() < 1e-6);
            assert!((r.get_y() - 0.25).abs() < 1e-6);
            assert_eq!(r.get_z(), 0.0);
        }

        #[test]
        fn reflect() {
            let v = Vec3::new(1.0, -1.0, 0.0);
            let n = Vec3::new(0.0, 1.0, 0.0);
            let r = v.reflect(&n);
            assert!(r.close(&Vec3::new(1.0, 1.0, 0.0)));
        }

        #[test]
        fn dot_and_cross() {
            let a = Vec3::new(1.0, 0.0, 0.0);
            let b = Vec3::new(0.0, 1.0, 0.0);
            assert_eq!(a.dot(&b), 0.0);
            let c = a.cross(&b);
            assert!(c.close(&Vec3::new(0.0, 0.0, 1.0)));
        }

        #[test]
        fn serde() {
            let v = Vec3::new(1.0, 2.0, 3.0);
            let serialized = serde_json::to_string(&v).unwrap();
            assert_eq!(serialized, "[1.0,2.0,3.0]");
            let deserialized: Vec3 = serde_json::from_str(&serialized).unwrap();
            assert_eq!(v, deserialized);
        }
    }
}
