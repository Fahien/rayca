// Copyright © 2022-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use owo_colors::OwoColorize;

#[cfg(feature = "parallel")]
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};

use super::*;

#[derive(Default)]
pub struct Scene {
    pub triangles: Vec<Triangle>,
    pub triangles_ex: Vec<TriangleEx>,

    pub spheres: Vec<Sphere>,
    pub spheres_ex: Vec<SphereEx>,

    pub gltf_model: GltfModel,
}

impl Scene {
    pub fn new() -> Self {
        Self::default()
    }

    fn draw_pixel_triangles(
        &self,
        ray: Ray,
        triangles: &[Triangle],
        triangles_ex: &[TriangleEx],
        depth: &mut f32,
        pixel: &mut RGBA8,
    ) {
        for i in 0..triangles.len() {
            let triangle = &triangles[i];
            if let Some(hit) = triangle.intersects(&ray) {
                if hit.depth < *depth {
                    *depth = hit.depth;
                    let triangle_ex = &triangles_ex[i];
                    let mut color = triangle_ex.get_color(&hit, self);

                    // Facing ratio
                    let n = triangle_ex.get_normal(&hit);
                    let n_dot_dir = n.dot(&-ray.dir);
                    color.r *= n_dot_dir;
                    color.g *= n_dot_dir;
                    color.b *= n_dot_dir;

                    pixel.over(color);
                    //*pixel = color.into();
                }
            }
        }
    }

    fn draw_pixel_spheres(
        &self,
        ray: Ray,
        spheres: &[Sphere],
        spheres_ex: &[SphereEx],
        depth: &mut f32,
        pixel: &mut RGBA8,
    ) {
        for i in 0..spheres.len() {
            let sphere = &spheres[i];
            if let Some(hit) = sphere.intersects(&ray) {
                if hit.depth < *depth {
                    *depth = hit.depth;
                    let sphere_ex = &spheres_ex[i];
                    let color = sphere_ex.get_color(sphere, &hit);
                    *pixel = color.into();
                }
            }
        }
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

                let mut depth = f32::INFINITY;
                self.draw_pixel_triangles(
                    ray.clone(),
                    &triangles,
                    &triangles_ex,
                    &mut depth,
                    pixel,
                );
                self.draw_pixel_triangles(
                    ray.clone(),
                    &self.triangles,
                    &self.triangles_ex,
                    &mut depth,
                    pixel,
                );
                self.draw_pixel_spheres(ray, &self.spheres, &self.spheres_ex, &mut depth, pixel);
            });
        });

        rlog!(
            "{:>12} in {:.2}ms",
            "Rendered".green().bold(),
            timer.get_delta().as_millis()
        );
    }
}
