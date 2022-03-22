// Copyright © 2022-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::RGBA8;

pub struct Image {
    /// Row major, top-left origin
    buffer: Vec<RGBA8>,

    width: u32,
    height: u32,
}

impl Image {
    pub fn new(width: u32, height: u32) -> Self {
        let mut buffer = Vec::new();

        buffer.resize(width as usize * height as usize, RGBA8::default());

        Self {
            buffer,
            width,
            height,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.buffer.as_ptr() as *const u8,
                self.buffer.len() * std::mem::size_of::<u32>(),
            )
        }
    }

    pub fn data(&self) -> &[RGBA8] {
        &self.buffer
    }

    pub fn data_mut(&mut self) -> &mut [RGBA8] {
        &mut self.buffer
    }

    fn index(&self, x: u32, y: u32) -> usize {
        assert!(x < self.width);
        assert!(y < self.height);
        y as usize * self.width as usize + x as usize
    }

    pub fn get(&self, x: u32, y: u32) -> RGBA8 {
        self.buffer[self.index(x, y)]
    }

    pub fn pixels_mut(&mut self) -> Vec<Vec<&mut RGBA8>> {
        let width = self.width as usize;
        let height = self.height as usize;

        let mut pixels: Vec<Vec<&mut RGBA8>> = vec![];
        pixels.resize_with(height, || Vec::with_capacity(width));

        let mut data = self.data_mut();

        #[allow(clippy::needless_range_loop)]
        for y in 0..height {
            for _ in 0..width {
                let (pixel, rest) = data.split_first_mut().unwrap();
                data = rest;
                pixels[y].push(pixel);
            }
        }

        pixels
    }

    pub fn set(&mut self, x: u32, y: u32, value: RGBA8) {
        let index = self.index(x, y);
        self.buffer[index] = value;
    }

    pub fn clear(&mut self, color: RGBA8) {
        self.buffer.fill(color);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn default() {
        let (width, height) = (2, 1);
        let image = Image::new(width, height);
        assert!(image.height() == height && image.width() == width);
        assert!(image.get(1, 0) == RGBA8::from(0));
    }

    #[test]
    fn clear() {
        let mut image = Image::new(1, 2);
        let color = RGBA8::from(0xFFFFFFFF);
        image.clear(color);
        assert!(image.data().iter().all(|&value| value == color));
    }
}
