// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::ops::Mul;

use super::*;

/// TRanSform, or Translation-Rotation-Scale
/// Order of transformations: scale-rotate-translate
pub struct Trs {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Trs {
    pub fn new() -> Self {
        Self {
            translation: Vec3::default(),
            rotation: Quat::default(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        }
    }
}

impl Default for Trs {
    fn default() -> Self {
        Self::new()
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn mul() {
        let mut trs = Trs::new();
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
