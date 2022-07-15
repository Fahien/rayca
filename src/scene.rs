// Copyright © 2022-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use owo_colors::OwoColorize;

#[cfg(feature = "parallel")]
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};

use super::*;

#[derive(Clone)]
pub struct SolvedCamera {
    pub camera: Camera,
    pub trs: Trs,
}

impl Default for SolvedCamera {
    fn default() -> Self {
        Self {
            camera: Camera::default(),
            trs: Trs::new(
                Vec3::new(0.0, 0.0, 4.0),
                Quat::default(),
                Vec3::new(1.0, 1.0, 1.0),
            ),
        }
    }
}

impl SolvedCamera {
    pub fn new(camera: Camera, trs: Trs) -> Self {
        Self { camera, trs }
    }
}

#[derive(Clone)]
pub struct SolvedLight {
    pub light: Light,
    pub trs: Trs,
}

impl SolvedLight {
    pub fn new(light: Light, trs: Trs) -> Self {
        Self { light, trs }
    }
}

pub struct DefaultLights {
    pub lights: Vec<SolvedLight>,
}

impl Default for DefaultLights {
    fn default() -> Self {
        let mut lights = Vec::new();

        // Add 2 point lights
        let mut light = Light::point();
        light.scale_intensity(64.0);

        let mut trs = Trs {
            translation: Vec3::new(-1.0, 2.0, 1.0),
            ..Default::default()
        };
        lights.push(SolvedLight::new(light.clone(), trs.clone()));

        trs.translation = Vec3::new(1.0, 2.0, 1.0);
        lights.push(SolvedLight::new(light, trs));

        Self { lights }
    }
}

#[derive(Default)]
pub struct Scene {
    pub config: Config,

    pub tlas: Tlas,
    pub gltf_models: Pack<GltfModel>,

    /// This can be used for default values which are not defined in any other model in the scene
    pub default_camera: SolvedCamera,
    pub default_lights: DefaultLights,
}

impl Scene {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    fn draw_pixel(&self, ray: Ray, pixel: &mut RGBA8) {
        let color = self.config.integrator.trace(self, ray, 0);
        // No over operation here as transparency should be handled by the lighting mode
        *pixel = color.into();
    }

    pub fn update(&mut self) {
        let bvh_max_depth = if self.config.bvh { u8::MAX } else { 1 };
        self.tlas.max_depth = bvh_max_depth;
        self.tlas.replace_models(&self.gltf_models);

        let solved_cameras = &self.tlas.bvhs[0].cameras;
        if !solved_cameras.is_empty() {
            self.default_camera = solved_cameras.last().unwrap().clone();
        }

        let solved_lights = &self.tlas.bvhs[0].lights;
        if !solved_lights.is_empty() {
            self.default_lights.lights = solved_lights.clone();
        }
    }
}

impl Draw for Scene {
    fn draw(&self, image: &mut Image) {
        let width = image.width() as f32;
        let height = image.height() as f32;

        let inv_width = 1.0 / width;
        let inv_height = 1.0 / height;

        let aspectratio = width / height;
        let angle = self.default_camera.camera.get_angle();

        #[cfg(feature = "parallel")]
        let row_iter = image.pixels_mut().into_par_iter();
        #[cfg(not(feature = "parallel"))]
        let row_iter = image.pixels_mut().into_iter();

        let mut timer = Timer::new();

        row_iter.enumerate().for_each(|(y, row)| {
            #[cfg(feature = "parallel")]
            let pixel_iter = row.into_par_iter();
            #[cfg(not(feature = "parallel"))]
            let pixel_iter = row.into_iter();

            pixel_iter.enumerate().for_each(|(x, pixel)| {
                // Generate primary ray
                let xx = (2.0 * ((x as f32 + 0.5) * inv_width) - 1.0) * angle * aspectratio;
                let yy = (1.0 - 2.0 * ((y as f32 + 0.5) * inv_height)) * angle;
                let mut dir = Vec3::new(xx, yy, -1.0);
                dir.normalize();
                let origin = Point3::new(0.0, 0.0, 0.0);
                let ray = &self.default_camera.trs * Ray::new(origin, dir);

                self.draw_pixel(ray, pixel);
            });
        });

        rlog!(
            "{:>12} in {:.2}ms",
            "Rendered".green().bold(),
            timer.get_delta().as_millis()
        );
    }
}
