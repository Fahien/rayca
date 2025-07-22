// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

#![feature(portable_simd)]

pub mod color;
pub mod mat3;
pub mod mat4;
pub mod ops;
pub mod point3;
pub mod quat;
pub mod ray;
pub mod size2;
pub mod trs;
pub mod vec2;
pub mod vec3;
pub mod vec4;

pub use color::*;
pub use mat3::*;
pub use mat4::*;
pub use ops::*;
pub use point3::*;
pub use quat::*;
pub use ray::*;
pub use size2::*;
pub use trs::*;
pub use vec2::*;
pub use vec3::*;
pub use vec4::*;

const EPS: f32 = f32::EPSILON * 8192.0;
