// Copyright © 2022-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::{Color, Handle, Texture};

pub struct GgxMaterial {
    pub color: Color,
    pub albedo: Handle<Texture>,
}

impl GgxMaterial {
    pub const WHITE: Self = Self {
        color: Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        },
        albedo: Handle::NONE,
    };

    pub fn new() -> Self {
        Self {
            color: Color::white(),
            albedo: Handle::NONE,
        }
    }
}

impl Default for GgxMaterial {
    fn default() -> Self {
        Self::new()
    }
}
