// Copyright © 2024-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::{io, path::Path};

use crate::*;

#[derive(Clone, Default)]
pub struct ImageBuilder {
    uri: String,
}

impl ImageBuilder {
    pub fn uri<S: Into<String>>(mut self, uri: S) -> Self {
        self.uri = uri.into();
        self
    }

    pub fn build(self) -> Image {
        Image::new_with_uri(self.uri)
    }
}

#[derive(Clone, Debug, Default)]
pub struct Image {
    pub uri: String,

    /// Row major, top-left origin
    pub color_type: ColorType,
    buffer: Vec<u8>,

    width: u32,
    height: u32,
}

impl Image {
    pub fn builder() -> ImageBuilder {
        ImageBuilder::default()
    }

    pub fn new_with_uri(uri: String) -> Self {
        Self {
            uri,
            ..Default::default()
        }
    }

    pub fn new(width: u32, height: u32, color_type: ColorType) -> Self {
        let mut buffer = Vec::new();
        let buffer_size = width * height * color_type.channels();
        buffer.resize(buffer_size as usize, 0);

        Self {
            color_type,
            buffer,
            width,
            height,
            ..Default::default()
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

    pub fn load_data(data: &[u8]) -> Image {
        let image_reader = ::image::ImageReader::new(io::Cursor::new(data))
            .with_guessed_format()
            .expect("Failed to guess image format")
            .decode()
            .expect("Failed to decode image");

        let color_type = from_image_color_type(image_reader.color());

        let width = image_reader.width();
        let height = image_reader.height();

        let mut ret = Self::new(width, height, color_type);
        ret.buffer.copy_from_slice(image_reader.as_bytes());
        ret
    }

    pub fn dump_png<P: AsRef<Path>>(&self, path: P) -> Result<(), ::image::ImageError> {
        ::image::save_buffer_with_format(
            path,
            &self.buffer,
            self.width,
            self.height,
            to_image_color_type(self.color_type),
            ::image::ImageFormat::Png,
        )
    }

    pub fn load_file<P: AsRef<Path>>(path: P, assets: &Assets) -> io::Result<Image> {
        let data = assets.load(path)?.into_bytes();
        Ok(Self::load_data(&data))
    }
}

fn to_image_color_type(color_type: ColorType) -> ::image::ColorType {
    match color_type {
        ColorType::RGB8 => ::image::ColorType::Rgb8,
        ColorType::RGBA8 => ::image::ColorType::Rgba8,
        ColorType::RGBA32F => ::image::ColorType::Rgba32F,
    }
}

fn from_image_color_type(color_type: ::image::ColorType) -> ColorType {
    match color_type {
        ::image::ColorType::Rgb8 => ColorType::RGB8,
        ::image::ColorType::Rgba8 => ColorType::RGBA8,
        ::image::ColorType::Rgba32F => ColorType::RGBA32F,
        _ => panic!("Unsupported image color type: {:?}", color_type),
    }
}

impl std::fmt::Display for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ \"uri\": \"{}\" }}", self.uri)
    }
}

#[cfg(test)]
mod test {
    use base64::Engine;

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
        image
            .dump_png(tests::get_artifact_path().join("image.png"))
            .expect("Failed to dump image");
    }

    #[test]
    fn load() {
        let mut image = Image::new(1, 1, ColorType::RGBA8);
        let color = RGBA8::from(0x0000FFFF);
        image.clear(color);

        let blue_path = tests::get_artifact_path().join("blue.png");
        image.dump_png(&blue_path).expect("Failed to dump image");

        let assets = Assets::new();
        let image = Image::load_file(blue_path, &assets).expect("Failed to load image");
        assert!(
            image
                .data::<RGBA8>()
                .iter()
                .all(|&value: &RGBA8| value == color)
        );
    }

    #[test]
    fn base64() {
        const DUCK_BASE64: &str = include_str!("../../tests/model/duck/duck.base64");
        let duck_data = base64::engine::general_purpose::STANDARD
            .decode(DUCK_BASE64)
            .expect("Failed to decode duck base64");
        let image = Image::load_data(&duck_data);
        image
            .dump_png(tests::get_artifact_path().join("duck-texture.png"))
            .expect("Failed to dump image");
        logging::init();
        log::info!("{:?}", image.data::<RGB8>()[0]);
    }
}
