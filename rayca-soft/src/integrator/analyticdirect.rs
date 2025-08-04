// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

#[derive(Default)]
pub struct AnalyticDirect {}

impl AnalyticDirect {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Integrator for AnalyticDirect {
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

        // This is the color of the primitive with no light
        let ambient_and_emission = primitive.get_color(scene, &hit);
        if primitive.is_emissive(scene) {
            return Some(ambient_and_emission);
        }

        let n = primitive.get_normal(scene, &hit);
        let mut light_contribution = Color::BLACK;

        for light_draw_info in scene.light_draw_infos.iter().copied() {
            let light_node = scene.get_node(light_draw_info);
            let light = scene.get_light(light_draw_info);
            light_contribution += light.get_intensity(&light_node.trs, hit.point, n);
        }

        let uv = primitive.get_uv(&hit);
        let f = primitive.get_diffuse(scene, &hit, uv) * std::f32::consts::FRAC_1_PI;
        Some(f * light_contribution)
    }
}
