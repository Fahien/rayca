// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};

use jpeg_decoder as jpeg;
use png::Transformations;

use super::*;

#[derive(Default)]
pub struct Image {
    pub id: usize,

    /// Row major, top-left origin
    pub color_type: ColorType,
    buffer: Vec<u8>,

    width: u32,
    height: u32,
}

impl Image {
    pub fn new(width: u32, height: u32, color_type: ColorType) -> Self {
        let mut buffer = Vec::new();
        buffer.resize(width as usize * height as usize * color_type.channels(), 0);

        Self {
            id: 0,
            color_type,
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
        &self.buffer
    }

    pub fn bytes_mut(&mut self) -> &mut [u8] {
        &mut self.buffer
    }

    pub fn data<Col: ColorTyped>(&self) -> &[Col] {
        assert!(Col::color_type() == self.color_type);
        unsafe {
            std::slice::from_raw_parts(
                self.buffer.as_ptr() as *const Col,
                self.buffer.len() / std::mem::size_of::<Col>(),
            )
        }
    }

    pub fn data_mut<Col: ColorTyped>(&mut self) -> &mut [Col] {
        assert!(Col::color_type() == self.color_type);
        unsafe {
            std::slice::from_raw_parts_mut(
                self.buffer.as_mut_ptr() as *mut Col,
                self.buffer.len() / std::mem::size_of::<Col>(),
            )
        }
    }

    fn index(&self, x: u32, y: u32) -> usize {
        assert!(x < self.width);
        assert!(y < self.height);
        y as usize * self.width as usize + x as usize
    }

    pub fn get<Col: ColorTyped>(&self, x: u32, y: u32) -> Col {
        assert!(Col::color_type() == self.color_type);
        self.data()[self.index(x, y)]
    }

    pub fn pixels_mut<Col: ColorTyped>(&mut self) -> Vec<Vec<&mut Col>> {
        assert!(Col::color_type() == self.color_type);
        let width = self.width as usize;
        let height = self.height as usize;

        let mut pixels: Vec<Vec<&mut Col>> = vec![];
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

    pub fn set<Col: ColorTyped>(&mut self, x: u32, y: u32, value: Col) {
        assert!(Col::color_type() == self.color_type);
        let index = self.index(x, y);
        self.data_mut()[index] = value;
    }

    pub fn clear<Col: ColorTyped>(&mut self, color: Col) {
        assert!(Col::color_type() == self.color_type);
        self.data_mut().fill(color);
    }

    pub fn load_png_data(data: &[u8]) -> Image {
        let mut decoder = png::Decoder::new(data);
        decoder.set_transformations(Transformations::normalize_to_color8());

        let mut reader = decoder.read_info().unwrap();
        let info = reader.info();

        let color_type = match info.color_type {
            png::ColorType::Rgb => ColorType::RGB8,
            png::ColorType::Rgba => ColorType::RGBA8,
            png::ColorType::Indexed => ColorType::RGB8,
            _ => panic!("Color type not supported: {:?}", info.color_type),
        };

        let mut ret = Self::new(info.width, info.height, color_type);
        reader
            .next_frame(ret.bytes_mut())
            .expect("Failed to read frame from PNG data");
        ret
    }

    /// Opens a PNG file without loading data yet
    pub fn load_png_file<P: AsRef<Path>>(path: P) -> Image {
        let current_dir = std::env::current_dir().expect("Failed to get current dir");
        let Ok(file) = File::open(path.as_ref()) else {
            log::error!(
                "Failed to load PNG file: {}/{}",
                current_dir.display(),
                path.as_ref().display()
            );
            panic!();
        };

        let mut decoder = png::Decoder::new(file);
        decoder.set_transformations(Transformations::normalize_to_color8());

        let mut reader = decoder.read_info().unwrap();
        let info = reader.info();

        let color_type = match info.color_type {
            png::ColorType::Rgb => ColorType::RGB8,
            png::ColorType::Rgba => ColorType::RGBA8,
            png::ColorType::Indexed => ColorType::RGB8,
            _ => panic!("Color type not supported: {:?}", info.color_type),
        };

        let mut ret = Self::new(info.width, info.height, color_type);
        if reader.next_frame(ret.bytes_mut()).is_err() {
            log::error!(
                "Failed to read frame from PNG file: {}/{}",
                current_dir.display(),
                path.as_ref().display()
            );
            panic!();
        }
        ret
    }

    pub fn dump_png<P: AsRef<Path>>(&self, path: P) {
        let Ok(file) = File::create(path.as_ref()) else {
            log::error!("Failed to create PNG file: {}", path.as_ref().display());
            panic!();
        };
        let w = BufWriter::new(file);

        let mut encoder = png::Encoder::new(w, self.width, self.height);

        let png_color_type = match self.color_type {
            ColorType::RGB8 => png::ColorType::Rgb,
            ColorType::RGBA8 => png::ColorType::Rgba,
        };
        encoder.set_color(png_color_type);
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

    pub fn load_jpg_file<P: AsRef<Path>>(path: P) -> Image {
        let file = File::open(path).expect("Failed to open JPG file");
        let mut decoder = jpeg::Decoder::new(BufReader::new(file));
        let pixels = decoder.decode().expect("Failed to decode JPG image");
        let metadata = decoder.info().unwrap();

        let mut image = Image::new(
            metadata.width as u32,
            metadata.height as u32,
            ColorType::RGB8,
        );
        image.buffer = pixels;
        image
    }

    pub fn load_file<P: AsRef<Path>>(path: P) -> Image {
        let ext = path.as_ref().extension().unwrap();

        if ext.eq_ignore_ascii_case("png") {
            Self::load_png_file(path)
        } else if ext.eq_ignore_ascii_case("jpg") {
            Self::load_jpg_file(path)
        } else {
            panic!(
                "Unsupported image extension: {} ({})",
                ext.to_string_lossy(),
                path.as_ref().display()
            )
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn default() {
        let (width, height) = (2, 1);
        let image = Image::new(width, height, ColorType::RGBA8);
        assert!(image.height() == height && image.width() == width);
        assert!(image.get::<RGBA8>(1, 0) == RGBA8::from(0));
    }

    #[test]
    fn clear() {
        let mut image = Image::new(1, 2, ColorType::RGBA8);
        let color = RGBA8::from(0xFFFFFFFF);
        image.clear(color);
        assert!(image.data().iter().all(|&value: &RGBA8| value == color));
    }

    #[test]
    fn dump() {
        let image = Image::new(32, 32, ColorType::RGBA8);
        image.dump_png(get_artifacts_path().join("image.png"));
    }

    #[test]
    fn load() {
        let mut image = Image::new(1, 1, ColorType::RGBA8);
        let color = RGBA8::from(0x0000FFFF);
        image.clear(color);

        let blue_path = get_artifacts_path().join("blue.png");
        image.dump_png(&blue_path);

        let image = Image::load_png_file(blue_path);
        assert!(image
            .data::<RGBA8>()
            .iter()
            .all(|&value: &RGBA8| value == color));
    }

    #[test]
    fn base64() {
        const DUCK_BASE64: &str = include_str!("../tests/model/duck/duck.base64");
        let duck_data = base64::decode(DUCK_BASE64).expect("Failed to decode duck base64");
        let image = Image::load_png_data(&duck_data);
        image.dump_png(get_artifacts_path().join("duck-texture.png"));
        logging::init();
        log::info!("{:?}", image.data::<RGB8>()[0]);
    }
}
