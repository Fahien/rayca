// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::{Color, HitInfo, Vec3};

pub trait Sample: Sync {
    /// Returns the shadow ray direction used for this sample
    fn get_omega(&self) -> Vec3;

    /// Returns this sample's value
    fn get_x(&self) -> Color;

    /// Returns this samples' pdf value
    fn get_pdf(&self) -> f32;

    /// Calculates the pdf associated to this sample for another sample
    fn get_pdf_for(&self, hit: &mut HitInfo<'_>, sample: &dyn Sample) -> f32;
}
