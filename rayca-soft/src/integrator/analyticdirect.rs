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
        tlas: &Tlas,
        ray: Ray,
        depth: u32,
    ) -> Option<Color> {
        if depth >= config.max_depth {
            return None;
        }

        let mut hit = tlas.intersects(scene, ray)?;

        // This is the color of the primitive with no light
        let ambient_and_emission = hit.get_color();
        if hit.is_emissive() {
            return Some(ambient_and_emission);
        }

        let n = hit.get_normal();
        let mut light_contribution = Color::BLACK;

        for light_draw_info in scene.light_draw_infos.iter().copied() {
            let light_node = scene.get_node(light_draw_info);
            let light = scene.get_light(light_draw_info);
            light_contribution += light.get_intensity(&light_node.trs, hit.get_point(), n);
        }

        let f = hit.get_diffuse() * std::f32::consts::FRAC_1_PI;
        Some(f * light_contribution)
    }
}
