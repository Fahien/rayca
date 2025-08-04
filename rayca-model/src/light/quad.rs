// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

/// ```text
///    c  /‾‾‾‾‾/ d
///      /     /
///  ac /     /
///    /     /
/// a /_____/ b
///     ab
/// ```
#[derive(Default)]
pub struct QuadLight {
    pub ab: Vec3,
    pub ac: Vec3,
    pub color: Color,
    pub intensity: f32,
    pub material: Handle<Material>,
}

impl QuadLight {
    pub fn new(ab: Vec3, ac: Vec3, color: Color, material: Handle<Material>) -> Self {
        Self {
            ab,
            ac,
            color,
            intensity: 1.0,
            material,
        }
    }

    /// Returns the normal of the quad light in model space.
    pub fn get_normal(&self) -> Vec3 {
        self.ab.cross(self.ac).get_normalized()
    }

    pub fn get_area(&self) -> f32 {
        let ab_len = self.ab.len();
        let ac_len = self.ac.len();
        let cos_theta = self.ab.dot(self.ac) / (ab_len * ac_len);
        let sin_theta = 1.0 - cos_theta;
        sin_theta * ab_len * ac_len
    }

    pub fn get_a(&self, trs: &Trs, edge_index: u32) -> Point3 {
        let a: Point3 = trs.get_translation().into();
        match edge_index {
            0 => a,
            1 => a + self.ab,
            2 => a + self.ab + self.ac,
            3 => a + self.ac,
            _ => panic!("Quad light edge index out of bounds"),
        }
    }

    pub fn get_b(&self, trs: &Trs, edge_index: u32) -> Point3 {
        let a: Point3 = trs.get_translation().into();
        match edge_index {
            0 => a + self.ab,
            1 => a + self.ab + self.ac,
            2 => a + self.ac,
            3 => a,
            _ => panic!("Quad light edge index out of bounds"),
        }
    }

    /// Returns the angle subtended by the `i`th edge of the quad as seen by `frag_pos`
    pub fn theta(&self, trs: &Trs, edge_index: u32, frag_pos: Point3) -> f32 {
        // Frag pos is `r`, `ra` is unit vector from `r` to `a`.
        let ra = (self.get_a(trs, edge_index) - frag_pos).get_normalized();
        // Frag pos is `r`, `rb` is unit vector from `r` to `b`.
        let rb = (self.get_b(trs, edge_index) - frag_pos).get_normalized();
        (ra.dot(rb)).acos()
    }

    /// Returns the unit normal of the polygonal cone with cross section this quad and apex at `frag_pos`
    pub fn gamma(&self, trs: &Trs, edge_index: u32, frag_pos: Point3) -> Vec3 {
        let ra = self.get_a(trs, edge_index) - frag_pos;
        let rb = self.get_b(trs, edge_index) - frag_pos;
        ra.cross(rb).get_normalized()
    }

    /// Returns the intensity of quad light reaching that point, at a certain angle.
    pub fn get_intensity(&self, trs: &Trs, frag_pos: Point3, frag_n: Vec3) -> Color {
        let omega = self.get_radiance(trs, frag_pos);
        let irradiance = omega.dot(frag_n);
        self.intensity * self.color * irradiance
    }

    pub fn get_radiance(&self, trs: &Trs, frag_pos: Point3) -> Vec3 {
        let mut ret = Vec3::default();
        for edge_index in 0..4 {
            ret += self.theta(trs, edge_index, frag_pos) * self.gamma(trs, edge_index, frag_pos);
        }
        ret /= 2.0;
        ret
    }

    pub fn get_fallof(&self) -> f32 {
        1.0
    }

    /// Returns a random point on the surface of the area light.
    /// - `trs`: transformation of the light
    /// - `stratify`: if true, the area will be stratified according to
    ///   the `strate_count` and a point in the `i`th stratum will be returned.
    /// - `strate_count`: number of strata to divide the area into
    /// - `i`: index of the stratum to return
    pub fn get_random_point(&self, trs: &Trs, stratify: bool, strate_count: u32, i: u32) -> Point3 {
        let step1 = self.ab / strate_count as f32;
        let step2 = self.ac / strate_count as f32;

        let u1 = fastrand::f32() / strate_count as f32;
        let u2 = fastrand::f32() / strate_count as f32;

        let a = self.get_a(trs, 0);
        let mut x1 = a + (u1 * self.ab) + (u2 * self.ac);

        if stratify {
            let i1 = (i % strate_count) as f32;
            let i2 = (i / strate_count) as f32;

            let offset1 = step1 * i1;
            let offset2 = step2 * i2;

            x1 += offset1 + offset2;
        }

        x1
    }
}
