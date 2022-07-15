// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

pub struct BvhSphere<'m> {
    center: Point3,
    radius: f32,
    radius2: f32,

    pub trs: &'m Trs,
    pub material: Option<Handle<Material>>,
    pub model: &'m Model,
}

impl<'m> BvhSphere<'m> {
    pub fn new(
        center: Point3,
        radius: f32,
        trs: &'m Trs,
        material: Option<Handle<Material>>,
        model: &'m Model,
    ) -> Self {
        Self {
            center,
            radius,
            radius2: radius * radius,
            trs,
            material,
            model,
        }
    }

    pub fn get_center(&self) -> Point3 {
        self.trs * self.center
    }

    pub fn get_radius(&self) -> f32 {
        self.radius
    }

    /// This will return a point outside of the sphere, useful for the AABB
    pub fn min(&self) -> Point3 {
        let rad3 = Vec3::splat(self.radius);
        self.get_center() - rad3
    }

    pub fn max(&self) -> Point3 {
        let rad3 = Vec3::splat(self.radius);
        self.get_center() + rad3
    }

    pub fn get_material(&self) -> &Material {
        if let Some(material_handle) = self.material {
            self.model.materials.get(material_handle).unwrap()
        } else {
            &Material::WHITE
        }
    }

    /// Point should be in geometry space
    fn get_normal_impl(&self, point: &Point3) -> Vec3 {
        (point - self.center).get_normalized()
    }

    /// Geometric formula.
    /// Ray should be in model space
    pub fn intersects_impl(&self, ray: &Ray) -> Option<Hit> {
        // a = p1 * p1
        let a = ray.dir.dot(&ray.dir);

        // b = 2(p1 * (p0 - c))
        // sphere center to ray origin vector
        let c_to_r = ray.origin - self.center;
        let b = 2.0 * c_to_r.dot(&ray.dir);

        // c = (p0 - c) * (p0 - c) - r^2
        let c = c_to_r.dot(&c_to_r) - self.radius2;

        // (-b +- sqrt(b^2 - 4ac) ) / 2a;
        let det = b * b - 4.0 * a * c;
        if det < 0.0 {
            return None;
        }

        let t0 = (-b + det.sqrt()) / (2.0 * a);
        let t1 = (-b - det.sqrt()) / (2.0 * a);

        if t0 < 0.0 && t1 < 0.0 {
            return None; // Sphere behind ray origin
        }

        let t = if t0 >= 0.0 && t1 >= 0.0 {
            // Two positive roots, pick smaller
            t0.min(t1)
        } else if t0 >= 0.0 {
            // Ray origin inside sphere, pick positive
            t0
        } else {
            t1
        };

        let point = ray.origin + ray.dir * t;
        let hit = Hit::new(self, t, point, Vec2::default());

        Some(hit)
    }
}

impl<'m> Intersect<'m> for BvhSphere<'m> {
    fn intersects(&self, ray: &Ray) -> Option<Hit> {
        let ray = ray.clone();
        let inverse = Inversed::from(self.trs);
        let inverse_ray = &inverse * ray;
        let mut hit = self.intersects_impl(&inverse_ray)?;
        let transformed_point = hit.point;
        hit.point = self.trs * transformed_point;
        Some(hit)
    }

    fn get_color(&self, _hit: &Hit) -> Color {
        self.get_material().color
    }

    fn get_metallic_roughness(&self, _hit: &Hit) -> (f32, f32) {
        let uv = Vec2::default();
        self.get_material().get_metallic_roughness(&uv, self.model)
    }

    fn get_normal(&self, hit: &Hit) -> Vec3 {
        let inverse = self.trs.get_inversed();
        let hit_point = &inverse * hit.point;
        let normal = self.get_normal_impl(&hit_point);

        let normal_matrix = Mat3::from(&inverse).get_transpose();
        (&normal_matrix * normal).get_normalized()
    }

    fn get_uv(&self, _hit: &Hit) -> Vec2 {
        // TODO: Sphere uvs?
        Vec2::default()
    }

    fn get_radiance(&self, irradiance: &Irradiance) -> Color {
        self.get_material().get_radiance(irradiance, self.model)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn intersect() {
        let model = Model::new();
        let trs = Trs::default();

        let orig = Point3::new(0.0, 0.0, 0.0);
        let sphere = BvhSphere::new(orig, 1.0, &trs, None, &model);

        let right = Vec3::new(1.0, 0.0, 0.0);
        let ray = Ray::new(orig, right);
        assert!(sphere.intersects(&ray).is_some());

        let ray = Ray::new(Point3::new(2.0, 0.0, 0.0), right);
        assert!(sphere.intersects(&ray).is_none());

        let sphere = BvhSphere::new(Point3::new(4.0, 0.0, 0.0), 1.0, &trs, None, &model);
        let forward = Vec3::new(0.0, 0.0, -1.0);
        let ray = Ray::new(orig, forward);
        assert!(sphere.intersects(&ray).is_none());
    }
}
