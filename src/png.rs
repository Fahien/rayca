// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use super::image::*;

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

pub fn dump<P: AsRef<Path>>(image: &Image, path: P) {
    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, image.width() as u32, image.height() as u32);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_trns(vec![0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8]);
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

    writer.write_image_data(image.bytes()).unwrap(); // Save
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn dump() {
        let image = Image::new(32, 32);
        super::dump(&image, "target/image.png");
    }
}
