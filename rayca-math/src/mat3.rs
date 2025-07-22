// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::{
    ops::{Index, IndexMut, Mul},
    simd::f32x4,
};

use super::*;

#[repr(C, align(4))]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mat3 {
    /// Row-major
    values: [[f32; 3]; 3],
}

impl From<[[f32; 3]; 3]> for Mat3 {
    fn from(values: [[f32; 3]; 3]) -> Self {
        let mut ret = Self::new();
        ret.values = values;
        ret
    }
}

impl Mat3 {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn get_row(&self, i: usize) -> &[f32; 3] {
        &self.values[i]
    }

    pub fn from_rotation(rotation: &Quat) -> Self {
        let mut ret = Mat3::identity();
        ret.rotate(rotation);
        ret
    }

    pub fn from_scale(scale: &Vec3) -> Self {
        let mut ret = Mat3::identity();
        ret.scale(scale);
        ret
    }

    pub fn identity() -> Self {
        Self {
            values: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        }
    }

    /// Tangent, bitangent, normal
    pub fn tbn(t: &Vec3, b: &Vec3, n: &Vec3) -> Self {
        Self {
            values: [
                [t.simd[0], b.simd[0], n.simd[0]],
                [t.simd[1], b.simd[1], n.simd[1]],
                [t.simd[2], b.simd[2], n.simd[2]],
            ],
        }
    }

    pub fn scale(&mut self, scale: &Vec3) {
        self[0][0] *= scale.get_x();
        self[1][1] *= scale.get_y();
        self[2][2] *= scale.get_z();
    }

    pub fn rotate(&mut self, q: &Quat) {
        *self = Mat3::from(q) * self as &Mat3;
    }

    pub fn get_scale(&self) -> Vec3 {
        Vec3::new(self[0][0], self[1][1], self[2][2])
    }

    pub fn get_rotation(&self) -> Quat {
        Quat::from(self)
    }

    pub fn get_transpose(&self) -> Self {
        let mut ret = Self::new();
        for i in 0..3 {
            for j in 0..3 {
                ret[i][j] = self[j][i]
            }
        }
        ret
    }

    pub fn new_rotation(right: Vec3, up: Vec3, forward: Vec3) -> Self {
        // A rotation matrix actually always defines an orthonormal basis,
        // where each column defines one of the original axes in its rotated state.
        Self {
            values: [
                [right.get_x(), up.get_x(), forward.get_x()],
                [right.get_y(), up.get_y(), forward.get_y()],
                [right.get_z(), up.get_z(), forward.get_z()],
            ],
        }
    }
}

impl From<&Mat4> for Mat3 {
    fn from(mat4: &Mat4) -> Self {
        let mut ret = Self::default();
        for i in 0..3 {
            for j in 0..3 {
                ret[i][j] = mat4.get(i, j)
            }
        }
        ret
    }
}

impl From<Trs> for Mat3 {
    fn from(trs: Trs) -> Self {
        Self::from(&trs)
    }
}

impl From<&Trs> for Mat3 {
    fn from(trs: &Trs) -> Self {
        let mut ret = Mat3::from_scale(&trs.scale);
        ret.rotate(&trs.rotation);
        ret
    }
}

impl From<&Inversed<Trs>> for Mat3 {
    fn from(inv_trs: &Inversed<Trs>) -> Self {
        Mat3::from_scale(&inv_trs.get_scale()) * Mat3::from_rotation(&inv_trs.get_rotation())
    }
}

impl From<&Inversed<&Trs>> for Mat3 {
    fn from(inv_trs: &Inversed<&Trs>) -> Self {
        Mat3::from_scale(&inv_trs.get_scale()) * Mat3::from_rotation(&inv_trs.get_rotation())
    }
}

impl Index<usize> for Mat3 {
    type Output = [f32; 3];
    fn index(&self, i: usize) -> &Self::Output {
        &self.values[i]
    }
}

impl IndexMut<usize> for Mat3 {
    fn index_mut(&mut self, i: usize) -> &mut [f32; 3] {
        &mut self.values[i]
    }
}

impl Mul<&Mat3> for Mat3 {
    type Output = Mat3;

    fn mul(self, rhs: &Mat3) -> Self::Output {
        let mut ret = Mat3::new();

        for i in 0..3 {
            let a = self.values[i][0];
            let b = self.values[i][1];
            let c = self.values[i][2];

            for j in 0..3 {
                let e = a * rhs.values[0][j];
                let f = b * rhs.values[1][j];
                let g = c * rhs.values[2][j];
                ret[i][j] = e + f + g;
            }
        }

        ret
    }
}

impl Mul<Mat3> for Mat3 {
    type Output = Mat3;

    fn mul(self, rhs: Mat3) -> Self::Output {
        self * &rhs
    }
}

impl From<&Quat> for Mat3 {
    fn from(q: &Quat) -> Self {
        let xq = f32x4::splat(q.simd[0]) * q.simd;
        let yq = f32x4::splat(q.simd[1]) * q.simd;
        let zq = f32x4::splat(q.simd[2]) * q.simd;

        Mat3::from([
            [
                1.0 - 2.0 * (yq[1] + zq[2]),
                2.0 * (xq[1] - zq[3]),
                2.0 * (xq[2] + yq[3]),
            ],
            [
                2.0 * (xq[1] + zq[3]),
                1.0 - 2.0 * (xq[0] + zq[2]),
                2.0 * (yq[2] - xq[3]),
            ],
            [
                2.0 * (xq[2] - yq[3]),
                2.0 * (yq[2] + xq[3]),
                1.0 - 2.0 * (xq[0] + yq[1]),
            ],
        ])
    }
}

impl Mul<Vec3> for &Mat3 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        let mut ret = [0.0, 0.0, 0.0];

        for i in 0..3 {
            for j in 0..3 {
                let vv = rhs.simd[j];
                let mv = self[i][j];
                ret[i] += mv * vv;
            }
        }

        Vec3::new(ret[0], ret[1], ret[2])
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn mul() {
        let a = Mat3::identity();
        let mut b = Mat3::identity();
        b.scale(&Vec3::new(2.0, 2.0, 2.0));
        assert!(b.values[0][0] == 2.0);
        assert!(b.values[1][1] == 2.0);
        assert!(b.values[2][2] == 2.0);

        let c = a * b;
        assert!(c != Mat3::identity());
        assert!(c.values[0][0] == 2.0);
        assert!(c.values[1][1] == 2.0);
        assert!(c.values[2][2] == 2.0);
    }

    #[test]
    fn identity_is_identity() {
        let id = Mat3::identity();
        let v = Vec3::new(1.0, 2.0, 3.0);
        let result = &id * v;
        assert_eq!(result, v);
    }

    #[test]
    fn transpose_twice_is_identity() {
        let m = Mat3::from([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]]);
        let t = m.get_transpose();
        let tt = t.get_transpose();
        assert_eq!(m, tt);
    }

    #[test]
    fn scale_and_get_scale() {
        let mut m = Mat3::identity();
        let scale = Vec3::new(3.0, 4.0, 5.0);
        m.scale(&scale);
        assert_eq!(m.get_scale(), scale);
    }

    #[test]
    fn from_and_get_row() {
        let arr = [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]];
        let m = Mat3::from(arr);
        for i in 0..3 {
            assert_eq!(m.get_row(i), &arr[i]);
        }
    }
}
