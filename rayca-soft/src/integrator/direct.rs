// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

#[derive(Default)]
pub struct Direct {}

impl Direct {
    const RAY_BIAS: f32 = 1e-4;

    pub const fn new() -> Self {
        Self {}
    }
}

impl Integrator for Direct {
    fn trace(
        &self,
        config: &Config,
        scene: &SceneDrawInfo,
        ray: Ray,
        tlas: &Tlas,
        depth: u32,
    ) -> Option<Color> {
        if depth >= config.max_depth {
            return None;
        }

        let hit = tlas.intersects(scene, &ray)?;

        let primitive = tlas.get_primitive(&hit);
        let n = primitive.get_normal(scene, &hit);

        let reflection_dir = ray.dir.reflect(&n).get_normalized();

        // This is the color of the primitive with no light
        let ambient_and_emissive = primitive.get_color(scene, &hit);

        if primitive.is_emissive(scene) {
            return Some(ambient_and_emissive);
        }

        let mut light_contribution = Color::black();

        let uv = primitive.get_uv(&hit);
        let diffuse = primitive.get_diffuse(scene, &hit, uv);
        let specular = primitive.get_specular(scene);
        let shininess = primitive.get_shininess(scene);

        let strate_count = config.get_strate_count();

        // Expecting only area lights here
        for light_draw_info in scene.light_draw_infos.iter().copied() {
            let light = scene.get_light(light_draw_info);
            let light_node = scene.get_node(light_draw_info);
            if let Light::Quad(quad_light) = light {
                let mut ld = Color::black();

                for i in 0..config.light_samples {
                    let x1 = quad_light.get_random_point(
                        &light_node.trs,
                        config.light_stratify,
                        strate_count,
                        i,
                    );

                    // Random sample incident direction
                    let x1_to_hit_point = x1 - hit.point;
                    let omega_i = x1_to_hit_point.get_normalized();

                    // Move ray origin slightly along the surface normal to avoid self intersections
                    let shadow_ray_origin = hit.point + n * Self::RAY_BIAS;
                    // Let us see if we actually see the light
                    let shadow_ray = Ray::new(shadow_ray_origin, omega_i);
                    if let Some(shadow_hit) = tlas.intersects(scene, &shadow_ray) {
                        let primitive = tlas.get_primitive(&shadow_hit);
                        if !primitive.is_emissive(scene) {
                            continue;
                        }
                    }

                    // BRDF
                    let kd = diffuse;
                    let lambertian = kd * std::f32::consts::FRAC_1_PI;
                    let ks = specular;
                    let s = shininess;
                    let r = &reflection_dir;
                    let brdf = lambertian
                        + (ks * (s + 2.0) * r.dot(omega_i).powf(s)) * std::f32::consts::FRAC_1_PI
                            / 2.0;

                    let r_squared = x1_to_hit_point.len().powf(2.0);
                    let d_omega_i = quad_light.get_normal().dot(omega_i) / r_squared;

                    ld += brdf * n.dot(omega_i) * d_omega_i;
                }

                // Constant radiance?
                let li = quad_light.intensity * quad_light.color;
                let area = quad_light.get_area();
                ld = (li * area * ld) / (config.light_samples as f32);
                light_contribution += ld;
            }
        }

        Some(light_contribution)
    }
}
