// Copyright © 2022-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::{Color, Image, Vec2};

#[derive(Default)]
pub struct Sampler {}

impl Sampler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn sample(&self, image: &Image, uv: &Vec2) -> Color {
        let x = (uv.x * image.width() as f32) as u32;
        let y = (uv.y * image.height() as f32) as u32;

        // TODO: fix according to sampling method
        let x = x.clamp(0, image.width() - 1);
        let y = y.clamp(0, image.height() - 1);

        image.get(x, y).into()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sampler() {
        let sampler = Sampler::default();
        let mut image = Image::new(1, 1);
        let color = Color::white();
        image.clear(color.into());

        let uv = Vec2::default();
        let pixel = sampler.sample(&image, &uv);
        assert!(pixel == color);

        let uv = Vec2::new(1.0, 1.0);
        let pixel = sampler.sample(&image, &uv);
        assert!(pixel == color);
    }
}
