// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

pub trait Dot<T> {
    fn dot(self, rhs: T) -> f32;
}
