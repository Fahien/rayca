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

    /// This can be used when there is no light in any model in the scene
    pub default_light_model: Model,
}

impl Scene {
    fn create_default_light_model() -> Model {
        let mut model = Model::new();

        // Add 2 point lights
        let mut light = Light::point();
        light.set_intensity(32.0);
        let light_handle = model.lights.push(light);

        let mut light_node = Node::builder()
            .translation(Vec3::new(-1.0, 2.0, 1.0))
            .light(light_handle)
            .build();
        light_node.light = Some(light_handle);
        model.nodes.push(light_node);

        let point_light_node = Node::builder()
            .light(light_handle)
            .translation(Vec3::new(1.0, 2.0, 1.0))
            .build();
        model.nodes.push(point_light_node);

        model
    }

    pub fn new() -> Self {
        let default_light_model = Self::create_default_light_model();

        Self {
            models: Default::default(),
            default_light_model,
        }
    }

    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        // Open glTF model
        let mut model = Model::builder().path(path)?.build()?;

        // Generate id for model
        model.id = self.models.len();

        // Put model into models
        self.models.push(model);

        Ok(())
    }

    fn draw_pixel(
        &self,
        ray: Ray,
        bvh: &Bvh,
        light_nodes: &[Node],
        lights: &Pack<Light>,
        pixel: &mut RGBA8,
    ) -> usize {
        let mut triangle_count = 0;
        if let Some((hit, triangle)) = bvh.intersects_stats(&ray, &mut triangle_count) {
            let n = triangle.get_normal(&hit);
            let mut pixel_color = Color::black();
            let color = triangle.get_color(&hit);

            const SHADOW_BIAS: f32 = 1e-4;
            // Before getting color, we should check whether it is visible from the sun
            let shadow_origin = hit.point + n * SHADOW_BIAS;

            for light_node in light_nodes {
                let light = lights.get(light_node.light.unwrap()).unwrap();
                let light_dir = light.get_direction(light_node, &hit.point);

                let shadow_ray = Ray::new(shadow_origin, light_dir);
                let shadow_result = bvh.intersects(&shadow_ray);

                let is_light = match shadow_result {
                    None => true,
                    Some((hit, _)) => {
                        let light_distance = light.get_distance(light_node, &hit.point);
                        hit.depth > light_distance
                    }
                };

                if is_light {
                    let n_dot_l = n.dot(&light_dir);

                    pixel_color += color / std::f32::consts::PI
                        * light.get_intensity(light_node, &hit.point)
                        * n_dot_l.max(0.0);
                }
            }

            // No over operation here as transparency should be handled by the lighting model
            *pixel = pixel_color.into();
        }
        triangle_count
    }
}

impl Draw for Scene {
    fn draw(&self, image: &mut Image) {
        let mut triangles = vec![];
        let mut cameras = vec![];

        let mut timer = Timer::new();

        for model in self.models.iter() {
            let transforms = model.collect_transforms();
            for (node, trs) in transforms {
                // Collect triangles
                if let Some(mesh) = model.meshes.get(node.mesh) {
                    for prim_handle in mesh.primitives.iter() {
                        let prim = model.primitives.get(*prim_handle).unwrap();
                        let mut prim_triangles = prim.triangles(&trs, prim.material, model);
                        triangles.append(&mut prim_triangles);
                    }
                }

                // Collect cameras
                if let Some(camera_handle) = node.camera {
                    let camera = model.cameras.get(camera_handle).unwrap();
                    cameras.push((camera, trs));
                }
            }
        }

        let bvh = Bvh::new(triangles);

        let width = image.width() as f32;
        let height = image.height() as f32;

        let inv_width = 1.0 / width;
        let inv_height = 1.0 / height;

        let mut fov = 0.7;

        let camera_trs = if cameras.len() > 0 {
            let (camera, camera_trs) = &cameras[0];
            fov = camera.yfov;
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

        // TODO collect lights from models
        let light_nodes = self.default_light_model.nodes.as_slice();
        let lights = &self.default_light_model.lights;

        #[cfg(feature = "parallel")]
        let pixel_iter = image.pixels_mut().into_par_iter();
        #[cfg(not(feature = "parallel"))]
        let pixel_iter = image.pixels_mut().into_iter();

        pixel_iter.enumerate().for_each(|(y, mut row)| {
            for x in 0..row.len() {
                // Generate primary ray
                let xx = (2.0 * ((x as f32 + 0.5) * inv_width) - 1.0) * angle * aspectratio;
                let yy = (1.0 - 2.0 * ((y as f32 + 0.5) * inv_height)) * angle;
                let mut dir = Vec3::new(xx, yy, -1.0);
                dir.normalize();
                let origin = Vec3::new(0.0, 0.0, 0.0);
                let ray = &camera_trs * Ray::new(origin, dir);

                self.draw_pixel(ray, &bvh, light_nodes, lights, row[x]);
            }
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
