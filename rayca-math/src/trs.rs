// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::{
    ops::{Div, Mul},
    simd::f32x4,
};

use serde::*;

use crate::Ray;

use super::*;

pub struct TrsBuilder {
    translation: Vec3,
    rotation: Quat,
    scale: Vec3,
}

impl TrsBuilder {
    fn new() -> Self {
        Self {
            translation: Vec3::default(),
            rotation: Quat::default(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        }
    }

    /// This constructs a right-handed look-at transform, suitable for view matrices (world-to-camera).
    /// The resulting TRS will transform world coordinates into camera coordinates.
    /// The translation is set so that the camera is positioned at `eye` looking at `target` with `up` as the up direction.
    pub fn look_at(mut self, target: Vec3, eye: Vec3, up: Vec3) -> Self {
        // Z axis points towards the eye!
        let z_axis = (eye - target).get_normalized();
        let x_axis = up.cross(&z_axis).get_normalized();
        let y_axis = z_axis.cross(&x_axis);

        self.translation = Vec3::new(x_axis.dot(&-eye), y_axis.dot(&-eye), z_axis.dot(&-eye));

        let rotation_matrix = Mat4::from([
            x_axis.simd,
            y_axis.simd,
            z_axis.simd,
            f32x4::from_array([0.0, 0.0, 0.0, 1.0]),
        ]);
        self.rotation = Quat::from(rotation_matrix);

        self
    }

    pub fn translation(mut self, translation: Vec3) -> Self {
        self.translation = translation;
        self
    }

    pub fn rotation(mut self, rotation: Quat) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn scale(mut self, scale: Vec3) -> Self {
        self.scale = scale;
        self
    }

    pub fn build(self) -> Trs {
        Trs::new(self.translation, self.rotation, self.scale)
    }
}

/// TRanSform, or Translation-Rotation-Scale
/// Order of transformations: scale-rotate-translate
#[repr(C, align(16))]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Trs {
    #[serde(default)]
    pub translation: Vec3,

    #[serde(default)]
    pub rotation: Quat,

    #[serde(default = "Vec3::unit")]
    pub scale: Vec3,
}

impl Trs {
    pub const IDENTITY: Trs = Trs {
        translation: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };

    pub fn builder() -> TrsBuilder {
        TrsBuilder::new()
    }

    pub fn new(translation: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            translation,
            rotation,
            scale,
        }
    }

    pub fn get_inversed(&self) -> Inversed<Self> {
        Inversed::from(self.clone())
    }

    pub fn left_mul(&mut self, rhs: &Trs) {
        let translation = self.translation + self.rotation * (self.scale * rhs.translation);
        let rotation = self.rotation * rhs.rotation;
        let scale = rhs.rotation.get_inverse() * (self.scale * (rhs.rotation * rhs.scale));
        self.translation = translation;
        self.rotation = rotation;
        self.scale = scale;
    }

    /// Returns the translation after rotation, meaning the actual translation
    pub fn get_translation(&self) -> Vec3 {
        self.rotation * self.translation
    }

    pub fn translate(&mut self, translation: Vec3) {
        self.translation += translation;
    }

    /// Rotates by the given quaternion
    pub fn rotate(&mut self, rotation: Quat) {
        self.rotation = rotation * self.rotation;
    }

    pub fn scale(&mut self, scale: Vec3) {
        self.scale *= scale;
    }

    /// Returns a Mat4 representation
    pub fn to_mat4(&self) -> Mat4 {
        Mat4::from(self)
    }

    /// Returns a Mat3 representation
    pub fn to_mat3(&self) -> Mat3 {
        Mat3::from(self)
    }

    pub fn to_view_mat4(&self) -> Mat4 {
        let mut matrix = self.to_mat4();
        // Invert translation
        matrix.set_translation(&-matrix.get_translation());
        matrix
    }

    pub fn to_view(&self) -> Trs {
        let mut trs = self.clone();
        // Invert translation
        trs.translation = -trs.get_translation();
        trs
    }
}

impl From<Mat4> for Trs {
    fn from(mat: Mat4) -> Self {
        // WARNING: This conversion ignores any scale present in the matrix and always sets scale to (1,1,1).
        // If the matrix contains non-uniform or non-identity scale, it will be lost in the resulting Trs.
        let translation = mat.get_translation();
        let rotation = mat.get_rotation();
        let scale = Vec3::new(1.0, 1.0, 1.0);
        Self::new(translation, rotation, scale)
    }
}

impl Default for Trs {
    fn default() -> Self {
        Self {
            translation: Vec3::default(),
            rotation: Quat::default(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        }
    }
}

impl Mul<Mat4> for Trs {
    type Output = Mat4;

    fn mul(self, mut rhs: Mat4) -> Self::Output {
        rhs.scale(&self.scale);
        rhs.rotate(&self.rotation);
        rhs.translate(&self.translation);
        rhs
    }
}

impl Mul<Mat4> for &Trs {
    type Output = Mat4;

    fn mul(self, mut rhs: Mat4) -> Self::Output {
        rhs.scale(&self.scale);
        rhs.rotate(&self.rotation);
        rhs.translate(&self.translation);
        rhs
    }
}

impl Mul<&Trs> for &Trs {
    type Output = Trs;

    /// See https://gamedev.stackexchange.com/questions/167287/combine-two-translation-rotation-scale-triplets-without-matrices
    fn mul(self, rhs: &Trs) -> Self::Output {
        let translation = self.translation + self.rotation * (self.scale * rhs.translation);
        let rotation = self.rotation * rhs.rotation;
        let scale = rhs.rotation.get_inverse() * (self.scale * (rhs.rotation * rhs.scale));
        Trs::new(translation, rotation, scale)
    }
}

impl Mul<&Trs> for Trs {
    type Output = Trs;

    /// See https://gamedev.stackexchange.com/questions/167287/combine-two-translation-rotation-scale-triplets-without-matrices
    fn mul(mut self, rhs: &Trs) -> Self::Output {
        let translation = self.translation + self.rotation * (self.scale * rhs.translation);
        let rotation = self.rotation * rhs.rotation;
        let scale = rhs.rotation.get_inverse() * (self.scale * (rhs.rotation * rhs.scale));
        self.translation = translation;
        self.rotation = rotation;
        self.scale = scale;
        self
    }
}

impl Mul<&mut Trs> for Trs {
    type Output = Trs;

    /// See https://gamedev.stackexchange.com/questions/167287/combine-two-translation-rotation-scale-triplets-without-matrices
    fn mul(mut self, rhs: &mut Trs) -> Self::Output {
        let translation = self.translation + self.rotation * (self.scale * rhs.translation);
        let rotation = self.rotation * rhs.rotation;
        let scale = rhs.rotation.get_inverse() * (self.scale * (rhs.rotation * rhs.scale));
        self.translation = translation;
        self.rotation = rotation;
        self.scale = scale;
        self
    }
}

impl Mul<Vec3> for &Trs {
    type Output = Vec3;

    fn mul(self, mut rhs: Vec3) -> Self::Output {
        rhs.scale(&self.scale);
        rhs.rotate(self.rotation);
        rhs.translate(&self.translation);
        rhs
    }
}

impl Mul<Point3> for &Trs {
    type Output = Point3;

    fn mul(self, mut rhs: Point3) -> Self::Output {
        rhs.scale(&self.scale);
        rhs.rotate(self.rotation);
        rhs.translate(&self.translation);
        rhs
    }
}

impl Mul<Ray> for &Trs {
    type Output = Ray;

    fn mul(self, mut rhs: Ray) -> Self::Output {
        rhs.scale(&self.scale);
        rhs.rotate(self.rotation);
        rhs.translate(&self.translation);
        rhs
    }
}

impl Div<Ray> for &Trs {
    type Output = Ray;

    fn div(self, mut rhs: Ray) -> Self::Output {
        rhs.translate(&self.translation);
        rhs.rotate(self.rotation);
        rhs.scale(&self.scale);
        rhs
    }
}

pub struct Inversed<T> {
    pub source: T,
}

impl Inversed<Trs> {
    pub fn get_translation(&self) -> Vec3 {
        -self.source.translation
    }

    pub fn get_rotation(&self) -> Quat {
        self.source.rotation.get_inverse()
    }

    pub fn get_scale(&self) -> Vec3 {
        self.source.scale.get_reciprocal()
    }

    /// Returns a Mat4 representation
    pub fn to_mat4(&self) -> Mat4 {
        Mat4::from(self)
    }
}

impl Inversed<&Trs> {
    pub fn get_translation(&self) -> Vec3 {
        -self.source.translation
    }

    pub fn get_rotation(&self) -> Quat {
        self.source.rotation.get_inverse()
    }

    pub fn get_scale(&self) -> Vec3 {
        self.source.scale.get_reciprocal()
    }

    /// Returns a Mat4 representation
    pub fn to_mat4(&self) -> Mat4 {
        Mat4::from(self)
    }
}

impl Mul<&Mat4> for &Inversed<Trs> {
    type Output = Mat4;

    fn mul(self, rhs: &Mat4) -> Self::Output {
        Mat4::from(self) * rhs
    }
}

impl From<Trs> for Inversed<Trs> {
    fn from(source: Trs) -> Self {
        Self { source }
    }
}

impl<'a> From<&'a Trs> for Inversed<&'a Trs> {
    fn from(source: &'a Trs) -> Self {
        Self { source }
    }
}

impl Mul<Point3> for &Inversed<&Trs> {
    type Output = Point3;

    fn mul(self, rhs: Point3) -> Self::Output {
        // (T * R * S)^(-1) = S^(-1) * R^(-1) * T^(-1)
        // This approach works
        (Mat4::from_scale(&self.get_scale())
            * (Mat4::from_rotation(&self.get_rotation())
                * Mat4::from_translation(&self.get_translation())))
            * rhs
    }
}

impl Mul<Point3> for &Inversed<Trs> {
    type Output = Point3;

    fn mul(self, rhs: Point3) -> Self::Output {
        (Mat4::from_scale(&self.get_scale())
            * (Mat4::from_rotation(&self.get_rotation())
                * Mat4::from_translation(&self.get_translation())))
            * rhs
    }
}

impl Mul<Vec3> for &Inversed<&Trs> {
    type Output = Vec3;

    fn mul(self, mut rhs: Vec3) -> Self::Output {
        rhs.translate(&self.get_translation());
        rhs.rotate(self.get_rotation());
        rhs.scale(&self.get_scale());
        rhs
    }
}

impl Mul<Vec3> for &Inversed<Trs> {
    type Output = Vec3;

    fn mul(self, mut rhs: Vec3) -> Self::Output {
        rhs.translate(&self.get_translation());
        rhs.rotate(self.get_rotation());
        rhs.scale(&self.get_scale());
        rhs
    }
}

impl Mul<Ray> for &Inversed<&Trs> {
    type Output = Ray;

    fn mul(self, mut rhs: Ray) -> Self::Output {
        rhs.translate(&self.get_translation());
        rhs.rotate(self.get_rotation());
        rhs.scale(&self.get_scale());
        rhs
    }
}

impl Mul<Ray> for &Inversed<Trs> {
    type Output = Ray;

    fn mul(self, mut rhs: Ray) -> Self::Output {
        rhs.translate(&self.get_translation());
        rhs.rotate(self.get_rotation());
        rhs.scale(&self.get_scale());
        rhs
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn mul() {
        let mut trs = Trs::default();
        let mut mat = Mat4::identity();
        assert!(Mat4::from(&trs) == mat);

        trs.translation.set_x(1.0);
        mat.set(0, 3, 1.0);
        assert!(Mat4::from(&trs) == mat);

        trs.scale.set_y(2.0);
        mat.set(1, 1, 2.0);
        assert!(Mat4::from(&trs) == mat);
    }

    #[test]
    fn look_at() {
        let origin = Vec3::default();
        let y_axis = Vec3::new(0.0, 1.0, 0.0);
        let eye = Vec3::new(0.0, 0.0, 1.0);

        // This transforms world coordinates into camera coordinates
        let world_to_camera_trs = Trs::builder().look_at(origin, eye, y_axis).build();

        // This transforms camera coordinates to world transform
        let camera_to_world_trs = world_to_camera_trs.get_inversed();
        assert_eq!(camera_to_world_trs.get_translation(), eye);
        assert_eq!(camera_to_world_trs.get_rotation(), Quat::default());
    }

    #[test]
    fn combine_trs_identity() {
        let trs1 = Trs::default();
        let trs2 = Trs::default();
        let combined = &trs1 * &trs2;
        assert_eq!(combined.translation, Vec3::default());
        assert_eq!(combined.rotation, Quat::default());
        assert_eq!(combined.scale, Vec3::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn combine_trs_translation() {
        let trs1 = Trs::new(
            Vec3::new(1.0, 2.0, 3.0),
            Quat::default(),
            Vec3::new(1.0, 1.0, 1.0),
        );
        let trs2 = Trs::new(
            Vec3::new(4.0, 5.0, 6.0),
            Quat::default(),
            Vec3::new(1.0, 1.0, 1.0),
        );
        let combined = &trs1 * &trs2;
        assert_eq!(combined.translation, Vec3::new(5.0, 7.0, 9.0));
    }

    #[test]
    fn combine_trs_scale() {
        let trs1 = Trs::new(Vec3::default(), Quat::default(), Vec3::new(2.0, 2.0, 2.0));
        let trs2 = Trs::new(Vec3::default(), Quat::default(), Vec3::new(0.5, 0.5, 0.5));
        let combined = &trs1 * &trs2;
        assert_eq!(combined.scale, Vec3::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn combine_trs_rotation() {
        let rot1 = Quat::axis_angle(Vec3::new(0.0, 1.0, 0.0), std::f32::consts::FRAC_PI_2);
        let rot2 = Quat::axis_angle(Vec3::new(0.0, 1.0, 0.0), std::f32::consts::FRAC_PI_2);
        let trs1 = Trs::new(Vec3::default(), rot1, Vec3::new(1.0, 1.0, 1.0));
        let trs2 = Trs::new(Vec3::default(), rot2, Vec3::new(1.0, 1.0, 1.0));
        let combined = &trs1 * &trs2;
        let expected = Quat::axis_angle(Vec3::new(0.0, 1.0, 0.0), std::f32::consts::PI);
        assert!(combined.rotation.close(&expected));
    }

    #[test]
    fn inverse_trs_translation_only() {
        let trs = Trs::new(
            Vec3::new(1.0, 2.0, 3.0),
            Quat::default(),
            Vec3::new(1.0, 1.0, 1.0),
        );
        let inv = trs.get_inversed();
        let v = Vec3::new(4.0, 5.0, 6.0);
        let transformed = &trs * v;
        let restored = &inv * transformed;
        assert!(restored.close(&v));
    }

    #[test]
    fn inverse_trs_scale_only() {
        let trs = Trs::new(Vec3::default(), Quat::default(), Vec3::new(2.0, 3.0, 4.0));
        let inv = trs.get_inversed();
        let v = Vec3::new(8.0, 9.0, 12.0);
        let transformed = &trs * v;
        let restored = &inv * transformed;
        assert!(restored.close(&v));
    }

    #[test]
    fn inverse_trs_rotation_only() {
        let rot = Quat::axis_angle(Vec3::new(0.0, 0.0, 1.0), std::f32::consts::FRAC_PI_2);
        let trs = Trs::new(Vec3::default(), rot, Vec3::new(1.0, 1.0, 1.0));
        let inv = trs.get_inversed();
        let v = Vec3::new(1.0, 0.0, 0.0);
        let transformed = &trs * v;
        let restored = &inv * transformed;
        assert!(restored.close(&v));
    }

    #[test]
    fn inverse_trs_full() {
        let trs = Trs::new(
            Vec3::new(1.0, 2.0, 3.0),
            Quat::axis_angle(Vec3::new(0.0, 0.0, 1.0), std::f32::consts::FRAC_PI_2),
            Vec3::new(2.0, 2.0, 2.0),
        );
        let inv = trs.get_inversed();
        let v = Vec3::new(1.0, 1.0, 1.0);
        let transformed = &trs * v;
        let restored = &inv * transformed;
        assert!(restored.close(&v));
    }
}
