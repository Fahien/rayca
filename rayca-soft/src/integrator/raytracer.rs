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
        tlas: &Tlas,
        ray: Ray,
        depth: u32,
    ) -> Option<Color> {
        if depth > config.max_depth {
            return None;
        }

        let mut hit = tlas.intersects(scene, ray)?;

        let ambient_emissive = hit.get_color();

        let mut light_contribution = Color::BLACK;

        // Lights iteration
        for light_draw_info in scene.light_draw_infos.iter().copied() {
            let light_node = scene.get_node(light_draw_info);
            let light = scene.get_light(light_draw_info);
            let light_dir = light.get_direction(&light_node.trs, &hit.get_point());

            let shadow_ray = hit.get_shadow_ray(light_dir);
            let shadow_result = tlas.intersects(scene, shadow_ray);

            // Whether this object is light (verb) by a light (noun)
            let is_light = match shadow_result {
                None => true,
                Some(mut shadow_hit) => {
                    let light_distance = light.get_distance(&light_node.trs, &hit.get_point());
                    if shadow_hit.get_depth() > light_distance {
                        true
                    } else {
                        // check whether this is a transparent surface
                        shadow_hit.is_transparent()
                    }
                }
            };

            if is_light {
                let intensity =
                    light.get_intensity(&light_node.trs, hit.get_point(), hit.get_normal());
                let ir = Irradiance::new(intensity, &mut hit, light_dir);
                light_contribution += hit.get_radiance(ir);
            }
        } // end iterate light

        let reflection_ray = hit.get_reflection_ray();
        if let Some(reflection_color) =
            self.trace(config, hit.scene, hit.tlas, reflection_ray, depth + 1)
        {
            // Cosine-law should apply here as well
            let n_dot_r = 1.0; //n.dot(&reflection_dir);
            light_contribution += reflection_color * hit.get_specular() * n_dot_r;
        }

        Some(ambient_emissive + light_contribution)
    }
}
