// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::{error::Error, path::Path};

use owo_colors::OwoColorize;

#[cfg(feature = "parallel")]
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};

use super::*;

pub struct Scene {
    pub models: Vec<Model>,
    /// This can be used for default values which are not defined in any other model in the scene
    pub default_model: Model,

    pub config: Config,
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene {
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
        light_node.light = Some(light_handle);
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
        let default_model = Self::create_default_model();

        Self {
            models: Default::default(),
            default_model,
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
        let mut model = Model::builder().path(path)?.build()?;

        // Generate id for model
        model.id = self.models.len();

        // Put model into models
        self.models.push(model);

        print_info!(
            "Loaded",
            "{} in {:.2}ms",
            path_str,
            timer.get_delta().as_millis()
        );
        Ok(())
    }

    fn draw_pixel(&self, ray: Ray, bvh: &Bvh, lights: &[BvhLight], pixel: &mut RGBA8) -> usize {
        let triangle_count = 0;
        if let Some(pixel_color) = self.config.integrator.trace(ray, bvh, lights, 0) {
            // No over operation here as transparency should be handled by the lighting model
            *pixel = pixel_color.into();
        }
        triangle_count
    }

    fn collect_trs(&self) -> Vec<SolvedTrs> {
        let mut solved_trs = Vec::new();
        for model in self.models.iter() {
            solved_trs.extend(model.collect_trs());
        }
        solved_trs.extend(self.default_model.collect_trs());
        solved_trs
    }

    fn collect_primitives<'m>(&'m self, solved_trs: &'m Vec<SolvedTrs<'m>>) -> Vec<BvhPrimitive> {
        let mut primitives = vec![];
        for trs in solved_trs {
            primitives.extend(trs.collect_primitives());
        }
        primitives
    }

    fn collect_cameras<'m>(
        &'m self,
        solved_trs: &'m Vec<SolvedTrs<'m>>,
    ) -> Vec<(&'m Camera, &'m Trs)> {
        let mut cameras = vec![];
        for trs in solved_trs {
            cameras.extend(trs.collect_cameras());
        }
        cameras
    }

    fn collect_lights<'m>(&'m self, solved_trs: &'m Vec<SolvedTrs<'m>>) -> Vec<BvhLight> {
        let mut lights = vec![];
        for trs in solved_trs {
            lights.extend(trs.collect_lights())
        }
        lights
    }
}

impl Draw for Scene {
    fn draw(&self, image: &mut Image) {
        // Collect transforms, triangles, cameras, and lights from our models
        let solved_trs = self.collect_trs();
        let primitives = self.collect_primitives(&solved_trs);
        let mut cameras = self.collect_cameras(&solved_trs);
        let mut lights = self.collect_lights(&solved_trs);

        // Collect defaults
        let default_solved_trs = self.default_model.collect_trs();
        let default_cameras = self.collect_cameras(&default_solved_trs);
        let default_lights = self.collect_lights(&default_solved_trs);
        if cameras.is_empty() {
            cameras.extend(default_cameras);
        }
        if lights.is_empty() {
            lights.extend(default_lights);
        }
        print_info!("Lights:", "{}", lights.len());

        // Build BVH
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

        assert!(!cameras.is_empty());
        let (camera, camera_trs) = cameras[0];

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
                let ray = camera_trs * Ray::new(origin, dir);

                self.draw_pixel(ray, &bvh, &lights, pixel);
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
