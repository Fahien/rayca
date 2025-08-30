// Copyright © 2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

pub struct HitInfo<'a> {
    pub scene: &'a SceneDrawInfo<'a>,
    pub tlas: &'a Tlas,

    pub hit: Hit,
    primitive: Option<&'a BvhPrimitive>,
    color: Option<Color>,
    normal: Option<Vec3>,
    uv: Option<Vec2>,
    reflection: Option<Vec3>,
    next_ray_origin: Option<Point3>,
    diffuse: Option<Color>,
}

impl<'a> HitInfo<'a> {
    pub fn new(scene: &'a SceneDrawInfo, tlas: &'a Tlas, hit: Hit) -> Self {
        Self {
            scene,
            tlas,
            hit,
            primitive: None,
            color: None,
            normal: None,
            uv: None,
            reflection: None,
            next_ray_origin: None,
            diffuse: None,
        }
    }

    pub fn get_ray(&self) -> &Ray {
        &self.hit.ray
    }

    pub fn get_view(&self) -> Vec3 {
        self.hit.get_view()
    }

    pub fn get_depth(&self) -> f32 {
        self.hit.depth
    }

    pub fn get_point(&self) -> Point3 {
        self.hit.point
    }

    pub fn get_model(&mut self) -> &'a Model {
        let primitive = self.get_primitive();
        self.scene.get_model(primitive.node.model)
    }

    pub fn get_primitive(&mut self) -> &'a BvhPrimitive {
        if self.primitive.is_none() {
            self.primitive = Some(self.tlas.get_primitive(&self.hit));
        }
        self.primitive.unwrap()
    }

    pub fn get_color(&mut self) -> Color {
        if self.color.is_none() {
            let primitive = self.get_primitive();
            self.color = Some(primitive.get_color(self.scene, &self.hit));
        }
        self.color.unwrap()
    }

    pub fn is_transparent(&mut self) -> bool {
        self.get_color().is_transparent()
    }

    pub fn get_normal(&mut self) -> Vec3 {
        if self.normal.is_none() {
            let primitive = self.get_primitive();
            self.normal = Some(primitive.get_normal(self.scene, &self.hit));
        }
        self.normal.unwrap()
    }

    pub fn get_uv(&mut self) -> Vec2 {
        if self.uv.is_none() {
            let primitive = self.get_primitive();
            self.uv = Some(primitive.get_uv(&self.hit));
        }
        self.uv.unwrap()
    }

    pub fn get_reflection(&mut self) -> Vec3 {
        if self.reflection.is_none() {
            let n = self.get_normal();
            // Reflect the ray direction around the normal
            let r = self.hit.ray.dir.reflect(&n).get_normalized();
            self.reflection = Some(r);
        }
        self.reflection.unwrap()
    }

    pub fn get_material(&mut self) -> &'a Material {
        let primitive = self.get_primitive();
        primitive.get_material(self.scene)
    }

    pub fn get_pbr_material(&mut self) -> &'a PbrMaterial {
        let primitive = self.get_primitive();
        primitive.get_pbr_material(self.scene)
    }

    pub fn get_metallic_roughness(&mut self) -> (f32, f32) {
        let material = self.get_pbr_material();
        let uv = self.get_uv();
        let model = self.get_model();
        material.get_metallic_roughness(model, uv)
    }

    pub fn is_emissive(&mut self) -> bool {
        let primitive = self.get_primitive();
        primitive.is_emissive(self.scene)
    }

    pub fn get_emission(&mut self) -> Color {
        let primitive = self.get_primitive();
        primitive.get_emission(self.scene)
    }

    pub fn get_diffuse(&mut self) -> Color {
        if self.diffuse.is_none() {
            let primitive = self.get_primitive();
            let uv = self.get_uv();
            self.diffuse = Some(primitive.get_diffuse(self.scene, &self.hit, uv));
        }
        self.diffuse.unwrap()
    }

    pub fn get_specular(&mut self) -> Color {
        let model = self.get_model();
        self.get_material().get_specular(model)
    }

    pub fn get_t(&mut self) -> f32 {
        let primitive = self.get_primitive();
        primitive.get_t(self.scene)
    }

    pub fn get_shininess(&mut self) -> f32 {
        let primitive = self.get_primitive();
        primitive.get_shininess(self.scene)
    }

    pub fn get_roughness(&mut self) -> f32 {
        let material = self.get_material();
        material.get_roughness(self.get_model(), self.get_uv())
    }

    pub fn get_phong_material(&mut self) -> &'a PhongMaterial {
        let primitive = self.get_primitive();
        primitive.get_phong_material(self.scene)
    }

    pub fn get_next_ray_origin(&mut self) -> Point3 {
        if self.next_ray_origin.is_none() {
            // Move ray origin slightly along the surface normal to avoid self intersections
            let next_origin = self.hit.point + self.get_normal() * Ray::BIAS;
            self.next_ray_origin = Some(next_origin);
        }
        self.next_ray_origin.unwrap()
    }

    pub fn get_next_ray(&mut self, next_dir: Vec3) -> Ray {
        let next_origin = self.get_next_ray_origin();
        Ray::new(next_origin, next_dir)
    }

    pub fn get_transmit_origin(&mut self) -> Point3 {
        let n = self.get_normal();
        // Move ray origin slightly along the surface normal to avoid self intersections
        self.hit.point + -n * Ray::BIAS
    }

    pub fn get_transmit_ray(&mut self) -> Ray {
        let transmit_origin = self.get_transmit_origin();
        Ray::new(transmit_origin, self.hit.ray.dir)
    }

    pub fn get_reflection_ray(&mut self) -> Ray {
        let next_origin = self.get_next_ray_origin();
        let reflection_dir = self.get_reflection();
        Ray::new(next_origin, reflection_dir)
    }

    pub fn get_shadow_ray(&mut self, light_dir: Vec3) -> Ray {
        let next_origin = self.get_next_ray_origin();
        Ray::new(next_origin, light_dir)
    }

    /// Calculates the light coming out towards the viewer at a certain intersection
    /// This is used by the raytracer and scratcher integrators
    pub fn get_radiance(&mut self, ir: Irradiance) -> Color {
        let material = self.get_material();
        match material {
            Material::Pbr(_) => ggx::get_radiance(self, ir),
            Material::Phong(_) => lambertian::get_radiance(self, ir),
            Material::Ggx(_) => ggx::get_radiance(self, ir),
        }
    }

    pub fn get_random_dir(&mut self) -> Vec3 {
        let material = self.get_material();
        match material {
            Material::Phong(_) => lambertian::get_random_dir(self),
            Material::Pbr(_) => ggx::get_random_dir(self),
            Material::Ggx(_) => ggx::get_random_dir(self),
        }
    }

    pub fn get_brdf(&mut self, omega_i: Vec3) -> Color {
        let material = self.get_material();
        match material {
            Material::Phong(_) => lambertian::get_brdf(self, omega_i),
            Material::Pbr(_) => ggx::get_brdf(self, omega_i),
            Material::Ggx(_) => ggx::get_brdf(self, omega_i),
        }
    }

    pub fn get_pdf(&mut self, omega: Vec3) -> f32 {
        let material = self.get_material();
        match material {
            Material::Phong(_) => lambertian::get_pdf(self, omega),
            Material::Pbr(_) => ggx::get_pdf(self, omega),
            Material::Ggx(_) => ggx::get_pdf(self, omega),
        }
    }

    pub fn get_specular_component(&mut self, omega: Vec3) -> Color {
        let material = self.get_material();
        match material {
            Material::Phong(_) => lambertian::get_specular_component(self, omega),
            Material::Pbr(_) => ggx::get_specular_component(self, omega),
            Material::Ggx(_) => ggx::get_specular_component(self, omega),
        }
    }
}

/// Helper structure which should simplify drawing function interfaces
pub struct Irradiance {
    // Intensity of incoming light
    pub intensity: Color,

    /// Surface normal
    pub n_dot_v: f32,
    pub n_dot_l: f32,

    /// Half-angle (direction between ray and light)
    pub h: Vec3,
    pub n_dot_h: f32,
    pub l_dot_h: f32,
}

impl Irradiance {
    /// - l: light direction
    /// - n: normal to the surface
    /// - v: view direction
    pub fn new(intensity: Color, hit: &mut HitInfo, light_dir: Vec3) -> Self {
        let l = light_dir;
        let n = hit.get_normal();
        let v = -hit.get_ray().dir;

        let n_dot_v = n.dot(&v).clamp(0.0, 1.0) + 1e-5;
        let n_dot_l = n.dot(&l).clamp(0.0, 1.0);
        let h = (v + l).get_normalized();
        let n_dot_h = n.dot(&h).clamp(0.0, 1.0);
        let l_dot_h = l.dot(&h).clamp(0.0, 1.0);

        Self {
            intensity,
            n_dot_v,
            n_dot_l,
            h,
            n_dot_h,
            l_dot_h,
        }
    }
}
