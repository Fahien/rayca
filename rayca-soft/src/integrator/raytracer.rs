// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

#[derive(Default)]
pub struct Raytracer {}

impl Raytracer {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Integrator for Raytracer {
    fn trace(
        &self,
        config: &Config,
        scene: &SceneDrawInfo,
        ray: Ray,
        tlas: &Tlas,
        depth: u32,
    ) -> Option<Color> {
        if depth > config.max_depth {
            return None;
        }

        let hit = tlas.intersects(scene, &ray)?;
        let blas = tlas.get_blas(hit.blas);
        let primitive = blas.model.get_primitive(hit.primitive);

        let n = primitive.get_normal(scene, &hit);
        let albedo_color = primitive.get_color(scene, &hit);
        let uv = primitive.get_uv(&hit);

        const RAY_BIAS: f32 = 1e-4;
        let next_origin = hit.point + n * RAY_BIAS;

        let ambient_emissive = albedo_color;

        let mut light_contribution = Color::black();

        // Lights iteration
        for light_draw_info in scene.light_draw_infos.iter().copied() {
            let light_node = scene.get_node(light_draw_info);
            let light = scene.get_light(light_draw_info);
            let light_dir = light.get_direction(&light_node.trs, &hit.point);

            let shadow_ray = Ray::new(next_origin, light_dir);
            let shadow_result = tlas.intersects(scene, &shadow_ray);

            // Whether this object is light (verb) by a light (noun)
            let is_light = match shadow_result {
                None => true,
                Some(shadow_hit) => {
                    let light_distance = light.get_distance(&light_node.trs, &hit.point);
                    if shadow_hit.depth > light_distance {
                        true
                    } else {
                        // check whether this is a transparent surface
                        let shadow_color = albedo_color;
                        shadow_color.a < 1.0
                    }
                }
            };

            if is_light {
                let intensity = light.get_intensity(&light_node.trs, hit.point, n);
                let ir = Irradiance::new(intensity, &hit, light_dir, n, -ray.dir, albedo_color, uv);
                light_contribution += primitive.get_radiance(scene, &ir);
            }
        } // end iterate light

        let reflection_dir = ray.dir.reflect(&n).get_normalized();
        let reflection_ray = Ray::new(next_origin, reflection_dir);
        if let Some(reflection_color) = self.trace(config, scene, reflection_ray, tlas, depth + 1) {
            // Cosine-law should apply here as well
            let n_dot_r = 1.0; //n.dot(&reflection_dir);
            let blas = tlas.get_blas(hit.blas);
            let primitive = blas.model.get_primitive(hit.primitive);
            let specular = primitive.get_specular(scene);
            light_contribution += reflection_color * specular * n_dot_r;
        }

        Some(ambient_emissive + light_contribution)
    }
}
