// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use core::simd::*;
use std::{
    ops::{Add, AddAssign, Div, Index, Mul, MulAssign, Neg, Sub, SubAssign},
    simd::StdFloat,
};

use num_traits::MulAdd;

use crate::{Color, Quat};

#[derive(Clone, Copy)]
pub enum Axis3 {
    X,
    Y,
    Z,
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

const EPS: f32 = f32::EPSILON * 8192.0;

impl Vec3 {
    pub fn iso(d: f32) -> Self {
        Self::new(d, d, d)
    }

    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn close(&self, b: &Vec3) -> bool {
        ((self.x - b.x).abs() < EPS) && ((self.y - b.y).abs() < EPS) && ((self.z - b.z).abs() < EPS)
    }

    pub fn dot(&self, rhs: &Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    pub fn cross(&self, rhs: &Self) -> Self {
        Self::new(
            self.y * rhs.z - rhs.y * self.z,
            self.z * rhs.x - rhs.z * self.x,
            self.x * rhs.y - rhs.x * self.y,
        )
    }

    pub fn norm(&self) -> f32 {
        self.dot(self)
    }

    pub fn len(&self) -> f32 {
        self.norm().sqrt()
    }

    pub fn normalize(&mut self) {
        let len = self.len();
        self.x /= len;
        self.y /= len;
        self.z /= len;
    }

    pub fn get_normalized(&self) -> Self {
        let mut ret = self.clone();
        ret.normalize();
        ret
    }

    pub fn get_reciprocal(&self) -> Self {
        Self::new(1.0 / self.x, 1.0 / self.y, 1.0 / self.z)
    }

    pub fn rotate(&mut self, rotation: &Quat) {
        // Extract the vector part of the quaternion
        let u = Vec3::new(rotation.x, rotation.y, rotation.z);
        let v = self.clone();

        // Extract the scalar part of the quaternion
        let s = rotation.w;

        // Do the math
        *self = 2.0 * u.dot(&v) * u + (s * s - u.dot(&u)) * v + 2.0 * s * u.cross(&v);
    }

    pub fn min(&self, other: &Self) -> Self {
        Self::new(
            self.x.min(other.x),
            self.y.min(other.y),
            self.z.min(other.z),
        )
    }

    pub fn max(&self, other: &Self) -> Self {
        Self::new(
            self.x.max(other.x),
            self.y.max(other.y),
            self.z.max(other.z),
        )
    }

    /// Returns the reflection of this vector around a surface normal
    pub fn reflect(&self, normal: &Vec3) -> Self {
        self - 2.0 * normal.dot(self) * normal
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

    fn add(mut self, rhs: &Vec3) -> Self::Output {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
        self
    }
}

impl Add<f32> for Vec3 {
    type Output = Vec3;

    fn add(mut self, rhs: f32) -> Self::Output {
        self.x += rhs;
        self.y += rhs;
        self.z += rhs;
        self
    }
}

impl Add<Vec3> for f32 {
    type Output = Vec3;

    fn add(self, mut rhs: Vec3) -> Self::Output {
        rhs.x += self;
        rhs.y += self;
        rhs.z += self;
        rhs
    }
}

impl Add<Color> for Vec3 {
    type Output = Vec3;

    fn add(mut self, rhs: Color) -> Self::Output {
        self.x += rhs.r * rhs.a;
        self.y += rhs.g * rhs.a;
        self.z += rhs.b * rhs.a;
        self
    }
}

impl AddAssign<&Vec3> for Vec3 {
    fn add_assign(&mut self, rhs: &Vec3) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
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

impl Mul<&Vec3> for f32 {
    type Output = Vec3;

    fn mul(self, rhs: &Vec3) -> Self::Output {
        Self::Output::new(self * rhs.x, self * rhs.y, self * rhs.z)
    }
}

impl MulAssign<f32> for Vec3 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl Div<f32> for Vec3 {
    type Output = Vec3;

    fn div(self, rhs: f32) -> Self::Output {
        Self::Output::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

impl Div<f32> for &Vec3 {
    type Output = Vec3;

    fn div(self, rhs: f32) -> Self::Output {
        Self::Output::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

impl Mul<Vec3> for f32 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        rhs * self
    }
}

impl Mul<Vec3> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        Self::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
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

impl Index<Axis3> for Vec3 {
    type Output = f32;

    fn index(&self, index: Axis3) -> &Self::Output {
        match index {
            Axis3::X => &self.x,
            Axis3::Y => &self.y,
            Axis3::Z => &self.z,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct FVec3 {
    pub simd: f32x4,
}

impl FVec3 {
    pub fn new(x: f32, y: f32, z: f32) -> FVec3 {
        Self {
            simd: f32x4::from_array([x, y, z, 0.0]),
        }
    }

    pub fn splat(d: f32) -> FVec3 {
        Self::new(d, d, d)
    }

    pub fn abs(&self) -> FVec3 {
        let mut ret = self.clone();
        ret.simd = ret.simd.abs();
        ret
    }

    pub fn close(&self, b: &FVec3) -> bool {
        let diff = (self - b).abs();
        diff < FVec3::splat(EPS)
    }

    pub fn min(self, other: &FVec3) -> FVec3 {
        Self::new(
            self.simd[0].min(other.simd[0]),
            self.simd[1].min(other.simd[1]),
            self.simd[2].min(other.simd[2]),
        )
    }

    pub fn max(self, other: &FVec3) -> FVec3 {
        Self::new(
            self.simd[0].max(other.simd[0]),
            self.simd[1].max(other.simd[1]),
            self.simd[2].max(other.simd[2]),
        )
    }

    pub fn dot(&self, rhs: &Self) -> f32 {
        let m = self.simd * rhs.simd;
        m.reduce_sum()
    }

    pub fn cross(&self, rhs: &Self) -> Self {
        Self::new(
            self.simd[1] * rhs.simd[2] - rhs.simd[1] * self.simd[2],
            self.simd[2] * rhs.simd[0] - rhs.simd[2] * self.simd[0],
            self.simd[0] * rhs.simd[1] - rhs.simd[0] * self.simd[1],
        )
    }

    pub fn rotate(&mut self, rotation: &Quat) {
        // Extract the vector part of the quaternion
        let u = FVec3::new(rotation.x, rotation.y, rotation.z);
        let v = self.clone();

        // Extract the scalar part of the quaternion
        let s = rotation.w;

        // Do the math
        *self = 2.0 * u.dot(&v) * u + (s * s - u.dot(&u)) * v + 2.0 * s * u.cross(&v);
    }

    pub fn norm(&self) -> f32 {
        self.dot(self)
    }

    pub fn len(&self) -> f32 {
        self.norm().sqrt()
    }

    pub fn normalize(&mut self) {
        let len = self.len();
        self.simd /= f32x4::splat(len);
    }

    pub fn get_normalized(&self) -> Self {
        let mut ret = self.clone();
        ret.normalize();
        ret
    }

    /// Returns the reflection of this vector around a surface normal
    pub fn reflect(&self, normal: &FVec3) -> Self {
        self - 2.0 * normal.dot(self) * normal
    }
}

impl From<&Color> for FVec3 {
    fn from(c: &Color) -> Self {
        FVec3::new(c.r * c.a, c.g * c.a, c.b * c.a)
    }
}

impl Sub for FVec3 {
    type Output = Self;

    fn sub(mut self, rhs: Self) -> Self::Output {
        self.simd -= rhs.simd;
        self
    }
}

impl Sub for &FVec3 {
    type Output = FVec3;

    fn sub(self, rhs: &FVec3) -> Self::Output {
        self.clone() - rhs
    }
}

impl Sub<FVec3> for &FVec3 {
    type Output = FVec3;

    fn sub(self, mut rhs: FVec3) -> Self::Output {
        rhs.simd = self.simd - rhs.simd;
        rhs
    }
}

impl Sub<&FVec3> for FVec3 {
    type Output = FVec3;

    fn sub(mut self, rhs: &FVec3) -> Self::Output {
        self.simd -= rhs.simd;
        self
    }
}

impl SubAssign<&FVec3> for FVec3 {
    fn sub_assign(&mut self, rhs: &FVec3) {
        self.simd -= rhs.simd;
    }
}

impl From<&Vec3> for FVec3 {
    fn from(v: &Vec3) -> Self {
        Self::new(v.x, v.y, v.z)
    }
}

impl Default for FVec3 {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
}

impl Add for FVec3 {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.simd += rhs.simd;
        self
    }
}

impl Add<FVec3> for f32 {
    type Output = FVec3;

    fn add(self, mut rhs: FVec3) -> Self::Output {
        rhs += self;
        rhs
    }
}

impl AddAssign for FVec3 {
    fn add_assign(&mut self, rhs: Self) {
        self.simd += rhs.simd;
    }
}

impl AddAssign<f32> for FVec3 {
    fn add_assign(&mut self, rhs: f32) {
        self.simd += f32x4::splat(rhs)
    }
}

impl Mul for FVec3 {
    type Output = Self;

    fn mul(mut self, rhs: Self) -> Self::Output {
        self.simd *= rhs.simd;
        self
    }
}

impl Mul<FVec3> for &FVec3 {
    type Output = FVec3;

    fn mul(self, mut rhs: FVec3) -> Self::Output {
        rhs.simd *= self.simd;
        rhs
    }
}

impl Mul<f32> for FVec3 {
    type Output = Self;

    fn mul(mut self, rhs: f32) -> Self::Output {
        self.simd *= f32x4::splat(rhs);
        self
    }
}
impl Mul<f32> for &FVec3 {
    type Output = FVec3;

    fn mul(self, rhs: f32) -> Self::Output {
        self * FVec3::splat(rhs)
    }
}

impl Mul<FVec3> for f32 {
    type Output = FVec3;

    fn mul(self, mut rhs: FVec3) -> Self::Output {
        rhs.simd *= f32x4::splat(self);
        rhs
    }
}

impl Mul<&FVec3> for f32 {
    type Output = FVec3;

    fn mul(self, rhs: &FVec3) -> Self::Output {
        rhs.clone() * self
    }
}

impl MulAssign<f32> for FVec3 {
    fn mul_assign(&mut self, rhs: f32) {
        self.simd *= f32x4::splat(rhs);
    }
}

impl MulAdd for FVec3 {
    type Output = Self;

    fn mul_add(mut self, a: Self, b: Self) -> Self::Output {
        self.simd = self.simd.mul_add(a.simd, b.simd);
        self
    }
}

impl Div<f32> for FVec3 {
    type Output = FVec3;

    fn div(mut self, rhs: f32) -> Self::Output {
        self.simd /= f32x4::splat(rhs);
        self
    }
}

impl Div<FVec3> for f32 {
    type Output = FVec3;

    fn div(self, mut rhs: FVec3) -> Self::Output {
        rhs.simd = f32x4::splat(self) / rhs.simd;
        rhs
    }
}

impl Neg for FVec3 {
    type Output = FVec3;

    fn neg(mut self) -> Self::Output {
        self.simd = -self.simd;
        self
    }
}

impl Neg for &FVec3 {
    type Output = FVec3;

    fn neg(self) -> Self::Output {
        -self.clone()
    }
}

impl Index<Axis3> for FVec3 {
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

    mod vec3 {
        use crate::Timer;

        use super::*;

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
            v.rotate(&y180);
            assert!(v.close(&Vec3::new(-1.0, 0.0, 0.0)));

            let mut v = Vec3::new(1.0, 0.0, 0.0);
            let y90 = Quat::new(0.0, 0.707, 0.0, 0.707);
            v.rotate(&y90);
            assert!(v.close(&Vec3::new(0.0, 0.0, -1.0)));

            let mut v = Vec3::new(1.0, 0.0, 0.0);
            let z180 = Quat::new(0.0, 0.0, 1.0, 0.0);
            v.rotate(&z180);
            assert!(v.close(&Vec3::new(-1.0, 0.0, 0.0)));

            let mut v = Vec3::new(1.0, 0.0, 0.0);
            let z90 = Quat::new(0.0, 0.0, 0.707, 0.707);
            v.rotate(&z90);
            assert!(v.close(&Vec3::new(0.0, 1.0, 0.0)));

            let mut v = Vec3::new(0.0, 0.0, 1.0);
            // x: -45 degrees
            let rot = Quat::new(-0.383, 0.0, 0.0, 0.924);
            v.rotate(&rot);
            assert!(v.close(&Vec3::new(0.0, 0.707, 0.707)));
        }

        #[test]
        fn bench_simd() {
            let mut timer = Timer::new();

            const ITERATIONS: usize = 1;

            let a = Vec3::new(1.0, 2.0, 3.0);
            let b = Vec3::new(4.0, 5.0, 6.0);
            let mut c = a + b;

            for _ in 0..ITERATIONS {
                c += &b;
            }

            let ns = timer.get_delta().as_nanos();
            println!("Normal: {:?} {}ns", c, ns);

            let _ = timer.get_delta().as_nanos();

            let a = f32x4::from_array([1.0, 2.0, 3.0, 1.0]);
            let b = f32x4::from_array([4.0, 5.0, 6.0, 1.0]);
            let mut c = a + b;

            for _ in 0..ITERATIONS {
                c += b;
            }

            let ns = timer.get_delta().as_nanos();
            println!("SIMD: {:?} {}ns", c, ns);
        }
    }

    mod fvec3 {
        use super::FVec3;
        use crate::Quat;

        #[test]
        fn normalize() {
            let mut v = FVec3::new(2.0, 0.0, 0.0);
            v.normalize();
            assert!(v.close(&FVec3::new(1.0, 0.0, 0.0)));
        }

        #[test]
        fn rotate() {
            let mut v = FVec3::new(1.0, 0.0, 0.0);
            let y180 = Quat::new(0.0, 1.0, 0.0, 0.0);
            v.rotate(&y180);
            assert!(v.close(&FVec3::new(-1.0, 0.0, 0.0)));

            let mut v = FVec3::new(1.0, 0.0, 0.0);
            let y90 = Quat::new(0.0, 0.707, 0.0, 0.707);
            v.rotate(&y90);
            assert!(v.close(&FVec3::new(0.0, 0.0, -1.0)));

            let mut v = FVec3::new(1.0, 0.0, 0.0);
            let z180 = Quat::new(0.0, 0.0, 1.0, 0.0);
            v.rotate(&z180);
            assert!(v.close(&FVec3::new(-1.0, 0.0, 0.0)));

            let mut v = FVec3::new(1.0, 0.0, 0.0);
            let z90 = Quat::new(0.0, 0.0, 0.707, 0.707);
            v.rotate(&z90);
            assert!(v.close(&FVec3::new(0.0, 1.0, 0.0)));

            let mut v = FVec3::new(0.0, 0.0, 1.0);
            // x: -45 degrees
            let rot = Quat::new(-0.383, 0.0, 0.0, 0.924);
            v.rotate(&rot);
            assert!(v.close(&FVec3::new(0.0, 0.707, 0.707)));
        }
    }
}
