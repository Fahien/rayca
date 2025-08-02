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
        let n = primitive.get_normal(scene, &hit);

        // This is the color of the primitive with no light
        let ambient_and_emission = primitive.get_color(scene, &hit);

        let model = scene.get_model(primitive.node.model);
        let uv = primitive.get_uv(&hit);
        let f = primitive.get_material(scene).get_diffuse(model, uv) * std::f32::consts::FRAC_1_PI;

        let mut light_contribution = Color::black();

        for light_draw_info in scene.light_draw_infos.iter().copied() {
            let light_node = scene.get_node(light_draw_info);
            let light = scene.get_light(light_draw_info);
            light_contribution += f * light.get_intensity(&light_node.trs, hit.point, n);
        }

        Some(ambient_and_emission + light_contribution)
    }
}
