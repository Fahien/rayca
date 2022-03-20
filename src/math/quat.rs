// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::Mat4;

/// Quaternion structure
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
