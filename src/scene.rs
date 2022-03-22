// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

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
}

impl Scene {
    fn draw_pixel(&self, ray: Ray, pixel: &mut RGBA8) {
        let mut depth = f32::INFINITY;

        for i in 0..self.triangles.len() {
            let triangle = &self.triangles[i];
            if let Some(hit) = triangle.intersects(&ray) {
                if hit.depth < depth {
                    depth = hit.depth;
                    let triangle_ex = &self.triangles_ex[i];
                    let color = triangle_ex.get_color(&hit);
                    *pixel = color.into();
                }
            }
        }

        for i in 0..self.spheres.len() {
            let sphere = &self.spheres[i];
            if let Some(hit) = sphere.intersects(&ray) {
                if hit.depth < depth {
                    depth = hit.depth;
                    let sphere_ex = &self.spheres_ex[i];
                    let color = sphere_ex.get_color(sphere, &hit);
                    *pixel = color.into();
                }
            }
        }

        for node in self.gltf_model.nodes.iter() {
            for mesh in self.gltf_model.meshes.iter() {
                for prim_handle in mesh.primitives.iter() {
                    let prim = self.gltf_model.primitives.get(*prim_handle).unwrap();
                    let transform = Mat4::from(&node.trs);
                    let (triangles, triangles_ex) = prim.triangles(transform);
                    for i in 0..triangles.len() {
                        let triangle = &triangles[i];
                        if let Some(hit) = triangle.intersects(&ray) {
                            if hit.depth < depth {
                                depth = hit.depth;
                                let triangle_ex = &triangles_ex[i];
                                let n = triangle_ex.get_normal(&hit);
                                let color = Color::from(n);
                                *pixel = color.into();
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Draw for Scene {
    fn draw(&self, image: &mut Image) {
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
                let origin = Point3::new(0.0, 0.0, 4.0);
                let ray = Ray::new(origin, dir);

                self.draw_pixel(ray, pixel);
            });
        });
    }
}
