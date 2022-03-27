// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use super::*;

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
