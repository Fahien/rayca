// Copyright © 2022-2024
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::{error::Error, path::Path};

use owo_colors::OwoColorize;

#[cfg(feature = "parallel")]
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};

use super::*;

#[derive(Default)]
pub struct Scene {
    /// This can be used for default values which are not defined in any other model in the scene
    pub default_model: Model,
    // Single model collecting elements from all loaded models
    pub model: Model,
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

        model
    }

    pub fn new() -> Self {
        let default_model = Self::create_default_model();

        Self {
            default_model,
            ..Default::default()
        }
    }

    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        // Open glTF model
        let model = Model::builder().path(path)?.build()?;
        self.model.append(model);
        Ok(())
    }

    pub fn push(&mut self, model: Model) {
        self.model.append(model);
    }

    fn draw_pixel(&self, ray: Ray, bvh: &Bvh, pixel: &mut RGBA8) -> usize {
        let mut triangle_count = 0;
        if let Some((hit, triangle)) = bvh.intersects_stats(&ray, &mut triangle_count) {
            let mut color = triangle.get_color(&hit);

            // Facing ratio
            let n = triangle.get_normal(&hit);
            let n_dot_dir = n.dot(&-ray.dir);
            color.r *= n_dot_dir;
            color.g *= n_dot_dir;
            color.b *= n_dot_dir;

            // No over operation here as transparency should be handled by the lighting model
            *pixel = color.into();
        }
        triangle_count
    }
}

impl Draw for Scene {
    fn draw(&self, image: &mut Image) {
        let mut triangles = vec![];
        let mut cameras = vec![];

        let mut timer = Timer::new();

        let transforms = self.model.collect_transforms();
        for (node, trs) in transforms {
            // Collect triangles
            let node = self.model.nodes.get(node).unwrap();
            if let Some(mesh) = self.model.meshes.get(node.mesh) {
                for prim_handle in mesh.primitives.iter() {
                    let prim = self.model.primitives.get(*prim_handle).unwrap();
                    let mut prim_triangles = prim.triangles(&trs, prim.material, &self.model);
                    triangles.append(&mut prim_triangles);
                }
            }

            // Collect cameras
            if let Some(camera) = self.model.cameras.get(node.camera) {
                cameras.push((camera, trs));
            }
        }

        let default_transforms = self.default_model.collect_transforms();
        for (node, trs) in default_transforms {
            // Collect cameras
            if let Some(camera) = self.default_model.cameras.get(node.camera) {
                cameras.push((camera, trs));
            }
        }

        let bvh = Bvh::new(triangles);

        let width = image.width() as f32;
        let height = image.height() as f32;

        let inv_width = 1.0 / width;
        let inv_height = 1.0 / height;

        assert!(!cameras.is_empty());
        let (camera, camera_trs) = &cameras[0];

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
