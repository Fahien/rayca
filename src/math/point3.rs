// Copyright © 2022-2024
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::ops::{Add, AddAssign, Index, Sub};

use crate::*;

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Point3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Point3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn scale(&mut self, scale: &Vec3) {
        // Make sure we do not scale w
        self.x *= scale.x;
        self.y *= scale.y;
        self.z *= scale.z;
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
        *self += translation;
    }
}

impl From<Vec3> for Point3 {
    fn from(v: Vec3) -> Self {
        Self::new(v.x, v.y, v.z)
    }
}

impl Dot<Point3> for Point3 {
    fn dot(self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }
}

impl Dot<Vec3> for Point3 {
    fn dot(self, rhs: Vec3) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }
}

impl Add<Vec3> for Point3 {
    type Output = Vec3;

    fn add(self, rhs: Vec3) -> Self::Output {
        Self::Output::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl AddAssign<&Vec3> for Point3 {
    fn add_assign(&mut self, rhs: &Vec3) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Sub for Point3 {
    type Output = Vec3;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Sub<&Point3> for &Point3 {
    type Output = Vec3;

    fn sub(self, rhs: &Point3) -> Self::Output {
        Self::Output::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Index<usize> for Point3 {
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
