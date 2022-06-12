// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::{error::Error, path::Path};

use owo_colors::OwoColorize;

#[cfg(feature = "parallel")]
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};

use super::*;

#[derive(Default)]
pub struct Scene {
    pub models: Vec<Model>,
    /// This can be used for default values which are not defined in any other model in the scene
    pub default_model: Model,
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
        light.set_intensity(32.0);
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
        }
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

        rlog!(
            "{:>12} {} in {:.2}ms",
            "Loaded".cyan().bold(),
            path_str,
            timer.get_delta().as_millis()
        );

        Ok(())
    }

    fn trace(
        ray: Ray,
        bvh: &Bvh,
        light_nodes: &[Node],
        lights: &Pack<Light>,
        depth: u32,
    ) -> Option<Color> {
        if depth > 1 {
            return None;
        }

        if let Some(hit) = bvh.intersects_iter(&ray) {
            let n = hit.primitive.get_normal(&hit);
            let mut pixel_color = Color::black();
            let color = hit.primitive.get_color(&hit);

            const SHADOW_BIAS: f32 = 1e-4;
            // Before getting color, we should check whether it is visible from the sun
            let shadow_origin = hit.point + n * SHADOW_BIAS;

            for light_node in light_nodes {
                let light = lights.get(light_node.light.unwrap()).unwrap();
                let light_dir = light.get_direction(light_node, &hit.point);

                let shadow_ray = Ray::new(shadow_origin, light_dir);
                let shadow_result = bvh.intersects_iter(&shadow_ray);

                let is_light = match shadow_result {
                    None => true,
                    Some(shadow_hit) => {
                        // Distance between current surface and the light source
                        let light_distance = light.get_distance(light_node, &hit.point);
                        // If the obstacle is beyong the light source then the current surface is light
                        shadow_hit.depth > light_distance
                    }
                };

                if is_light {
                    let n_dot_l = n.dot(&light_dir).clamp(0.0, 1.0);
                    let fallof = light.get_fallof(&light_node.trs, &hit.point);
                    pixel_color +=
                        (color / std::f32::consts::PI * light.get_intensity() * n_dot_l) / fallof;
                }
            }

            let metalness = hit.primitive.get_metalness(&hit);
            if metalness > 0.0 {
                let reflection_dir = ray.dir.reflect(&n);
                let reflection_ray = Ray::new(hit.point, reflection_dir);
                if let Some(reflection_color) =
                    Self::trace(reflection_ray, bvh, light_nodes, lights, depth + 1)
                {
                    pixel_color += reflection_color * metalness;
                }
            }

            return Some(pixel_color);
        }

        None
    }

    fn draw_pixel(
        &self,
        ray: Ray,
        bvh: &Bvh,
        light_nodes: &[Node],
        lights: &Pack<Light>,
        pixel: &mut RGBA8,
    ) -> usize {
        let triangle_count = 0;
        if let Some(pixel_color) = Self::trace(ray, bvh, light_nodes, lights, 0) {
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

        for model in self.models.iter() {
            let transforms = model.collect_transforms();
            for (node, trs) in transforms {
                // Collect triangles
                if let Some(mesh_handle) = node.mesh {
                    let mesh = model.meshes.get(mesh_handle).unwrap();
                    for prim_handle in mesh.primitives.iter() {
                        let prim = model.primitives.get(*prim_handle).unwrap();
                        let mut prim_triangles = prim.triangles(&trs, model);
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

        let default_transforms = self.default_model.collect_transforms();
        for (node, trs) in default_transforms {
            // Collect cameras
            if let Some(camera_handle) = node.camera {
                let camera = self.default_model.cameras.get(camera_handle).unwrap();
                cameras.push((camera, trs));
            }
        }

        let bvh = Bvh::new(triangles);

        let mut timer = Timer::new();

        let width = image.width() as f32;
        let height = image.height() as f32;

        let inv_width = 1.0 / width;
        let inv_height = 1.0 / height;

        assert!(!cameras.is_empty());
        let (camera, camera_trs) = &cameras[0];

        let aspectratio = width / height;
        let angle = camera.get_angle();

        // TODO collect lights from models
        let light_nodes = &self.default_model.nodes[1..];
        let lights = &self.default_model.lights;

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

                self.draw_pixel(ray, &bvh, light_nodes, lights, pixel);
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
