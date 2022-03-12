// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

pub struct Image {
    /// Row major, top-left origin
    buffer: Vec<u32>,

    width: u32,
    height: u32,
}

impl Image {
    pub fn new(width: u32, height: u32) -> Self {
        let mut buffer = Vec::new();

        buffer.resize(width as usize * height as usize, 0);

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

    pub fn data(&self) -> &[u32] {
        &self.buffer
    }

    pub fn data_mut(&mut self) -> &mut [u32] {
        &mut self.buffer
    }

    fn index(&self, x: u32, y: u32) -> usize {
        assert!(x < self.width);
        assert!(y < self.height);
        y as usize * self.width as usize + x as usize
    }

    pub fn get(&self, x: u32, y: u32) -> u32 {
        self.buffer[self.index(x, y)]
    }

    pub fn set(&mut self, x: u32, y: u32, value: u32) {
        let index = self.index(x, y);
        self.buffer[index] = value;
    }

    pub fn clear(&mut self, color: u32) {
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
        assert!(image.get(1, 0) == 0);
    }

    #[test]
    fn clear() {
        let mut image = Image::new(1, 2);
        let color = 0xFFFFFFFFu32;
        image.clear(color);
        assert!(image.data().iter().all(|&value| value == color));
    }
}
