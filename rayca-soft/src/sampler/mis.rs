// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::{sample::Sample, sampler::brdf::BrdfSampler, Color, Config, HitInfo};

use super::{DirectSampler, NextEventEstimationSampler};

/// This sampler uses multiple importance sampling to calculate direct lighting
/// combining two integrators: next event estimation, and brdf.
#[derive(Default)]
pub struct MultipleImportanceSampling {
    nee: NextEventEstimationSampler,
    brdf: BrdfSampler,
}

impl MultipleImportanceSampling {
    pub const fn new() -> Self {
        Self {
            nee: NextEventEstimationSampler {},
            brdf: BrdfSampler {},
        }
    }

    fn get_pdf_nee(
        &self,
        hit: &mut HitInfo,
        sample: &dyn Sample,
        light_samples: &[Box<dyn Sample>],
    ) -> f32 {
        let mut ret = 0.0;
        for other_sample in light_samples {
            ret += other_sample.get_pdf_for(hit, sample);
        }
        ret / light_samples.len() as f32
    }
}

impl DirectSampler for MultipleImportanceSampling {
    fn get_direct_lighting(&self, config: &Config, hit: &mut HitInfo) -> Color {
        let light_samples = self.nee.get_samples(config, hit);
        let brdf_sample = self.brdf.sample_direct(hit);

        let mut ret = Color::BLACK;

        // Accumulate light samples
        // N(NEE) = number of lights
        for i in 0..light_samples.len() {
            let pdf_nee = self.get_pdf_nee(hit, light_samples[i].as_ref(), &light_samples);
            let pdf_brdf = brdf_sample.get_pdf_for(hit, light_samples[i].as_ref());
            let pdf_den = pdf_nee.powf(2.0) + pdf_brdf.powf(2.0);
            let w = if pdf_den == 0.0 {
                0.0
            } else {
                pdf_nee.powf(2.0) / pdf_den
            };
            ret += w * light_samples[i].get_x();
        }

        // N(BRDF) = 1
        let pdf_nee = self.get_pdf_nee(hit, &brdf_sample, &light_samples);
        let pdf_brdf = brdf_sample.get_pdf();
        let pdf_den = pdf_nee.powf(2.0) + pdf_brdf.powf(2.0);
        let w = if pdf_den == 0.0 {
            0.0
        } else {
            pdf_brdf.powf(2.0) / pdf_den
        };
        ret += w * brdf_sample.get_x();

        ret
    }
}
