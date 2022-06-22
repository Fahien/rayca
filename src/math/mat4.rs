// Copyright © 2022-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::ops::{Index, IndexMut, Mul};

use super::*;

#[derive(Clone, Default, PartialEq)]
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

    pub fn scale(&mut self, scale: &Vec3) {
        self[0][0] *= scale.get_x();
        self[1][1] *= scale.get_y();
        self[2][2] *= scale.get_z();
    }

    pub fn rotate(&mut self, q: &Quat) {
        *self = Mat4::from(q) * self as &Mat4;
    }

    pub fn translate(&mut self, translation: &Vec3) {
        self[0][3] += translation.get_x();
        self[1][3] += translation.get_y();
        self[2][3] += translation.get_z();
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

    pub fn get_transpose(&self) -> Self {
        let mut ret = Self::new();
        for i in 0..4 {
            for j in 0..4 {
                ret[i][j] = self[j][i]
            }
        }
        ret
    }
}

impl Index<usize> for Mat4 {
    type Output = [f32; 4];
    fn index(&self, i: usize) -> &Self::Output {
        &self.values[i]
    }
}

impl IndexMut<usize> for Mat4 {
    fn index_mut(&mut self, i: usize) -> &mut [f32; 4] {
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
        let xx = q.get_x() * q.get_x();
        let xy = q.get_x() * q.get_y();
        let xz = q.get_x() * q.get_z();
        let xw = q.get_x() * q.get_w();

        let yy = q.get_y() * q.get_y();
        let yz = q.get_y() * q.get_z();
        let yw = q.get_y() * q.get_w();

        let zz = q.get_z() * q.get_z();
        let zw = q.get_z() * q.get_w();

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

                for i in 0..4 {
                    for j in 0..3 {
                        let vv = rhs.simd[j];
                        let mv = self[i][j];
                        ret[i] += mv * vv;
                    }
                    let mv = self[i][3];
                    ret[i] += mv;
                }

                let den = if ret[3] != 0.0 { ret[3] } else { 1.0 };
                <$T3>::new(ret[0] / den, ret[1] / den, ret[2] / den)
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
