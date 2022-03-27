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
        let x = (uv.x * image.width() as f32) as u32;
        let y = (uv.y * image.height() as f32) as u32;

        // TODO: fix according to sampling method
        let x = x.clamp(0, image.width() - 1);
        let y = y.clamp(0, image.height() - 1);

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
