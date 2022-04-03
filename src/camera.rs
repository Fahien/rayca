// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::f32::consts::FRAC_PI_4;

use super::*;

pub struct Camera {
    pub projection: Mat4,
    pub yfov: f32,
}

impl Camera {
    pub fn infinite_perspective(aspect_ratio: f32, yfov: f32, near: f32) -> Self {
        let tan_frac_yfov_2 = (yfov * 0.5).tan();
        let projection = Mat4::from([
            [1.0 / (aspect_ratio * tan_frac_yfov_2), 0.0, 0.0, 0.0],
            [0.0, 1.0 / tan_frac_yfov_2, 0.0, 0.0],
            [0.0, 0.0, -1.0, -2.0 * near],
            [0.0, 0.0, -1.0, 0.0],
        ]);
        Self { projection, yfov }
    }

    pub fn finite_perspective(aspect_ratio: f32, yfov: f32, near: f32, far: f32) -> Self {
        let tan_frac_yfov_2 = (yfov / 2.0).tan();
        let projection = Mat4::from([
            [1.0 / (aspect_ratio * tan_frac_yfov_2), 0.0, 0.0, 0.0],
            [0.0, 1.0 / tan_frac_yfov_2, 0.0, 0.0],
            [
                0.0,
                0.0,
                (far + near) / (near - far),
                (2.0 * far * near) / (near - far),
            ],
            [0.0, 0.0, -1.0, 0.0],
        ]);
        Self { projection, yfov }
    }

    pub fn orthographic(width: f32, height: f32, near: f32, far: f32) -> Self {
        let r = width / 2.0;
        let t = height / 2.0;
        let projection = Mat4::from([
            [1.0 / r, 0.0, 0.0, 0.0],
            [0.0, 1.0 / t, 0.0, 0.0],
            [0.0, 0.0, 2.0 / (near - far), (far + near) / (near - far)],
            [0.0, 0.0, 0.0, 1.0],
        ]);
        Self { projection, yfov: 1.0 }
    }
}

impl Default for Camera {
    fn default() -> Self {
        Camera::infinite_perspective(1.0, FRAC_PI_4, 0.1)
    }
}
