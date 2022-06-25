// Copyright © 2022-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

#[derive(Debug, Default, Clone)]
pub struct GltfSpheres {
    spheres: Vec<Sphere>,
    spheres_ex: Vec<SphereEx>,
}

impl GltfSpheres {
    pub fn new(spheres: Vec<Sphere>, spheres_ex: Vec<SphereEx>) -> Self {
        assert_eq!(spheres.len(), spheres_ex.len());
        Self {
            spheres,
            spheres_ex,
        }
    }

    pub fn unit_sphere() -> Self {
        let sphere = Sphere::new(Point3::default(), 1.0);
        let sphere_ex = SphereEx::default();
        Self::new(vec![sphere], vec![sphere_ex])
    }

    pub fn primitives(
        &self,
        trs: Handle<SolvedTrs>,
        material: Handle<GgxMaterial>,
    ) -> (Vec<Sphere>, Vec<SphereEx>) {
        let mut ret = self.spheres.clone();
        let mut ret_ex = self.spheres_ex.clone();

        for i in 0..ret.len() {
            ret[i].trs = trs;
            ret_ex[i].material = material;
        }

        (ret, ret_ex)
    }
}
