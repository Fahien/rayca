// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::ops::{Index, IndexMut, Mul};

use super::*;

#[derive(Default, PartialEq)]
pub struct Mat4 {
    /// Row-major
    values: [[f32; 4]; 4],
}

impl From<[[f32; 4]; 4]> for Mat4 {
    fn from(values: [[f32; 4]; 4]) -> Self {
        let mut ret = Self::new();
        ret.values = values;
        ret
    }
}

impl Mat4 {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn identity() -> Self {
        Self {
            values: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn scale(&mut self, scale: &Vec3) {
        self[0][0] *= scale.x;
        self[1][1] *= scale.y;
        self[2][2] *= scale.z;
    }

    pub fn rotate(&mut self, q: &Quat) {
        *self = Mat4::from(q) * self as &Mat4;
    }

    pub fn translate(&mut self, translation: &Vec3) {
        self[0][3] += translation.x;
        self[1][3] += translation.y;
        self[2][3] += translation.z;
    }

    pub fn get_scale(&self) -> Vec3 {
        Vec3::new(self[0][0], self[1][1], self[2][2])
    }

    pub fn get_rotation(&self) -> Quat {
        Quat::from(self)
    }

    pub fn get_translation(&self) -> Vec3 {
        Vec3::new(self[0][3], self[1][3], self[2][3])
    }
}

impl Index<usize> for Mat4 {
    type Output = [f32; 4];
    fn index<'a>(&'a self, i: usize) -> &'a Self::Output {
        &self.values[i]
    }
}

impl IndexMut<usize> for Mat4 {
    fn index_mut<'a>(&'a mut self, i: usize) -> &'a mut [f32; 4] {
        &mut self.values[i]
    }
}

impl Mul<&Mat4> for Mat4 {
    type Output = Mat4;

    fn mul(self, rhs: &Mat4) -> Self::Output {
        let mut ret = Mat4::new();

        for i in 0..4 {
            let a = self.values[i][0];
            let b = self.values[i][1];
            let c = self.values[i][2];
            let d = self.values[i][3];

            for j in 0..4 {
                let e = a * rhs.values[0][j];
                let f = b * rhs.values[1][j];
                let g = c * rhs.values[2][j];
                let h = d * rhs.values[3][j];
                ret[i][j] = e + f + g + h;
            }
        }

        ret
    }
}

impl Mul<Mat4> for Mat4 {
    type Output = Mat4;

    fn mul(self, rhs: Mat4) -> Self::Output {
        self * &rhs
    }
}

impl From<&Quat> for Mat4 {
    fn from(q: &Quat) -> Self {
        let xx = q.x * q.x;
        let xy = q.x * q.y;
        let xz = q.x * q.z;
        let xw = q.x * q.w;

        let yy = q.y * q.y;
        let yz = q.y * q.z;
        let yw = q.y * q.w;

        let zz = q.z * q.z;
        let zw = q.z * q.w;

        Mat4::from([
            [1.0 - 2.0 * (yy + zz), 2.0 * (xy - zw), 2.0 * (xz + yw), 0.0],
            [2.0 * (xy + zw), 1.0 - 2.0 * (xx + zz), 2.0 * (yz - xw), 0.0],
            [2.0 * (xz - yw), 2.0 * (yz + xw), 1.0 - 2.0 * (xx + yy), 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }
}

impl From<&Trs> for Mat4 {
    fn from(trs: &Trs) -> Self {
        trs * Mat4::identity()
    }
}

impl Mul<Vec3> for &Mat4 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        let mut ret = [0.0, 0.0, 0.0, 0.0];

        for i in 0..4 {
            for j in 0..3 {
                let vv = rhs[j];
                let mv = self[i][j];
                ret[i] += mv * vv;
            }
            let mv = self[i][3];
            ret[i] += mv;
        }

        Vec3::new(ret[0] / ret[3], ret[1] / ret[3], ret[2] / ret[3])
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn mul() {
        // TODO: Further test multiplication
        let a = Mat4::identity();
        let mut b = Mat4::identity();
        b.scale(&Vec3::new(2.0, 2.0, 2.0));
        assert!(b.values[0][0] == 2.0);
        assert!(b.values[1][1] == 2.0);
        assert!(b.values[2][2] == 2.0);

        let c = a * b;
        assert!(c != Mat4::identity());
        assert!(c.values[0][0] == 2.0);
        assert!(c.values[1][1] == 2.0);
        assert!(c.values[2][2] == 2.0);
    }
}