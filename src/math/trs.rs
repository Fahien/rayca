// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::ops::Mul;

use super::*;

/// TRanSform, or Translation-Rotation-Scale
/// Order of transformations: scale-rotate-translate
#[derive(Clone)]
pub struct Trs {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Trs {
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

        trs.translation.x = 1.0;
        mat[0][3] = 1.0;
        assert!(Mat4::from(&trs) == mat);

        trs.scale.y = 2.0;
        mat[1][1] = 2.0;
        assert!(Mat4::from(&trs) == mat);
    }
}
