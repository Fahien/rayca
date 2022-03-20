// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::{error::Error, path::Path};

use super::*;

pub struct Scene {
    pub objects: Vec<Box<dyn Intersect>>,
    pub models: Vec<Model>,
}

impl Scene {
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
                let origin = Vec3::new(0.0, 0.0, 3.0);
                let ray = Ray::new(origin, dir);

                for obj in &self.objects {
                    if let Some(hit) = obj.intersects(&ray) {
                        let color = obj.get_color(&hit);
                        image.set(x, y, color.into());
                    }
                }

                for model in &self.models {
                    for node in model.nodes.iter() {
                        if let Some(mesh) = model.meshes.get(node.mesh) {
                            for prim_handle in mesh.primitives.iter() {
                                let prim = model.primitives.get(*prim_handle).unwrap();
                                for triangle in prim.triangles(Mat4::from(&node.trs)) {
                                    if let Some(hit) = triangle.intersects(&ray) {
                                        let color = triangle.get_color(&hit);
                                        image.set(x, y, color.into());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
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
