// Copyright © 2022-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use rayon::{
    iter::IndexedParallelIterator,
    prelude::{IntoParallelIterator, ParallelIterator},
};

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
                    let color = triangle_ex.get_color(&hit, self);
                    *pixel = color.into();
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

    fn collect_triangles(&self) -> (Vec<Triangle>, Vec<TriangleEx>) {
        let mut triangles = vec![];
        let mut triangles_ex = vec![];

        for node in self.gltf_model.nodes.iter() {
            if let Some(mesh_handle) = node.mesh {
                let mesh = self.gltf_model.meshes.get(mesh_handle).unwrap();
                for prim_handle in mesh.primitives.iter() {
                    let prim = self.gltf_model.primitives.get(*prim_handle).unwrap();
                    let transform = Mat4::from(&node.trs);
                    let (mut prim_triangles, mut prim_triangles_ex) =
                        prim.triangles(transform, prim.material);
                    triangles.append(&mut prim_triangles);
                    triangles_ex.append(&mut prim_triangles_ex);
                }
            }
        }

        println!("Collected {} triangles", triangles.len());
        (triangles, triangles_ex)
    }
}

impl Draw for Scene {
    fn draw(&self, image: &mut Image) {
        let (triangles, triangles_ex) = self.collect_triangles();

        let width = image.width() as f32;
        let height = image.height() as f32;

        let inv_width = 1.0 / width;
        let inv_height = 1.0 / height;

        let fov = 30.0;
        let aspectratio = width / height;
        let angle = (std::f32::consts::FRAC_PI_2 * fov / 180.0).tan();

        let row_iter = image.pixels_mut().into_par_iter();

        row_iter.enumerate().for_each(|(y, row)| {
            row.into_par_iter().enumerate().for_each(|(x, pixel)| {
                // Generate primary ray
                let xx = (2.0 * ((x as f32 + 0.5) * inv_width) - 1.0) * angle * aspectratio;
                let yy = (1.0 - 2.0 * ((y as f32 + 0.5) * inv_height)) * angle;
                let mut dir = Vec3::new(xx, yy, -1.0);
                dir.normalize();
                let origin = Point3::new(0.0, 0.0, 4.5);
                let ray = Ray::new(origin, dir);

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
    }
}
