// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::{
    ops::{Index, IndexMut, Mul},
    simd::{f32x4, f32x16, mask32x16, num::SimdFloat, simd_swizzle},
};

use super::*;

#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq)]
/// Row-major 4x4 Matrix
pub struct Mat4 {
    values: f32x16,
}

impl From<[[f32; 4]; 4]> for Mat4 {
    fn from(values: [[f32; 4]; 4]) -> Self {
        let slice: &[f32; 16] = unsafe { std::mem::transmute(&values) };
        Self::new(f32x16::from_slice(slice))
    }
}

impl From<[f32x4; 4]> for Mat4 {
    fn from(values: [f32x4; 4]) -> Self {
        Self::new(unsafe { std::mem::transmute::<[f32x4; 4], f32x16>(values) })
    }
}

impl Mat4 {
    pub fn new(values: f32x16) -> Self {
        Self { values }
    }

    pub fn new_with_rows(row0: [f32; 4], row1: [f32; 4], row2: [f32; 4], row3: [f32; 4]) -> Self {
        Self::from([row0, row1, row2, row3])
    }

    pub fn identity() -> Self {
        Self::new(f32x16::from_array([
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ]))
    }

    pub fn as_slices(&self) -> &[[f32; 4]; 4] {
        unsafe { std::mem::transmute(&self.values) }
    }

    pub fn as_slice(&self) -> &[f32; 16] {
        unsafe { std::mem::transmute(&self.values) }
    }

    pub fn get(&self, row: usize, col: usize) -> f32 {
        self.values[row * 4 + col]
    }

    pub fn set(&mut self, row: usize, col: usize, val: f32) {
        self.values[row * 4 + col] = val;
    }

    pub fn from_translation(translation: &Vec3) -> Self {
        let mut ret = Mat4::identity();
        ret.translate(translation);
        ret
    }

    pub fn from_rotation(rotation: &Quat) -> Self {
        let mut ret = Mat4::identity();
        ret.rotate(rotation);
        ret
    }

    pub fn from_scale(scale: &Vec3) -> Self {
        let mut ret = Mat4::identity();
        ret.scale(scale);
        ret
    }

    pub fn look_at(target: Vec3, eye: Vec3, up: Vec3) -> Self {
        // Z axis points towards the eye!
        let z_axis = (eye - target).get_normalized();
        let x_axis = up.cross(&z_axis).get_normalized();
        let y_axis = z_axis.cross(&x_axis);

        let values = [
            x_axis.simd + f32x4::from_array([0.0, 0.0, 0.0, x_axis.dot(&-eye)]),
            y_axis.simd + f32x4::from_array([0.0, 0.0, 0.0, y_axis.dot(&-eye)]),
            z_axis.simd + f32x4::from_array([0.0, 0.0, 0.0, z_axis.dot(&-eye)]),
            f32x4::from_array([0.0, 0.0, 0.0, 1.0]),
        ];

        Self::from(values)
    }

    pub fn scale(&mut self, scale: &Vec3) {
        self[0] *= scale.get_x();
        self[5] *= scale.get_y();
        self[10] *= scale.get_z();
    }

    pub fn rotate(&mut self, q: &Quat) {
        *self = Mat4::from(q) * self as &Mat4;
    }

    pub fn translate(&mut self, translation: &Vec3) {
        self[3] += translation.get_x();
        self[7] += translation.get_y();
        self[11] += translation.get_z();
    }

    pub fn get_scale(&self) -> Vec3 {
        Vec3::new(self[0], self[5], self[10])
    }

    pub fn get_rotation(&self) -> Quat {
        Quat::from(self)
    }

    pub fn get_translation(&self) -> Vec3 {
        Vec3::new(self[3], self[7], self[11])
    }

    pub fn set_translation(&mut self, translation: &Vec3) {
        self[3] = translation.get_x();
        self[7] = translation.get_y();
        self[11] = translation.get_z();
    }

    pub fn get_transpose(&self) -> Self {
        let ret = simd_swizzle!(
            self.values,
            [0, 4, 8, 12, 1, 5, 9, 13, 2, 6, 10, 14, 3, 7, 11, 15]
        );
        Self::new(ret)
    }
}

impl Index<usize> for Mat4 {
    type Output = f32;
    fn index(&self, i: usize) -> &Self::Output {
        &self.values[i]
    }
}

impl IndexMut<usize> for Mat4 {
    fn index_mut(&mut self, i: usize) -> &mut f32 {
        &mut self.values[i]
    }
}

impl Index<std::ops::Range<usize>> for Mat4 {
    type Output = [f32];
    fn index(&self, index: std::ops::Range<usize>) -> &Self::Output {
        &self.as_slice()[index]
    }
}

impl Mul<&Mat4> for Mat4 {
    type Output = Mat4;

    fn mul(self, rhs: &Mat4) -> Self::Output {
        let mut ret = [[0.0; 4]; 4];
        static ZEROS: f32x16 = f32x16::from_array([0.0; 16]);
        let rhs = rhs.get_transpose().values;
        let mut rows = self.values;
        let mask = mask32x16::from_bitmask(15);

        ret.iter_mut().for_each(|ret_i| {
            let row = mask.select(rows, ZEROS);

            let mut columns = rhs;

            ret_i.iter_mut().for_each(|ret_i_j| {
                let column = mask.select(columns, ZEROS);
                let num = (row * column).reduce_sum();
                *ret_i_j = num;

                columns = columns.rotate_elements_left::<4>();
            });

            // Prepare for second row to appear at the beginning
            rows = rows.rotate_elements_left::<4>();
        });

        Self::Output::from(ret)
    }
}

impl Mul<Mat4> for Mat4 {
    type Output = Mat4;

    fn mul(self, rhs: Mat4) -> Self::Output {
        self * &rhs
    }
}

impl Mul<Ray> for &Mat4 {
    type Output = Ray;

    fn mul(self, rhs: Ray) -> Self::Output {
        Ray::new(self * rhs.origin, self * rhs.dir)
    }
}

impl From<&Quat> for Mat4 {
    fn from(q: &Quat) -> Self {
        let xq = f32x4::splat(q.simd[0]) * q.simd;
        let yq = f32x4::splat(q.simd[1]) * q.simd;
        let zq = f32x4::splat(q.simd[2]) * q.simd;

        Mat4::from([
            [
                1.0 - 2.0 * (yq[1] + zq[2]),
                2.0 * (xq[1] - zq[3]),
                2.0 * (xq[2] + yq[3]),
                0.0,
            ],
            [
                2.0 * (xq[1] + zq[3]),
                1.0 - 2.0 * (xq[0] + zq[2]),
                2.0 * (yq[2] - xq[3]),
                0.0,
            ],
            [
                2.0 * (xq[2] - yq[3]),
                2.0 * (yq[2] + xq[3]),
                1.0 - 2.0 * (xq[0] + yq[1]),
                0.0,
            ],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }
}

impl From<Mat3> for Mat4 {
    fn from(mat3: Mat3) -> Self {
        let mr0 = mat3.get_row(0);
        let row0 = [mr0[0], mr0[1], mr0[2], 0.0];
        let mr1 = mat3.get_row(1);
        let row1 = [mr1[0], mr1[1], mr1[2], 0.0];
        let mr2 = mat3.get_row(2);
        let row2 = [mr2[0], mr2[1], mr2[2], 0.0];
        let row3 = [0.0, 0.0, 0.0, 1.0];
        Mat4::from([row0, row1, row2, row3])
    }
}

impl From<&Trs> for Mat4 {
    fn from(trs: &Trs) -> Self {
        Mat4::from_translation(&trs.translation)
            * (Mat4::from_rotation(&trs.rotation) * Mat4::from_scale(&trs.scale))
    }
}

impl From<Trs> for Mat4 {
    fn from(trs: Trs) -> Self {
        Self::from(&trs)
    }
}

impl From<&Inversed<&Trs>> for Mat4 {
    fn from(inv_trs: &Inversed<&Trs>) -> Self {
        Mat4::from_scale(&inv_trs.get_scale())
            * (Mat4::from_rotation(&inv_trs.get_rotation())
                * Mat4::from_translation(&inv_trs.get_translation()))
    }
}

impl From<Inversed<&Trs>> for Mat4 {
    fn from(inv_trs: Inversed<&Trs>) -> Self {
        Self::from(&inv_trs)
    }
}

impl From<&Inversed<Trs>> for Mat4 {
    fn from(inv_trs: &Inversed<Trs>) -> Self {
        Mat4::from_scale(&inv_trs.get_scale())
            * (Mat4::from_rotation(&inv_trs.get_rotation())
                * Mat4::from_translation(&inv_trs.get_translation()))
    }
}

impl From<Inversed<Trs>> for Mat4 {
    fn from(inv_trs: Inversed<Trs>) -> Self {
        Self::from(&inv_trs)
    }
}

macro_rules! impl_mul3 {
    ($T3: ty, $M4: ty) => {
        impl Mul<$T3> for $M4 {
            type Output = $T3;

            fn mul(self, rhs: $T3) -> Self::Output {
                let mut ret = [0.0, 0.0, 0.0, 0.0];
                let mut rows = self.values;

                for i in 0..4 {
                    let row: &f32x4 = unsafe { std::mem::transmute(&rows) };
                    // Use last element
                    let column = &rhs.simd;
                    ret[i] = (row * column).reduce_sum();

                    rows = rows.rotate_elements_left::<4>();
                }

                let den = if ret[3] != 0.0 { ret[3] } else { 1.0 };
                Self::Output::new(ret[0] / den, ret[1] / den, ret[2] / den)
            }
        }
    };
}

impl_mul3!(Point3, Mat4);
impl_mul3!(Vec3, Mat4);
impl_mul3!(Point3, &Mat4);
impl_mul3!(Vec3, &Mat4);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn mul() {
        let a = Mat4::identity();
        let mut b = Mat4::identity();
        b.scale(&Vec3::new(2.0, 2.0, 2.0));
        assert_eq!(b[0], 2.0);
        assert_eq!(b[5], 2.0);
        assert_eq!(b[10], 2.0);

        let c = a * b;
        assert_ne!(c, Mat4::identity());
        assert_eq!(c[0], 2.0);
        assert_eq!(c[5], 2.0);
        assert_eq!(c[10], 2.0);
    }

    #[test]
    fn look_at() {
        let target = Vec3::default();
        let eye = Vec3::new(0.0, 0.0, 4.0);
        let up = Vec3::new(0.0, 1.0, 0.0);
        let mat = Mat4::look_at(target, eye, up);
        assert_eq!(-mat.get_translation(), eye);
        assert_eq!(mat.get_rotation(), Quat::default());
    }

    #[test]
    fn transmute() {
        let m = Mat4::from([
            f32x4::from_array([0.0, 1.0, 2.0, 3.0]),
            f32x4::from_array([4.0, 5.0, 6.0, 7.0]),
            f32x4::from_array([8.0, 9.0, 10.0, 11.0]),
            f32x4::from_array([12.0, 13.0, 14.0, 15.0]),
        ]);

        for i in 0..16 {
            assert_eq!(m[i], i as f32);
        }
    }

    #[test]
    fn transpose() {
        let m = Mat4::from([
            f32x4::from_array([0.0, 4.0, 8.0, 12.0]),
            f32x4::from_array([1.0, 5.0, 9.0, 13.0]),
            f32x4::from_array([2.0, 6.0, 10.0, 14.0]),
            f32x4::from_array([3.0, 7.0, 11.0, 15.0]),
        ])
        .get_transpose();

        for i in 0..16 {
            assert_eq!(m[i], i as f32);
        }
    }

    #[test]
    fn identity_is_identity() {
        let id = Mat4::identity();
        let v = Vec3::new(1.0, 2.0, 3.0);
        let result = &id * v;
        assert_eq!(result, v);
    }

    #[test]
    fn scale_and_get_scale() {
        let mut m = Mat4::identity();
        let scale = Vec3::new(3.0, 4.0, 5.0);
        m.scale(&scale);
        assert_eq!(m.get_scale(), scale);
    }

    #[test]
    fn translation_and_get_translation() {
        let mut m = Mat4::identity();
        let t = Vec3::new(7.0, 8.0, 9.0);
        m.translate(&t);
        assert_eq!(m.get_translation(), t);
    }

    #[test]
    fn set_and_get() {
        let mut m = Mat4::identity();
        m.set(2, 3, 42.0);
        assert_eq!(m.get(2, 3), 42.0);
    }

    #[test]
    fn from_and_as_slices() {
        let arr = [
            [1.0, 2.0, 3.0, 4.0],
            [5.0, 6.0, 7.0, 8.0],
            [9.0, 10.0, 11.0, 12.0],
            [13.0, 14.0, 15.0, 16.0],
        ];
        let m = Mat4::from(arr);
        assert_eq!(m.as_slices(), &arr);
    }
}
