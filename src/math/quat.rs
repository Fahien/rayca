// Copyright © 2022-2024
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::ops::{Mul, MulAssign};

use crate::*;

/// Quaternion structure
#[derive(Copy, Clone)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quat {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Quat { x, y, z, w }
    }

    /// Standard euclidean for product in 4D
    pub fn dot(&self, rhs: &Quat) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z + self.w * rhs.w
    }

    pub fn len(&self) -> f32 {
        self.dot(self).sqrt()
    }

    pub fn normalize(&mut self) {
        let len = self.len();

        self.x /= len;
        self.y /= len;
        self.z /= len;
        self.w /= len;
    }

    pub fn get_xyz(&self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }

    pub fn get_w(&self) -> f32 {
        self.w
    }

    pub fn is_normalized(&self) -> bool {
        (self.len() - 1.0).abs() < 0.001
    }

    pub fn get_conjugate(&self) -> Self {
        Self::new(-self.x, -self.y, -self.z, self.w)
    }

    pub fn get_inverse(&self) -> Self {
        // The inverse of a unit quaternion is its conjugate
        assert!(self.is_normalized());
        self.get_conjugate()
    }
}

impl Default for Quat {
    fn default() -> Self {
        Quat::new(0.0, 0.0, 0.0, 1.0)
    }
}

impl From<&Mat4> for Quat {
    fn from(matrix: &Mat4) -> Self {
        let mut ret = Quat::default();

        let t = matrix[0][0] + matrix[1][1] + matrix[2][2];
        if t > 0.0 {
            let s = 0.5 / (t + 1.0).sqrt();
            ret.w = 0.25 / s;
            ret.x = (matrix[2][1] - matrix[1][2]) * s;
            ret.y = (matrix[0][2] - matrix[2][0]) * s;
            ret.z = (matrix[1][0] - matrix[0][1]) * s;
        } else if matrix[0][0] > matrix[1][1] && matrix[0][0] > matrix[2][2] {
            let s = 2.0 * (1.0 + matrix[0][0] - matrix[1][1] - matrix[2][2]).sqrt();
            ret.w = (matrix[2][1] - matrix[1][2]) / s;
            ret.x = 0.25 * s;
            ret.y = (matrix[0][1] + matrix[1][0]) / s;
            ret.z = (matrix[0][2] + matrix[2][0]) / s;
        } else if matrix[1][1] > matrix[2][2] {
            let s = 2.0 * (1.0 + matrix[1][1] - matrix[0][0] - matrix[2][2]).sqrt();
            ret.w = (matrix[0][2] - matrix[2][0]) / s;
            ret.x = (matrix[0][1] + matrix[1][0]) / s;
            ret.y = 0.25 * s;
            ret.z = (matrix[1][2] + matrix[2][1]) / s;
        } else {
            let s = 2.0 * (1.0 + matrix[2][2] - matrix[0][0] - matrix[1][1]).sqrt();
            ret.w = (matrix[1][0] - matrix[0][1]) / s;
            ret.x = (matrix[0][2] + matrix[2][0]) / s;
            ret.y = (matrix[1][2] + matrix[2][1]) / s;
            ret.z = 0.25 * s;
        }

        ret.normalize();

        ret
    }
}

impl From<Mat4> for Quat {
    fn from(matrix: Mat4) -> Self {
        Quat::from(&matrix)
    }
}

impl Mul<Quat> for Quat {
    type Output = Quat;

    fn mul(self, rhs: Quat) -> Self::Output {
        Self::new(
            self.x * rhs.w + self.y * rhs.z - self.z * rhs.y + self.w * rhs.x,
            -self.x * rhs.z + self.y * rhs.w + self.z * rhs.x + self.w * rhs.y,
            self.x * rhs.y - self.y * rhs.x + self.z * rhs.w + self.w * rhs.z,
            -self.x * rhs.x - self.y * rhs.y - self.z * rhs.z + self.w * rhs.w,
        )
    }
}

impl MulAssign<Quat> for Quat {
    fn mul_assign(&mut self, rhs: Quat) {
        *self = *self * rhs;
    }
}

impl Mul<Vec3> for Quat {
    type Output = Vec3;

    fn mul(self, mut rhs: Vec3) -> Self::Output {
        rhs.rotate(&self);
        rhs
    }
}

#[cfg(test)]
mod test {
    use std::f32::consts::FRAC_PI_4;

    use super::*;

    #[test]
    fn invert() {
        // Rotation of PI/2 around Y axis (notice we use angle/2.0)
        let a = Quat::new(0.0, FRAC_PI_4.sin(), 0.0, FRAC_PI_4.cos());
        let b = a.get_inverse();
        assert!(a.x == b.x);
        assert!(a.y == -b.y);
        assert!(a.z == b.z);
        assert!(a.w == b.w);
        assert!(b.is_normalized());
    }
}
