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
    // Single model collecting elements from all loaded models
    pub model: Model,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            model: Default::default(),
        }
    }

    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        // Open glTF model
        let model = Model::builder(path)?.build()?;
        self.model.append(model);
        Ok(())
    }

    pub fn push(&mut self, model: Model) {
        self.model.append(model);
    }

    fn draw_pixel(&self, ray: Ray, triangles: &[Triangle], pixel: &mut RGBA8) {
        let mut depth = f32::INFINITY;

        for triangle in triangles {
            if let Some(hit) = triangle.intersects(&ray) {
                if hit.depth < depth {
                    depth = hit.depth;
                    let mut color = triangle.get_color(&hit);

                    // Facing ratio
                    let n = triangle.get_normal(&hit);
                    let n_dot_dir = n.dot(&-ray.dir);
                    color.r *= n_dot_dir;
                    color.g *= n_dot_dir;
                    color.b *= n_dot_dir;

                    pixel.over(color);
                }
            }
        }
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

        let width = image.width() as f32;
        let height = image.height() as f32;

        let inv_width = 1.0 / width;
        let inv_height = 1.0 / height;

        let mut fov = 0.7;

        let camera_trs = if !cameras.is_empty() {
            let (camera, camera_trs) = &cameras[0];
            fov = camera.yfov_radians;
            camera_trs.clone()
        } else {
            Trs::new(
                Vec3::new(0.0, 0.0, 4.0),
                Quat::default(),
                Vec3::new(1.0, 1.0, 1.0),
            )
        };

        let aspectratio = width / height;
        let angle = (fov * 0.5).tan();

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
                let ray = &camera_trs * Ray::new(origin, dir);

                self.draw_pixel(ray, &triangles, pixel);
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
