// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::{sample::Sample, *};

pub struct NextEventEstimationSample {
    light: LightDrawInfo,
    omega: Vec3,
    x: Color,
    pdf: f32,
}

impl NextEventEstimationSample {
    pub fn new(light: LightDrawInfo, omega: Vec3, x: Color, pdf: f32) -> Self {
        Self {
            light,
            omega,
            x,
            pdf,
        }
    }
}

impl Sample for NextEventEstimationSample {
    fn get_omega(&self) -> Vec3 {
        self.omega
    }

    fn get_x(&self) -> Color {
        self.x
    }

    fn get_pdf(&self) -> f32 {
        self.pdf
    }

    fn get_pdf_for(&self, sample_hit: &mut HitInfo<'_>, sample: &dyn Sample) -> f32 {
        return Self::get_pdf(self.light, sample_hit, sample.get_omega());
    }
}

impl NextEventEstimationSample {
    fn get_pdf(light: LightDrawInfo, hit: &mut HitInfo, omega: Vec3) -> f32 {
        // Check for intersection ignoring the rest of the scene
        let ray = hit.get_next_ray(omega);
        let Some(light_hit) = light.intersects(hit.scene, ray) else {
            return 0.0;
        };

        let light_node = hit.scene.get_node(light);
        let light = hit.scene.get_light(light);
        let area = light.get_area();
        if area == 0.0 {
            return 0.0;
        }

        let nl_dot_omega = light.get_normal(&light_node.trs).dot(omega).clamp(0.0, 1.0);
        if nl_dot_omega == 0.0 {
            return 0.0;
        }

        let r_squared = (light_hit.point - hit.get_point()).norm();
        r_squared / (area * nl_dot_omega)
    }
}

#[derive(Default)]
pub struct NextEventEstimationSampler {}

impl NextEventEstimationSampler {
    fn sample(
        config: &Config,
        hit: &mut HitInfo,
        light_index: usize,
        light_sample_index: u32,
    ) -> Box<dyn Sample> {
        let light_draw_info = hit.scene.light_draw_infos[light_index];

        let light_node = hit.scene.get_node(light_draw_info);
        let quad_light = hit.scene.get_quad_light(light_draw_info);
        let area = quad_light.get_area();

        let mut ld = Color::BLACK;

        let strate_count = config.get_strate_count();
        let x1 = quad_light.get_random_point(
            &light_node.trs,
            config.light_stratify,
            strate_count,
            light_sample_index,
        );

        // X is the point on the surface
        let x = hit.get_point();
        // Random sample incident direction
        let x_to_x1 = x1 - x;
        let omega = x_to_x1.get_normalized();

        // Let us see if we actually see the light
        let shadow_ray = hit.get_next_ray(omega);
        let mut pdf = 0.0;

        if let Some(mut shadow_hit) = hit.tlas.intersects(hit.scene, shadow_ray) {
            if shadow_hit.is_emissive() {
                // Constant radiance from all point of the light surface
                let le = quad_light.intensity * quad_light.color;

                let brdf = hit.get_brdf(omega);

                let r_squared = x_to_x1.norm();
                let d_omega = quad_light.get_normal().dot(omega) / r_squared;
                let n_dot_omega = hit.get_normal().dot(omega).clamp(0.0, 1.0);
                ld = le * area * brdf * n_dot_omega * d_omega;
                pdf = NextEventEstimationSample::get_pdf(light_draw_info, hit, omega);
            }
        }

        Box::new(NextEventEstimationSample::new(
            light_draw_info,
            omega,
            ld,
            pdf,
        ))
    }

    fn get_light_samples(
        config: &Config,
        hit: &mut HitInfo,
        light_index: usize,
    ) -> Vec<Box<dyn Sample>> {
        let mut samples = vec![];
        for light_sample_index in 0..config.light_samples {
            let sample = Self::sample(config, hit, light_index, light_sample_index);
            samples.push(sample);
        }
        samples
    }

    pub fn get_samples(&self, config: &Config, hit: &mut HitInfo) -> Vec<Box<dyn Sample>> {
        let mut samples = vec![];
        for light_index in 0..hit.scene.light_draw_infos.len() {
            let light_sample = Self::get_light_samples(config, hit, light_index);
            samples.extend(light_sample);
        }
        samples
    }
}

impl DirectSampler for NextEventEstimationSampler {
    fn get_direct_lighting(&self, config: &Config, hit: &mut HitInfo) -> Color {
        let samples = self.get_samples(config, hit);
        let mut ret = Color::BLACK;
        for sample in samples {
            ret += sample.get_x();
        }
        ret
    }
}
