// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use super::*;

pub struct Scene {
    objects: Vec<Sphere>,
}

impl Scene {
    pub fn new() -> Self {
        let objects = vec![Sphere::new(Vec3::new(0.0, 0.0, -1.0), 0.1, 0xFF0000FFu32)];
        Self { objects }
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
                let ray = Ray::new(Vec3::default(), dir);

                for obj in &self.objects {
                    if obj.intersects(&ray) {
                        image.set(x, y, obj.color);
                    }
                }
            }
        }
    }
}
