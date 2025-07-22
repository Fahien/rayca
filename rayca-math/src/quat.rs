// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::{
    ops::{Mul, MulAssign},
    simd::{f32x4, num::SimdFloat},
};

use serde::{Deserialize, Serialize};

use crate::*;

/// Quaternion structure
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Quat {
    pub simd: f32x4,
}

impl std::fmt::Display for Quat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[ {}, {}, {}, {} ]",
            self.get_x(),
            self.get_y(),
            self.get_z(),
            self.get_w()
        )
    }
}

impl Quat {
    pub const IDENTITY: Quat = Quat {
        simd: f32x4::from_array([0.0, 0.0, 0.0, 1.0]),
    };

    pub fn simd(simd: f32x4) -> Self {
        Self { simd }
    }

    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Quat {
            simd: f32x4::from_array([x, y, z, w]),
        }
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
    pub fn get_w(&self) -> f32 {
        self.simd[3]
    }

    pub fn get_xyz(&self) -> Vec3 {
        static XYZ: f32x4 = f32x4::from_array([1.0, 1.0, 1.0, 0.0]);
        Vec3::simd(self.simd * XYZ)
    }

    pub fn axis_angle(axis: Vec3, angle_radians: f32) -> Self {
        let factor = (angle_radians / 2.0).sin();

        let simd = axis.simd * f32x4::splat(factor)
            + f32x4::from_array([0.0, 0.0, 0.0, (angle_radians / 2.0).cos()]);

        let mut ret = Quat::simd(simd);
        ret.normalize();
        ret
    }

    /// Standard euclidean for product in 4D
    pub fn dot(&self, rhs: &Quat) -> f32 {
        (self.simd * rhs.simd).reduce_sum()
    }

    pub fn len(&self) -> f32 {
        self.dot(self).sqrt()
    }

    pub fn normalize(&mut self) {
        let len = self.len();
        self.simd /= f32x4::splat(len);
    }

    pub fn is_normalized(&self) -> bool {
        (self.len() - 1.0).abs() < 0.001
    }

    pub fn get_conjugate(&self) -> Self {
        Self::simd(self.simd * f32x4::from_array([-1.0, -1.0, -1.0, 1.0]))
    }

    pub fn get_inverse(&self) -> Self {
        // The inverse of a unit quaternion is its conjugate
        assert!(self.is_normalized());
        self.get_conjugate()
    }

    /// Returns true if all components are within `eps` of the other Quat
    pub fn close(&self, other: &Self) -> bool {
        let eps = 1e-5;
        (self.get_x() - other.get_x()).abs() < eps
            && (self.get_y() - other.get_y()).abs() < eps
            && (self.get_z() - other.get_z()).abs() < eps
            && (self.get_w() - other.get_w()).abs() < eps
    }

    pub fn to_array(self) -> [f32; 4] {
        self.simd.to_array()
    }
}

impl Default for Quat {
    fn default() -> Self {
        Quat::new(0.0, 0.0, 0.0, 1.0)
    }
}

impl From<[f32; 4]> for Quat {
    fn from(arr: [f32; 4]) -> Self {
        Quat::new(arr[0], arr[1], arr[2], arr[3])
    }
}

impl From<&Mat3> for Quat {
    fn from(matrix: &Mat3) -> Self {
        let mut ret;

        let t = matrix[0][0] + matrix[1][1] + matrix[2][2];
        if t > 0.0 {
            let s = 0.5 / (t + 1.0).sqrt();
            ret = Quat::new(
                (matrix[2][1] - matrix[1][2]) * s,
                (matrix[0][2] - matrix[2][0]) * s,
                (matrix[1][0] - matrix[0][1]) * s,
                0.25 / s,
            );
        } else if matrix[0][0] > matrix[1][1] && matrix[0][0] > matrix[2][2] {
            let s = 2.0 * (1.0 + matrix[0][0] - matrix[1][1] - matrix[2][2]).sqrt();
            ret = Quat::new(
                0.25 * s,
                (matrix[0][1] + matrix[1][0]) / s,
                (matrix[0][2] + matrix[2][0]) / s,
                (matrix[2][1] - matrix[1][2]) / s,
            );
        } else if matrix[1][1] > matrix[2][2] {
            let s = 2.0 * (1.0 + matrix[1][1] - matrix[0][0] - matrix[2][2]).sqrt();
            ret = Quat::new(
                (matrix[0][1] + matrix[1][0]) / s,
                0.25 * s,
                (matrix[1][2] + matrix[2][1]) / s,
                (matrix[0][2] - matrix[2][0]) / s,
            );
        } else {
            let s = 2.0 * (1.0 + matrix[2][2] - matrix[0][0] - matrix[1][1]).sqrt();
            ret = Quat::new(
                (matrix[0][2] + matrix[2][0]) / s,
                (matrix[1][2] + matrix[2][1]) / s,
                0.25 * s,
                (matrix[1][0] - matrix[0][1]) / s,
            );
        }

        ret.normalize();

        ret
    }
}

impl From<&Mat4> for Quat {
    fn from(matrix: &Mat4) -> Self {
        let mut ret;

        let t = matrix.get(0, 0) + matrix.get(1, 1) + matrix.get(2, 2);
        if t > 0.0 {
            let s = 0.5 / (t + 1.0).sqrt();
            ret = Quat::new(
                (matrix.get(2, 1) - matrix.get(1, 2)) * s,
                (matrix.get(0, 2) - matrix.get(2, 0)) * s,
                (matrix.get(1, 0) - matrix.get(0, 1)) * s,
                0.25 / s,
            )
        } else if matrix.get(0, 0) > matrix.get(1, 1) && matrix.get(0, 0) > matrix.get(2, 2) {
            let s = 2.0 * (1.0 + matrix.get(0, 0) - matrix.get(1, 1) - matrix.get(2, 2)).sqrt();
            ret = Quat::new(
                0.25 * s,
                (matrix.get(0, 1) + matrix.get(1, 0)) / s,
                (matrix.get(0, 2) + matrix.get(2, 0)) / s,
                (matrix.get(2, 1) - matrix.get(1, 2)) / s,
            );
        } else if matrix.get(1, 1) > matrix.get(2, 2) {
            let s = 2.0 * (1.0 + matrix.get(1, 1) - matrix.get(0, 0) - matrix.get(2, 2)).sqrt();
            ret = Quat::new(
                (matrix.get(0, 1) + matrix.get(1, 0)) / s,
                0.25 * s,
                (matrix.get(1, 2) + matrix.get(2, 1)) / s,
                (matrix.get(0, 2) - matrix.get(2, 0)) / s,
            );
        } else {
            let s = 2.0 * (1.0 + matrix.get(2, 2) - matrix.get(0, 0) - matrix.get(1, 1)).sqrt();
            ret = Quat::new(
                (matrix.get(0, 2) + matrix.get(2, 0)) / s,
                (matrix.get(1, 2) + matrix.get(2, 1)) / s,
                0.25 * s,
                (matrix.get(1, 0) - matrix.get(0, 1)) / s,
            );
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

    /// Given two quaternions q1 and q2, the product q = q2 * q1 represents the composition
    /// of rotations where q1 is applied first, then q2. That is, for a vector v: `v' = q * v = q2 * (q1 * v)`
    /// This matches the standard mathematical convention for quaternion rotation composition.
    fn mul(self, rhs: Quat) -> Self::Output {
        // This implementation ensures that the resulting quaternion encodes the rotation q2 followed by q1.
        // See the tests for examples and verification of this convention.
        Self::new(
            self.get_x() * rhs.get_w() + self.get_y() * rhs.get_z() - self.get_z() * rhs.get_y()
                + self.get_w() * rhs.get_x(),
            -self.get_x() * rhs.get_z()
                + self.get_y() * rhs.get_w()
                + self.get_z() * rhs.get_x()
                + self.get_w() * rhs.get_y(),
            self.get_x() * rhs.get_y() - self.get_y() * rhs.get_x()
                + self.get_z() * rhs.get_w()
                + self.get_w() * rhs.get_z(),
            -self.get_x() * rhs.get_x() - self.get_y() * rhs.get_y() - self.get_z() * rhs.get_z()
                + self.get_w() * rhs.get_w(),
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
        rhs.rotate(self);
        rhs
    }
}

impl Serialize for Quat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_array().serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Quat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let arr: [f32; 4] = Deserialize::deserialize(deserializer)?;
        Ok(Quat::from(arr))
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
        assert!(a.get_x() == b.get_x());
        assert!(a.get_y() == -b.get_y());
        assert!(a.get_z() == b.get_z());
        assert!(a.get_w() == b.get_w());
        assert!(b.is_normalized());
    }

    #[test]
    fn identity_and_getters() {
        let q = Quat::IDENTITY;
        assert_eq!(q.get_x(), 0.0);
        assert_eq!(q.get_y(), 0.0);
        assert_eq!(q.get_z(), 0.0);
        assert_eq!(q.get_w(), 1.0);
        assert!(q.is_normalized());
    }

    #[test]
    fn axis_angle_normalization() {
        let axis = Vec3::new(1.0, 0.0, 0.0);
        let angle = std::f32::consts::PI;
        let q = Quat::axis_angle(axis, angle);
        assert!(q.is_normalized());
    }

    #[test]
    fn conjugate_and_inverse() {
        let q = Quat::new(1.0, 2.0, 3.0, 4.0);
        let mut qn = q;
        qn.normalize();
        let conj = qn.get_conjugate();
        let inv = qn.get_inverse();
        assert_eq!(conj, inv);
        assert!(inv.is_normalized());
    }

    #[test]
    fn mul_quat_identity() {
        let q = Quat::axis_angle(Vec3::new(0.0, 1.0, 0.0), std::f32::consts::FRAC_PI_2);
        let id = Quat::IDENTITY;
        assert_eq!(q * id, q);
        assert_eq!(id * q, q);
    }

    #[test]
    fn dot_and_len() {
        let q = Quat::new(1.0, 0.0, 0.0, 0.0);
        assert_eq!(q.dot(&q), 1.0);
        assert_eq!(q.len(), 1.0);
    }

    #[test]
    fn hamilton_product_vs_expected() {
        // Two 90-degree rotations around X and Y
        let qx = Quat::axis_angle(Vec3::new(1.0, 0.0, 0.0), std::f32::consts::FRAC_PI_2);
        let qy = Quat::axis_angle(Vec3::new(0.0, 1.0, 0.0), std::f32::consts::FRAC_PI_4);
        // Expected Hamilton product (qx * qy)
        // Canonical Hamilton product: (qx * qy) != (qy * qx)
        let prod1 = qx * qy;
        let prod2 = qy * qx;
        // For correct Hamilton product, prod1 and prod2 should not be equal
        assert_ne!(
            prod1, prod2,
            "Quaternion multiplication is commutative, which is incorrect for Hamilton product"
        );
    }

    #[test]
    fn hamilton_product_known_values() {
        // q1: 90 deg around X, q2: 90 deg around Y
        let q1 = Quat::axis_angle(Vec3::new(1.0, 0.0, 0.0), std::f32::consts::FRAC_PI_2);
        let q2 = Quat::axis_angle(Vec3::new(0.0, 1.0, 0.0), std::f32::consts::FRAC_PI_2);
        // Instead of comparing to an axis-angle quaternion, compare the effect of sequential rotations
        let v = Vec3::new(1.0, 0.0, 0.0);
        let v_seq = q2 * (q1 * v); // Apply q1, then q2
        let v_prod = (q2 * q1) * v; // Compose, then apply
        let diff = (v_seq.simd - v_prod.simd).abs().reduce_sum();
        let threshold = 1e-4;
        assert!(
            diff < threshold,
            "Quaternion composition does not match sequential rotation"
        );
    }

    #[test]
    fn rotate_vector_by_quaternion() {
        // 90 deg rotation around Z axis should map (1,0,0) to (0,1,0)
        let q = Quat::axis_angle(Vec3::new(0.0, 0.0, 1.0), std::f32::consts::FRAC_PI_2);
        let v = Vec3::new(1.0, 0.0, 0.0);
        let rotated = q * v;
        assert!((rotated.get_x() - 0.0).abs() < 1e-5);
        assert!((rotated.get_y() - 1.0).abs() < 1e-5);
        assert!((rotated.get_z() - 0.0).abs() < 1e-5);
    }

    #[test]
    fn serde() {
        let quat = Quat::new(1.0, 2.0, 3.0, 4.0);
        let serialized = serde_json::to_string(&quat).unwrap();
        assert_eq!(serialized, "[1.0,2.0,3.0,4.0]");
        let deserialized: Quat = serde_json::from_str(&serialized).unwrap();
        assert_eq!(quat, deserialized);
    }
}
