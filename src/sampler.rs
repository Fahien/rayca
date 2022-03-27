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

    pub fn sample(&self, image: &Image, uv: &Vec2) -> RGBA8 {
        let width = image.width() as f32;
        let height = image.height() as f32;
        let x = uv.x * width;
        let y = uv.y * height;

        // TODO: fix according to sampling method
        let x = (x as u32) % image.width();
        let y = (y as u32) % image.height();

        image.get(x, y)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sampler() {
        let sampler = Sampler::default();
        let mut image = Image::new(1, 1);
        let color = RGBA8::white();
        image.clear(color);

        let uv = Vec2::default();
        let pixel = sampler.sample(&image, &uv);
        assert!(pixel == color);

        let uv = Vec2::new(1.0, 1.0);
        let pixel = sampler.sample(&image, &uv);
        assert!(pixel == color);
    }
}
