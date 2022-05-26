// Copyright © 2022
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

    pub fn get_inverse(&self) -> Self {
        Self::new(
            -self.translation,
            self.rotation.get_inverse(),
            self.scale.get_reciprocal(),
        )
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

    fn mul(self, rhs: &Trs) -> Self::Output {
        let translation = self.translation + rhs.translation;
        let rotation = self.rotation * rhs.rotation;
        let scale = self.scale * rhs.scale;
        Trs::new(translation, rotation, scale)
    }
}

impl Mul<Ray> for &Trs {
    type Output = Ray;

    fn mul(self, mut rhs: Ray) -> Self::Output {
        rhs.translate(&self.translation);
        rhs.rotate(&self.rotation);
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

        trs.translation.x = 1.0;
        mat[0][3] = 1.0;
        assert!(Mat4::from(&trs) == mat);

        trs.scale.y = 2.0;
        mat[1][1] = 2.0;
        assert!(Mat4::from(&trs) == mat);
    }
}
