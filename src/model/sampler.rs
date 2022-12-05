// Copyright © 2022-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::{Color, ColorType, Image, Vec2, RGB8, RGBA8};

#[derive(Default)]
pub struct Sampler {}

impl Sampler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn sample(&self, image: &Image, uv: &Vec2) -> Color {
        let width = image.width() as f32;
        let height = image.height() as f32;
        let x = (uv.x - uv.x.floor() + 1.0) * width;
        let y = (uv.y - uv.y.floor() + 1.0) * height;

        // TODO: fix according to sampling method
        let x = (x as u32) % image.width();
        let y = (y as u32) % image.height();

        match image.color_type {
            ColorType::RGBA8 => image.get::<RGBA8>(x, y).into(),
            ColorType::RGB8 => {
                let rgb8 = image.get::<RGB8>(x, y);
                RGBA8::from(rgb8).into()
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sampler() {
        let sampler = Sampler::default();
        let mut image = Image::new(1, 1, ColorType::RGBA8);
        let color = Color::white();
        image.clear::<RGBA8>(color.into());

        let uv = Vec2::default();
        let pixel = sampler.sample(&image, &uv);
        assert!(pixel == color);

        let uv = Vec2::new(1.0, 1.0);
        let pixel = sampler.sample(&image, &uv);
        assert!(pixel == color);
    }
}
