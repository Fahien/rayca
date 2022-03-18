// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use super::*;

pub struct Scene {
    pub objects: Vec<Box<dyn Intersect>>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            objects: Default::default(),
        }
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

impl Draw for Scene {
    fn draw(&self, image: &mut Image) {
        let width = image.width();
        let height = image.height();

        let inv_width = 1.0 / width as f32;
        let inv_height = 1.0 / height as f32;

        let fov = 30.0;
        let aspectratio = width as f32 / height as f32;
        let angle = (std::f32::consts::FRAC_PI_2 * fov / 180.0).tan();

        for y in 0..image.height() {
            for x in 0..image.width() {
                // Generate primary ray
                let xx = (2.0 * ((x as f32 + 0.5) * inv_width) - 1.0) * angle * aspectratio;
                let yy = (1.0 - 2.0 * ((y as f32 + 0.5) * inv_height)) * angle;
                let mut dir = Vec3::new(xx, yy, -1.0);
                dir.normalize();
                let ray = Ray::new(Point3::default(), dir);

                for obj in &self.objects {
                    if let Some(hit) = obj.intersects(&ray) {
                        let color = obj.get_color(&hit);
                        image.set(x, y, color.into());
                    }
                }
            }
        }
    }
}
