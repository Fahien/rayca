// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::{fs::File, io::BufWriter, path::Path};

use super::RGBA8;

#[derive(Default)]
pub struct Image {
    pub id: usize,

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
            id: 0,
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

    pub fn bytes_mut(&mut self) -> &mut [u8] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.buffer.as_mut_ptr() as *mut u8,
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

    /// Opens a PNG file without loading data yet
    pub fn load_png<P: AsRef<Path>>(path: P) -> Image {
        let current_dir = std::env::current_dir().expect("Failed to get current dir");
        let file = File::open(path.as_ref()).expect(&format!(
            "Failed to load PNG file: {}/{}",
            current_dir.display(),
            path.as_ref().display()
        ));

        let decoder = png::Decoder::new(file);
        let mut reader = decoder.read_info().unwrap();
        let info = reader.info();

        let mut ret = Self::new(info.width, info.height);
        reader.next_frame(ret.bytes_mut()).expect(&format!(
            "Failed to read frame from PNG file: {}/{}",
            current_dir.display(),
            path.as_ref().display()
        ));
        ret
    }

    pub fn dump_png<P: AsRef<Path>>(&self, path: P) {
        let file = File::create(path).unwrap();
        let ref mut w = BufWriter::new(file);

        let mut encoder = png::Encoder::new(w, self.width, self.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        // 1.0 / 2.2, scaled by 100000
        encoder.set_source_gamma(png::ScaledFloat::from_scaled(45455));
        // 1.0 / 2.2, unscaled, but rounded
        encoder.set_source_gamma(png::ScaledFloat::new(1.0 / 2.2));
        // Using unscaled instantiation here
        let source_chromaticities = png::SourceChromaticities::new(
            (0.31270, 0.32900),
            (0.64000, 0.33000),
            (0.30000, 0.60000),
            (0.15000, 0.06000),
        );
        encoder.set_source_chromaticities(source_chromaticities);
        let mut writer = encoder.write_header().unwrap();

        writer.write_image_data(self.bytes()).unwrap(); // Save
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

    #[test]
    fn dump() {
        let image = Image::new(32, 32);
        image.dump_png("target/image.png");
    }

    #[test]
    fn load() {
        let mut image = Image::new(1, 1);
        let color = RGBA8::from(0x0000FFFF);
        image.clear(color);

        let blue_path = "target/blue.png";
        image.dump_png(blue_path);

        let image = Image::load_png(blue_path);
        assert!(image.data().iter().all(|&value| value == color));
    }
}
