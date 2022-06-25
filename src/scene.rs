// Copyright © 2022-2024
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::{error::Error, path::Path};

use owo_colors::OwoColorize;

#[cfg(feature = "parallel")]
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};

use super::*;

pub struct Scene {
    // Single model collecting elements from all loaded models
    pub model: Model,

    pub config: Config,
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene {
    /// This can be used for default values which are not defined in any other model in the scene
    fn create_default_model() -> Model {
        let mut model = Model::new();

        // Add 1 camera
        let camera = Camera::default();
        let camera_handle = model.cameras.push(camera);
        let camera_node = Node::builder()
            .camera(camera_handle)
            .translation(Vec3::new(0.0, 0.0, 4.0))
            .build();
        let camera_node_handle = model.nodes.push(camera_node);
        model.root.children.push(camera_node_handle);

        // Add 2 point lights
        let mut light = Light::point();
        light.set_intensity(64.0);
        let light_handle = model.lights.push(light);

        let mut light_node = Node::builder()
            .translation(Vec3::new(-1.0, 2.0, 1.0))
            .light(light_handle)
            .build();
        light_node.light = light_handle;
        let light_node_handle = model.nodes.push(light_node);
        model.root.children.push(light_node_handle);

        let point_light_node = Node::builder()
            .light(light_handle)
            .translation(Vec3::new(1.0, 2.0, 1.0))
            .build();
        let light_node_handle = model.nodes.push(point_light_node);
        model.root.children.push(light_node_handle);

        model
    }

    pub fn new() -> Self {
        Self {
            model: Default::default(),
            config: Default::default(),
        }
    }

    pub fn new_with_config(config: Config) -> Self {
        let mut scene = Self::new();
        scene.config = config;
        scene
    }

    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let mut timer = Timer::new();
        let path_str = path.as_ref().to_string_lossy().to_string();

        // Open glTF model
        let model = Model::builder().path(path)?.build()?;
        self.model.append(model);

        print_info!(
            "Loaded",
            "{} in {:.2}ms",
            path_str,
            timer.get_delta().as_millis()
        );
        Ok(())
    }

    pub fn push(&mut self, model: Model) {
        self.model.append(model);
    }

    pub fn push_default_model(&mut self) {
        self.model.append(Self::create_default_model())
    }

    fn draw_pixel(&self, ray: Ray, bvh: &Bvh, pixel: &mut RGBA8) -> usize {
        let triangle_count = 0;
        if let Some(pixel_color) = self.config.integrator.trace(&self.model, ray, bvh, 0) {
            // No over operation here as transparency should be handled by the lighting model
            *pixel = pixel_color.into();
        }
        triangle_count
    }
}

impl Draw for Scene {
    fn draw(&mut self, image: &mut Image) {
        let primitives = self.model.collect();

        let mut bvh_builder = Bvh::builder().primitives(primitives);
        if !self.config.bvh {
            bvh_builder = bvh_builder.max_depth(0);
        }
        let bvh = bvh_builder.build();

        let mut timer = Timer::new();

        let width = image.width() as f32;
        let height = image.height() as f32;

        let inv_width = 1.0 / width;
        let inv_height = 1.0 / height;

        assert!(!self.model.camera_nodes.is_empty());
        let camera_node_handle = self.model.camera_nodes[0];
        let camera_node = self.model.nodes.get(self.model.camera_nodes[0]).unwrap();
        let camera = self.model.cameras.get(camera_node.camera).unwrap();
        let camera_trs = self.model.solved_trs.get(&camera_node_handle).unwrap();

        let aspectratio = width / height;
        let angle = camera.get_angle();

        #[cfg(feature = "parallel")]
        let row_iter = image.pixels_mut().into_par_iter();
        #[cfg(not(feature = "parallel"))]
        let row_iter = image.pixels_mut().into_iter();

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
                let ray = &camera_trs.trs * Ray::new(origin, dir);

                self.draw_pixel(ray, &bvh, pixel);
            });
        });

        rlog!(
            "{:>12} in {:.2}ms",
            "Rendered".green().bold(),
            timer.get_delta().as_millis()
        );
    }
}

#[cfg(test)]
mod test {
    use owo_colors::{OwoColorize, Stream::Stdout};

    use super::*;

    #[test]
    fn load() {
        let mut scene = Scene::new();
        assert!(scene.load("test").is_err());

        let path = "tests/model/box/box.gltf";
        match scene.load(path) {
            Ok(_) => (),
            Err(err) => {
                panic!(
                    "{}: Failed to load \"{}\": {}",
                    "ERROR".if_supports_color(Stdout, |text| text.red()),
                    path,
                    err
                );
            }
        };
    }
}
