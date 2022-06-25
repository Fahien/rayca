// Copyright © 2022-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use owo_colors::OwoColorize;

#[cfg(feature = "parallel")]
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};

use super::*;

pub struct DefaultCamera {
    pub camera: Camera,
    pub trs: Trs,
}

impl Default for DefaultCamera {
    fn default() -> Self {
        Self {
            camera: Camera::default(),
            trs: Trs::new(
                Vec3::new(0.0, 0.0, 4.0),
                Quat::default(),
                Vec3::new(1.0, 1.0, 1.0),
            ),
        }
    }
}

#[derive(Default)]
pub struct Scene {
    pub gltf_model: GltfModel,

    /// This can be used for default values which are not defined in any other model in the scene
    pub default_camera: DefaultCamera,
}

impl Scene {
    pub fn new() -> Self {
        Self::default()
    }

    fn draw_pixel_triangle(&self, ray: Ray, bvh: &Bvh, hit: &Hit, pixel: &mut RGBA8) {
        let triangle_ex = &bvh.triangles_ex[hit.primitive as usize];
        let mut color = triangle_ex.get_color(hit, self);

        // Facing ratio
        let n = triangle_ex.get_normal(hit);
        let n_dot_dir = n.dot(&-ray.dir);
        color.r *= n_dot_dir;
        color.g *= n_dot_dir;
        color.b *= n_dot_dir;

        // No over operation here as transparency should be handled by the lighting model
        *pixel = color.into();
    }

    fn draw_pixel_sphere(&self, ray: Ray, bvh: &Bvh, hit: &Hit, pixel: &mut RGBA8) {
        assert!(hit.primitive as usize >= bvh.triangles.len());
        let sphere_index = hit.primitive as usize - bvh.triangles.len();
        let sphere = &bvh.spheres[sphere_index];
        let sphere_ex = &bvh.spheres_ex[sphere_index];
        let mut color = sphere_ex.get_color(sphere, hit);

        // Facing ratio
        let n = sphere_ex.get_normal(sphere, hit);
        let n_dot_dir = n.dot(&-ray.dir);
        color.r *= n_dot_dir;
        color.g *= n_dot_dir;
        color.b *= n_dot_dir;

        // No over operation here as transparency should be handled by the lighting model
        *pixel = color.into();
    }

    fn draw_pixel_bvh(&self, ray: Ray, bvh: &Bvh, pixel: &mut RGBA8) -> usize {
        let mut triangle_count = 0;
        if let Some(hit) = bvh.intersects_stats(&ray, &mut triangle_count) {
            let primitive_index = hit.primitive as usize;
            if primitive_index < bvh.triangles.len() {
                self.draw_pixel_triangle(ray, bvh, &hit, pixel);
            } else if primitive_index - bvh.triangles.len() < bvh.spheres.len() {
                self.draw_pixel_sphere(ray, bvh, &hit, pixel);
            }
        }
        triangle_count
    }

    fn collect_triangles(&self) -> (Vec<Triangle>, Vec<TriangleEx>, Vec<(&Camera, Trs)>) {
        let mut triangles = vec![];
        let mut triangles_ex = vec![];
        let mut cameras = vec![];

        let mut timer = Timer::new();

        let transforms = self.gltf_model.collect_transforms();
        for (node, trs) in transforms {
            // Collect triangles
            if let Some(mesh_handle) = node.mesh {
                let mesh = self.gltf_model.meshes.get(mesh_handle).unwrap();
                for prim_handle in mesh.primitives.iter() {
                    let prim = self.gltf_model.primitives.get(*prim_handle).unwrap();
                    let (mut prim_triangles, mut prim_triangles_ex) = prim.triangles(&trs);
                    triangles.append(&mut prim_triangles);
                    triangles_ex.append(&mut prim_triangles_ex);
                }
            }

            // Collect cameras
            if let Some(camera_handle) = node.camera {
                let camera = self.gltf_model.cameras.get(camera_handle).unwrap();
                cameras.push((camera, trs));
            }
        }

        println!(
            "{:>12} {} triangles in {:.2}s",
            "Collected".green().bold(),
            triangles.len(),
            timer.get_delta().as_secs_f32()
        );
        (triangles, triangles_ex, cameras)
    }
}

impl Draw for Scene {
    fn draw(&self, image: &mut Image) {
        let (triangles, triangles_ex, cameras) = self.collect_triangles();
        let bvh = Bvh::new(triangles, triangles_ex);

        let width = image.width() as f32;
        let height = image.height() as f32;

        let inv_width = 1.0 / width;
        let inv_height = 1.0 / height;

        let (camera, camera_trs) = if cameras.is_empty() {
            (&self.default_camera.camera, self.default_camera.trs.clone())
        } else {
            cameras[0].clone()
        };

        let aspectratio = width / height;
        let angle = camera.get_angle();

        #[cfg(feature = "parallel")]
        let row_iter = image.pixels_mut().into_par_iter();
        #[cfg(not(feature = "parallel"))]
        let row_iter = image.pixels_mut().into_iter();

        let mut timer = Timer::new();

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

                self.draw_pixel_bvh(ray, &bvh, pixel);
            });
        });

        rlog!(
            "{:>12} in {:.2}ms",
            "Rendered".green().bold(),
            timer.get_delta().as_millis()
        );
    }
}
