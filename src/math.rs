// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::ops::Sub;

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn dot(&self, rhs: &Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    pub fn normalize(&mut self) {
        let len = self.dot(self).sqrt();
        self.x /= len;
        self.y /= len;
        self.z /= len;
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
