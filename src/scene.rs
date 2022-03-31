// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::{error::Error, path::Path};

use owo_colors::OwoColorize;
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};

use super::*;

pub struct Scene<'a> {
    pub objects: Vec<Box<dyn Intersect + Send + Sync + 'a>>,
    pub models: Vec<Model>,
}

impl<'a> Scene<'a> {
    pub fn new() -> Self {
        Self {
            objects: Default::default(),
            models: Default::default(),
        }
    }

    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        // Open glTF model
        let mut model = Model::builder(path)?.build()?;

        // Generate id for model
        model.id = self.models.len();

        // Put model into models
        self.models.push(model);

        Ok(())
    }

    fn draw_pixel(&self, ray: Ray, triangles: &[Triangle<Vertex>], pixel: &mut RGBA8) {
        let mut depth = f32::INFINITY;

        for obj in &self.objects {
            if let Some(hit) = obj.intersects(&ray) {
                if hit.depth < depth {
                    depth = hit.depth;
                    let color = obj.get_color(&hit);
                    *pixel = color;
                }
            }
        }

        for triangle in triangles {
            if let Some(hit) = triangle.intersects(&ray) {
                if hit.depth < depth {
                    depth = hit.depth;
                    let mut color = triangle.get_color(&hit);

                    // Facing ratio
                    let n = triangle.get_normal(&hit);
                    let n_dot_dir = n.dot(&-ray.dir);
                    color.r = ((color.r as f32) * n_dot_dir) as u8;
                    color.g = ((color.g as f32) * n_dot_dir) as u8;
                    color.b = ((color.b as f32) * n_dot_dir) as u8;

                    *pixel = color;
                }
            }
        }
    }
}

impl<'a> Draw for Scene<'a> {
    fn draw(&self, image: &mut Image) {
        let mut triangles = vec![];

        let mut timer = Timer::new();

        for model in &self.models {
            for node in model.nodes.iter() {
                if let Some(mesh) = model.meshes.get(node.mesh) {
                    for prim_handle in mesh.primitives.iter() {
                        let prim = model.primitives.get(*prim_handle).unwrap();
                        let mut prim_triangles = prim.triangles(&node.trs, prim.material, model);
                        triangles.append(&mut prim_triangles);
                    }
                }
            }
        }
        println!(
            "{:>12} {} triangles in {:.2}s",
            "Collected".green().bold(),
            triangles.len(),
            timer.get_delta().as_secs_f32()
        );

        let width = image.width() as f32;
        let height = image.height() as f32;

        let inv_width = 1.0 / width;
        let inv_height = 1.0 / height;

        let fov = 30.0;
        let aspectratio = width / height;
        let angle = (std::f32::consts::FRAC_PI_2 * fov / 180.0).tan();

        image
            .pixels_mut()
            .into_par_iter()
            .enumerate()
            .for_each(|(y, mut row)| {
                for x in 0..row.len() {
                    // Generate primary ray
                    let xx = (2.0 * ((x as f32 + 0.5) * inv_width) - 1.0) * angle * aspectratio;
                    let yy = (1.0 - 2.0 * ((y as f32 + 0.5) * inv_height)) * angle;
                    let mut dir = Vec3::new(xx, yy, -1.0);
                    dir.normalize();
                    let origin = Vec3::new(0.0, 0.0, 4.5);
                    let ray = Ray::new(origin, dir);

                    self.draw_pixel(ray, &triangles, row[x]);
                }
            });
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
