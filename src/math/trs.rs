// Copyright © 2022-2024
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::ops::Mul;

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
#[derive(Clone)]
pub struct Trs {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Trs {
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

impl Mul<Point3> for &Trs {
    type Output = Point3;

    fn mul(self, mut rhs: Point3) -> Self::Output {
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

impl Mul<Ray> for &Trs {
    type Output = Ray;

    fn mul(self, mut rhs: Ray) -> Self::Output {
        rhs.scale(&self.scale);
        rhs.rotate(&self.rotation);
        rhs.translate(&self.translation);
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn mul() {
        let mut trs = Trs::default();
        let mut mat = Mat4::identity();
        assert!(Mat4::from(&trs) == mat);

        trs.translation.set_x(1.0);
        mat[0][3] = 1.0;
        assert!(Mat4::from(&trs) == mat);

        trs.scale.set_y(2.0);
        mat[1][1] = 2.0;
        assert!(Mat4::from(&trs) == mat);
    }
}
