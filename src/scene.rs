// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::{error::Error, path::Path};

use owo_colors::OwoColorize;

#[cfg(feature = "parallel")]
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};

use super::*;

#[derive(PartialEq)]
pub enum Lighting {
    Flat,
    Positions,
    Normals,
    Facing,
    Pbr,
}

pub struct Config {
    pub lighting: Lighting,
    pub bvh: bool,
}

impl Config {
    pub fn new(lighting: Lighting, bvh: bool) -> Self {
        Self { lighting, bvh }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            lighting: Lighting::Pbr,
            bvh: true,
        }
    }
}

pub struct Scene {
    pub models: Vec<Model>,
    /// This can be used for default values which are not defined in any other model in the scene
    pub default_model: Model,

    pub config: Config,
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

    fn trace(&self, ray: Ray, bvh: &Bvh, lights: &[(&Light, &Trs)], depth: u32) -> Option<Color> {
        if depth > 1 {
            return None;
        }

        if let Some((hit, primitive)) = bvh.intersects_iter(&ray) {
            if self.config.lighting == Lighting::Positions {
                return Some(hit.point.into());
            }

            let n = primitive.get_normal(&hit);
            if self.config.lighting == Lighting::Normals {
                return Some(n.into());
            }

            const SMALL_BIAS: f32 = 1e-6;
            let n_dot_v = n.dot(&-ray.dir).abs() + SMALL_BIAS;

            if self.config.lighting == Lighting::Facing {
                return Some(Color::new(n_dot_v, n_dot_v, n_dot_v, 1.0));
            }

            let albedo_color = primitive.get_color(&hit);
            if self.config.lighting == Lighting::Flat {
                return Some(albedo_color);
            }

            // Ambient?
            let mut pixel_color = Color::black() + albedo_color / 8.0;
            const RAY_BIAS: f32 = 1e-3;

            if albedo_color.a < 1.0 {
                let transmit_origin = hit.point + -n * RAY_BIAS;
                let transmit_ray = Ray::new(transmit_origin, ray.dir);
                let transmit_result = self.trace(transmit_ray, bvh, lights, depth + 1);

                if let Some(mut transmit_color) = transmit_result {
                    // continue with the rest of the shading?
                    transmit_color.over(albedo_color);
                    pixel_color += transmit_color;
                }
            }

            // Before getting color, we should check whether it is visible from the sun
            let next_origin = hit.point + n * RAY_BIAS;

            let uv = primitive.geometry.get_uv(&hit);

            for (light, light_trs) in lights {
                let light_dir = light.get_direction(light_trs, &hit.point);

                let shadow_ray = Ray::new(next_origin, light_dir);
                let shadow_result = bvh.intersects_iter(&shadow_ray);

                // Whether this object is light (verb) by a light (noun)
                let is_light = match shadow_result {
                    None => true,
                    Some((shadow_hit, primitive)) => {
                        // Distance between current surface and the light source
                        let light_distance = light.get_distance(light_trs, &hit.point);
                        // If the obstacle is beyong the light source then the current surface is light
                        if shadow_hit.depth > light_distance {
                            true
                        } else {
                            // Check whether the obstacle is a transparent surface
                            let shadow_color = primitive.get_color(&shadow_hit);
                            shadow_color.a < 1.0
                        }
                    }
                };

                if is_light {
                    let incident_light = LightIntersection::new(
                        light_trs,
                        light,
                        &hit,
                        light_dir,
                        n,
                        -ray.dir,
                        n_dot_v,
                        albedo_color,
                        uv,
                    );

                    pixel_color += primitive.get_radiance(&incident_light);
                }
            } // end iterate light

            let n = primitive.geometry.get_normal(&hit);
            let reflection_dir = ray.dir.reflect(&n).get_normalized();
            let reflection_ray = Ray::new(next_origin, reflection_dir);
            if let Some(reflection_color) = self.trace(reflection_ray, bvh, lights, depth + 1) {
                // Cosine-law applies here as well
                let n_dot_r = 1.0; // How about fresnel reflecting more at grazing angles? n.dot(&reflection_dir);
                let (metallic, roughness) = primitive
                    .get_material()
                    .get_metallic_roughness(&uv, primitive.model);
                pixel_color += reflection_color * n_dot_r * metallic * (1.0 - roughness).clamp(0.25, 1.0);
            }

            return Some(pixel_color);
        }

        None
    }

    fn draw_pixel(
        &self,
        ray: Ray,
        bvh: &Bvh,
        lights: &[(&Light, &Trs)],
        pixel: &mut RGBA8,
    ) -> usize {
        let triangle_count = 0;
        if let Some(pixel_color) = self.trace(ray, bvh, lights, 0) {
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

    fn collect<'m>(
        &'m self,
        solved_trs: &'m Vec<SolvedTrs<'m>>,
    ) -> (
        Vec<BvhPrimitive>,
        Vec<(&'m Camera, &'m Trs)>,
        Vec<(&'m Light, &'m Trs)>,
    ) {
        let mut triangles = vec![];
        let mut cameras = vec![];
        let mut lights = vec![];

        for trs in solved_trs {
            let (curr_triangles, curr_cameras, curr_lights) = trs.collect();
            triangles.extend(curr_triangles);
            cameras.extend(curr_cameras);
            lights.extend(curr_lights)
        }

        (triangles, cameras, lights)
    }
}

impl Draw for Scene {
    fn draw(&self, image: &mut Image) {
        // Collect transforms, triangles, cameras, and lights from our models
        let solved_trs = self.collect_trs();
        let (primitives, mut cameras, mut lights) = self.collect(&solved_trs);

        // Collect defaults
        let default_solved_trs = self.default_model.collect_trs();
        let (_, default_cameras, default_lights) = self.collect(&default_solved_trs);
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

        assert!(cameras.len() > 0);
        let (camera, camera_trs) = cameras[0];

        let aspectratio = width / height;
        let angle = camera.get_angle();

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
                let ray = camera_trs * Ray::new(origin, dir);

                self.draw_pixel(ray, &bvh, &lights, row[x]);
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
