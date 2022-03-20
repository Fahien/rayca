// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::{error::Error, path::Path};

use super::*;

pub struct Scene {
    pub objects: Vec<Box<dyn Intersect>>,
    // Single model collecting elements from all loaded models
    pub model: Model,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            objects: Default::default(),
            model: Default::default(),
        }
    }

    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        // Open glTF model
        let model = Model::builder(path)?.build()?;
        self.model.append(model);
        Ok(())
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
                let origin = Point3::new(0.0, 0.0, 4.0);
                let ray = Ray::new(origin, dir);

                let mut depth = f32::INFINITY;

                for obj in &self.objects {
                    if let Some(hit) = obj.intersects(&ray) {
                        if hit.depth < depth {
                            depth = hit.depth;
                            let color = obj.get_color(&hit);
                            image.set(x, y, color.into());
                        }
                    }
                }

                for node in self.model.nodes.iter() {
                    if let Some(mesh) = self.model.meshes.get(node.mesh) {
                        for prim_handle in mesh.primitives.iter() {
                            let prim = self.model.primitives.get(*prim_handle).unwrap();
                            for triangle in prim.triangles(&node.trs) {
                                if let Some(hit) = triangle.intersects(&ray) {
                                    if hit.depth < depth {
                                        depth = hit.depth;
                                        //let color = triangle.get_color(&hit);
                                        let n = triangle.get_normal(&hit);
                                        let color = Color::from(n);
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
