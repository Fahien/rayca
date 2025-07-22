// Copyright © 2022-2024
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::f32::consts::FRAC_PI_4;

use crate::*;

#[derive(Debug, Clone)]
pub struct Camera {
    pub projection: Mat4,
    pub yfov_radians: f32,
}

fn angle_from_yfov(yfov_radians: f32) -> f32 {
    (yfov_radians * 0.5).tan()
}

impl Camera {
    pub fn infinite_perspective(aspect_ratio: f32, yfov_radians: f32, near: f32) -> Self {
        let angle = angle_from_yfov(yfov_radians);
        let projection = Mat4::from([
            [1.0 / (aspect_ratio * angle), 0.0, 0.0, 0.0],
            [0.0, 1.0 / angle, 0.0, 0.0],
            [0.0, 0.0, -1.0, -2.0 * near],
            [0.0, 0.0, -1.0, 0.0],
        ]);
        Self {
            projection,
            yfov_radians,
        }
    }

    pub fn finite_perspective(aspect_ratio: f32, yfov_radians: f32, near: f32, far: f32) -> Self {
        let angle = angle_from_yfov(yfov_radians);
        let projection = Mat4::from([
            [1.0 / (aspect_ratio * angle), 0.0, 0.0, 0.0],
            [0.0, 1.0 / angle, 0.0, 0.0],
            [
                0.0,
                0.0,
                (far + near) / (near - far),
                (2.0 * far * near) / (near - far),
            ],
            [0.0, 0.0, -1.0, 0.0],
        ]);
        Self {
            projection,
            yfov_radians,
        }
    }

    pub fn orthographic(width: f32, height: f32, near: f32, far: f32) -> Self {
        let mid = Vec3::new(0.0, 0.0, near / (near - far));

        let scale = Vec3::new(
            2.0 / width,
            2.0 / height,
            -1.0 / (near - far), // positive direction is towards
        );

        let projection = Mat4::from([
            [scale.get_x(), 0.0, 0.0, 0.0],
            [0.0, scale.get_y(), 0.0, 0.0],
            [0.0, 0.0, scale.get_z(), mid.get_z()],
            [0.0, 0.0, 0.0, 1.0],
        ]);
        Self {
            projection,
            yfov_radians: 1.0,
        }
    }

    pub fn get_angle(&self) -> f32 {
        (self.yfov_radians * 0.5).tan()
    }
}

impl Default for Camera {
    fn default() -> Self {
        Camera::infinite_perspective(1.0, FRAC_PI_4, 0.1)
    }
}
