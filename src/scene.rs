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

fn saturate_mediump(x: f32) -> f32 {
    const MEDIUMP_FLT_MAX: f32 = 65504.0;
    x.min(MEDIUMP_FLT_MAX)
}

/// Models the distribution of the microfacet
/// Surfaces are not smooth at the micro level, but made of a
/// large number of randomly aligned planar surface fragments.
/// This implementation is good for half-precision floats.
fn distribution_ggx(n_dot_h: f32, n: &Vec3, h: &Vec3, roughness: f32) -> f32 {
    let n_x_h = n.cross(h);
    let a = n_dot_h * roughness;
    let k = roughness / (n_x_h.dot(&n_x_h) + a * a);
    let d = k * k * (1.0 / std::f32::consts::PI);
    saturate_mediump(d)
}

fn fresnel_schlick(cos_theta: f32, f0: Vec3) -> Vec3 {
    let f = (1.0 - cos_theta).powf(5.0);
    f + f0 * (Vec3::iso(1.0) - f0)
}

/// Models the visibility of the microfacets, or occlusion or shadow-masking
fn geometry_smith_ggx(n_dot_v: f32, n_dot_l: f32, roughness: f32) -> f32 {
    let a = roughness;
    let ggxv = n_dot_l * (n_dot_v * (1.0 - a) + a);
    let ggxl = n_dot_v * (n_dot_l * (1.0 - a) + a);
    0.5 / (ggxv + ggxl)
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
        let default_model = Self::create_default_model();

        Self {
            default_model,
            ..Default::default()
        }
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

        if let Some((hit, triangle)) = bvh.intersects_iter(&ray) {
            let n = triangle.get_normal(&hit);
            let mut pixel_color = Color::black();
            let color = triangle.get_color(&hit);
            let (metallic, roughness) = triangle.get_metallic_roughness(&hit);

            let n_dot_v = n.dot(&ray.dir).abs() + 1e-5;

            const SHADOW_BIAS: f32 = 1e-4;
            // Before getting color, we should check whether it is visible from the sun
            let shadow_origin = hit.point + n * SHADOW_BIAS;

            for light_node in light_nodes {
                let light = lights.get(light_node.light).unwrap();
                let light_dir = light.get_direction(light_node, &hit.point);

                let shadow_ray = Ray::new(shadow_origin, light_dir);
                let shadow_result = bvh.intersects_iter(&shadow_ray);

                // Whether this object is light (verb) by a light (noun)
                let is_light = match shadow_result {
                    None => true,
                    Some((shadow_hit, _)) => {
                        // Distance between current surface and the light source
                        let light_distance = light.get_distance(light_node, &hit.point);
                        // If the obstacle is beyong the light source then the current surface is light
                        shadow_hit.depth > light_distance
                    }
                };

                if is_light {
                    let n_dot_l = n.dot(&light_dir).clamp(0.0, 1.0);

                    // Cook-Torrance approximation of the microfacet model integration
                    let h = (-ray.dir + light_dir).get_normalized();
                    let n_dot_h = n.dot(&h).clamp(0.0, 1.0);
                    let d = distribution_ggx(n_dot_h, &n, &h, roughness);

                    let l_dot_h = light_dir.dot(&h).clamp(0.0, 1.0);
                    let reflectance = 0.5;
                    let f0_value = 0.16 * reflectance * reflectance * (1.0 - metallic);
                    let f0 = Vec3::iso(f0_value) + color * metallic;
                    let f = fresnel_schlick(l_dot_h, f0);

                    let g = geometry_smith_ggx(n_dot_v, n_dot_l, roughness);

                    let fr = (d * g) * Color::from(f);

                    // Lambertian diffuse (1/PI)
                    let fd = color * std::f32::consts::FRAC_1_PI;

                    let light_color = light.get_intensity();
                    let fallof = light.get_fallof(&light_node.trs, &hit.point);
                    pixel_color += ((fd + fr) * light_color * n_dot_l) / fallof;
                }
            } // end iterate light

            let reflection_dir = ray.dir.reflect(&n).get_normalized();
            let reflection_ray = Ray::new(hit.point, reflection_dir);
            if let Some(reflection_color) =
                Self::trace(reflection_ray, bvh, light_nodes, lights, depth + 1)
            {
                // Cosine-law applies here as well
                let n_dot_r = n.dot(&reflection_dir);
                pixel_color += reflection_color * (metallic + 0.5) * n_dot_r;
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
            let node = self.default_model.nodes.get(node).unwrap();
            if let Some(camera) = self.default_model.cameras.get(node.camera) {
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
