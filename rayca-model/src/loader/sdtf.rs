// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::*;

/// Strategy for performing integration of the rendering equation and decide a pixel color
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SdtfIntegratorStrategy {
    /// Very roughly approximates the rendering equation using raytracing
    Raytracer,

    /// Analytic solution of direct lighting for simple scenes
    AnalyticDirect,

    /// Direct lighting using Monte Carlo integration
    Direct,

    /// Monte Carlo direct/indirect lighting
    Pathtracer,
}

impl FromStr for SdtfIntegratorStrategy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "raytracer" => Ok(Self::Raytracer),
            "analyticdirect" => Ok(Self::AnalyticDirect),
            "direct" => Ok(Self::Direct),
            "pathtracer" => Ok(Self::Pathtracer),
            _ => Err(format!("Failed to find an integrator for `{}`", s)),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SdtfSamplerStrategy {
    Hemisphere,
}

impl FromStr for SdtfSamplerStrategy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "hemisphere" => Ok(Self::Hemisphere),
            _ => Err(format!("Failed to find a sampler for `{}`", s)),
        }
    }
}

#[derive(Debug)]
pub struct SdtfConfig {
    pub width: u32,
    pub height: u32,
    pub max_depth: i32,
    pub light_samples: u32,
    pub light_stratify: bool,
    pub samples_per_pixel: u32,
    pub next_event_estimation: bool,
    pub russian_roulette: bool,
    pub sampler: SdtfSamplerStrategy,
    pub integrator: SdtfIntegratorStrategy,
}

impl Default for SdtfConfig {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            max_depth: 5,
            light_samples: 1,
            light_stratify: false,
            samples_per_pixel: 1,
            next_event_estimation: false,
            russian_roulette: false,
            sampler: SdtfSamplerStrategy::Hemisphere,
            integrator: SdtfIntegratorStrategy::Raytracer,
        }
    }
}

struct SdtfBuilder {
    path: Option<PathBuf>,
    string: Option<String>,
    vertices: Pack<Vertex>,
    transform: Vec<Trs>,
    temp_material: PhongMaterial,
    attenuation: Vec3,
    temp_model: Model,
    config: SdtfConfig,
}

impl SdtfBuilder {
    pub fn new() -> Self {
        Self {
            path: None,
            string: None,
            vertices: Pack::new(),
            transform: Vec::new(),
            temp_material: PhongMaterial::default(),
            attenuation: Vec3::new(1.0, 0.0, 0.0),
            temp_model: Model::default(),
            config: SdtfConfig::default(),
        }
    }
    pub fn path(mut self, path: impl AsRef<Path>) -> Self {
        self.path = Some(PathBuf::from(path.as_ref()));
        self
    }

    pub fn _str(mut self, string: impl AsRef<str>) -> Self {
        self.string = Some(string.as_ref().to_string());
        self
    }

    fn parse_size<'w>(
        &mut self,
        mut words: impl Iterator<Item = &'w str>,
    ) -> Result<(), Box<dyn Error>> {
        let width = words.next().expect("Failed to read width");
        self.config.width = width.parse()?;

        let height = words.next().expect("Failed to read height");
        self.config.height = height.parse()?;
        Ok(())
    }

    fn parse_camera<'w>(
        mut words: impl Iterator<Item = &'w str>,
        model: &mut Model,
    ) -> Result<(), Box<dyn Error>> {
        let eye = Vec3::new(
            words.next().expect("Failed to read camera x").parse()?,
            words.next().expect("Failed to read camera y").parse()?,
            words.next().expect("Failed to read camera z").parse()?,
        );

        let target = Vec3::new(
            words
                .next()
                .expect("Failed to read camera target x")
                .parse()?,
            words
                .next()
                .expect("Failed to read camera target y")
                .parse()?,
            words
                .next()
                .expect("Failed to read camera target z")
                .parse()?,
        );

        let up = Vec3::new(
            words.next().expect("Failed to read camera up x").parse()?,
            words.next().expect("Failed to read camera up y").parse()?,
            words.next().expect("Failed to read camera up z").parse()?,
        );

        let yfov_degrees: f32 = words.next().expect("Failed to read camera fov").parse()?;
        let yfov_radians = yfov_degrees * std::f32::consts::PI / 180.0;
        log::info!("Yfov : {}", yfov_radians);
        log::info!("Target : {:?}", target);
        log::info!("Eye : {:?}", eye);
        log::info!("Up : {:?}", up);

        // Aspect ratio always 1.0
        let camera = Camera::infinite_perspective(1.0, yfov_radians, 0.1);
        let camera_handle = model.cameras.push(camera);
        let look_at_matrix = Mat4::look_at(target, eye, up);

        // Invert look-at matrix to obtain camera transform
        let translation = eye;
        let rotation = look_at_matrix.get_rotation().get_inverse();

        let camera_node = Node::builder()
            .camera(camera_handle)
            .trs(
                Trs::builder()
                    .translation(translation)
                    .rotation(rotation)
                    .build(),
            )
            .build();
        let camera_node_handle = model.nodes.push(camera_node);
        model.root.children.push(camera_node_handle);

        Ok(())
    }

    fn parse_maxverts<'w>(
        &mut self,
        mut words: impl Iterator<Item = &'w str>,
    ) -> Result<(), Box<dyn Error>> {
        let max_verts = words.next().expect("Failed to read max verts").parse()?;
        self.vertices.reserve(max_verts);
        Ok(())
    }

    fn parse_vertex<'w>(
        &mut self,
        mut words: impl Iterator<Item = &'w str>,
    ) -> Result<(), Box<dyn Error>> {
        let x = words.next().expect("Failed to read vertex x").parse()?;
        let y = words.next().expect("Failed to read vertex y").parse()?;
        let z = words.next().expect("Failed to read vertex z").parse()?;
        let vertex = Vertex::builder().position(Point3::new(x, y, z)).build();
        self.vertices.push(vertex);
        Ok(())
    }

    fn parse_tri<'w>(
        &mut self,
        mut words: impl Iterator<Item = &'w str>,
    ) -> Result<(), Box<dyn Error>> {
        let a_index: usize = words.next().expect("Failed to read vertex x").parse()?;
        let b_index: usize = words.next().expect("Failed to read vertex y").parse()?;
        let c_index: usize = words.next().expect("Failed to read vertex z").parse()?;

        // Create or get current primitive
        if self.temp_model.primitives.is_empty() {
            let triangle_mesh = TriangleMesh::builder()
                .vertices(vec![])
                .indices(TriangleIndices::default())
                .build();
            let geometry_handle = self
                .temp_model
                .geometries
                .push(Geometry::TriangleMesh(triangle_mesh));
            self.temp_model
                .primitives
                .push(Primitive::builder().geometry(geometry_handle).build());
        }

        let geometry_handle = self.temp_model.primitives[0].geometry;
        let geometry = self.temp_model.get_geometry_mut(geometry_handle).unwrap();
        if let Geometry::TriangleMesh(triangle_mesh) = geometry {
            let last_vertex_index = triangle_mesh.vertices.len();
            triangle_mesh.indices.add_index(last_vertex_index);
            triangle_mesh.indices.add_index(last_vertex_index + 1);
            triangle_mesh.indices.add_index(last_vertex_index + 2);

            let mut a = self.vertices[a_index].clone();
            let mut b = self.vertices[b_index].clone();
            let mut c = self.vertices[c_index].clone();

            let ab = b.pos - a.pos;
            let ac = c.pos - a.pos;
            let n = ab.cross(ac).get_normalized();

            a.ext.normal = n;
            b.ext.normal = n;
            c.ext.normal = n;

            triangle_mesh.vertices.push(a);
            triangle_mesh.vertices.push(b);
            triangle_mesh.vertices.push(c);
        }

        Ok(())
    }

    fn parse_ambient<'w>(
        &mut self,
        mut words: impl Iterator<Item = &'w str>,
        model: &mut Model,
    ) -> Result<(), Box<dyn Error>> {
        // Process any pending primitive before starting working on a new one
        self.process_primitive(model);

        let r = words.next().expect("Failed to read ambient r").parse()?;
        let g = words.next().expect("Failed to read ambient g").parse()?;
        let b = words.next().expect("Failed to read ambient b").parse()?;

        self.temp_material.ambient = Color::new(r, g, b, 1.0);

        Ok(())
    }

    fn parse_sphere<'w>(
        &mut self,
        mut words: impl Iterator<Item = &'w str>,
        model: &mut Model,
    ) -> Result<(), Box<dyn Error>> {
        // Process any pending primitive before starting working on a new one
        self.process_primitive(model);

        let x = words.next().expect("Failed to read center x").parse()?;
        let y = words.next().expect("Failed to read center y").parse()?;
        let z = words.next().expect("Failed to read center z").parse()?;
        let center = Point3::new(x, y, z);

        let radius = words.next().expect("Failed to read radius").parse()?;

        // Create or get current primitive
        if self.temp_model.primitives.is_empty() {
            let sphere = Sphere::builder().center(center).radius(radius).build();
            let geometry_handle = self.temp_model.geometries.push(Geometry::Sphere(sphere));
            self.temp_model
                .primitives
                .push(Primitive::builder().geometry(geometry_handle).build());
        }

        Ok(())
    }

    fn parse_translate<'w>(
        &mut self,
        mut words: impl Iterator<Item = &'w str>,
    ) -> Result<(), Box<dyn Error>> {
        let x = words
            .next()
            .expect("Failed to read translation x")
            .parse()?;
        let y = words
            .next()
            .expect("Failed to read translation y")
            .parse()?;
        let z = words
            .next()
            .expect("Failed to read translation z")
            .parse()?;

        let trs = Trs::builder().translation(Vec3::new(x, y, z)).build();

        let curr = self.transform.last_mut().unwrap();
        curr.left_mul(&trs);

        Ok(())
    }

    fn parse_rotate<'w>(
        &mut self,
        mut words: impl Iterator<Item = &'w str>,
    ) -> Result<(), Box<dyn Error>> {
        let x = words.next().expect("Failed to read rotate x").parse()?;
        let y = words.next().expect("Failed to read rotate y").parse()?;
        let z = words.next().expect("Failed to read rotate z").parse()?;
        let angle: f32 = words.next().expect("Failed to read rotate angle").parse()?;

        let trs = Trs::builder()
            .rotation(Quat::axis_angle(Vec3::new(x, y, z), angle.to_radians()))
            .build();

        let curr = self.transform.last_mut().unwrap();
        curr.left_mul(&trs);

        Ok(())
    }

    fn parse_scale<'w>(
        &mut self,
        mut words: impl Iterator<Item = &'w str>,
    ) -> Result<(), Box<dyn Error>> {
        let x = words.next().expect("Failed to read scale x").parse()?;
        let y = words.next().expect("Failed to read scale y").parse()?;
        let z = words.next().expect("Failed to read scale z").parse()?;

        let trs = Trs::builder().scale(Vec3::new(x, y, z)).build();

        let curr = self.transform.last_mut().unwrap();
        curr.left_mul(&trs);

        Ok(())
    }

    fn parse_emission<'w>(
        &mut self,
        mut words: impl Iterator<Item = &'w str>,
        model: &mut Model,
    ) -> Result<(), Box<dyn Error>> {
        // Process any pending primitive before editing current material
        self.process_primitive(model);

        let r = words.next().expect("Failed to read emission r").parse()?;
        let g = words.next().expect("Failed to read emission g").parse()?;
        let b = words.next().expect("Failed to read emission b").parse()?;

        self.temp_material.emission = Color::new(r, g, b, 1.0);

        Ok(())
    }

    fn parse_diffuse<'w>(
        &mut self,
        mut words: impl Iterator<Item = &'w str>,
        model: &mut Model,
    ) -> Result<(), Box<dyn Error>> {
        // Process any pending primitive before editing current material
        self.process_primitive(model);

        let r = words.next().expect("Failed to read diffuse r").parse()?;
        let g = words.next().expect("Failed to read diffuse g").parse()?;
        let b = words.next().expect("Failed to read diffuse b").parse()?;

        self.temp_material.diffuse = Color::new(r, g, b, 1.0);

        Ok(())
    }

    fn parse_specular<'w>(
        &mut self,
        mut words: impl Iterator<Item = &'w str>,
        model: &mut Model,
    ) -> Result<(), Box<dyn Error>> {
        // Process any pending primitive before editing current material
        self.process_primitive(model);

        let r = words.next().expect("Failed to read specular r").parse()?;
        let g = words.next().expect("Failed to read specular g").parse()?;
        let b = words.next().expect("Failed to read specular b").parse()?;

        self.temp_material.specular = Color::new(r, g, b, 1.0);

        Ok(())
    }

    fn parse_shininess<'w>(
        &mut self,
        mut words: impl Iterator<Item = &'w str>,
        model: &mut Model,
    ) -> Result<(), Box<dyn Error>> {
        // Process any pending primitive before editing current material
        self.process_primitive(model);

        self.temp_material.shininess = words.next().expect("Failed to read shininess").parse()?;

        Ok(())
    }

    fn parse_point<'w>(
        &mut self,
        mut words: impl Iterator<Item = &'w str>,
        model: &mut Model,
    ) -> Result<(), Box<dyn Error>> {
        let x = words
            .next()
            .expect("Failed to read point light x")
            .parse()?;
        let y = words
            .next()
            .expect("Failed to read point light y")
            .parse()?;
        let z = words
            .next()
            .expect("Failed to read point light z")
            .parse()?;
        let r = words
            .next()
            .expect("Failed to read point light r")
            .parse()?;
        let g = words
            .next()
            .expect("Failed to read point light g")
            .parse()?;
        let b = words
            .next()
            .expect("Failed to read point light b")
            .parse()?;

        let mut light = PointLight::new();
        light.color = Color::new(r, g, b, 1.0);
        light.attenuation = self.attenuation;
        let light = Light::Point(light);
        let light_handle = model.lights.push(light);
        let point_node = Node::builder()
            .trs(Trs::builder().translation(Vec3::new(x, y, z)).build())
            .light(light_handle)
            .build();
        let node_handle = model.nodes.push(point_node);
        model.root.children.push(node_handle);

        Ok(())
    }

    fn parse_directional<'w>(
        &mut self,
        mut words: impl Iterator<Item = &'w str>,
        model: &mut Model,
    ) -> Result<(), Box<dyn Error>> {
        let x = words
            .next()
            .expect("Failed to read light direction x")
            .parse()?;
        let y = words
            .next()
            .expect("Failed to read light direction y")
            .parse()?;
        let z = words
            .next()
            .expect("Failed to read light direction z")
            .parse()?;
        let r = words
            .next()
            .expect("Failed to read directional light r")
            .parse()?;
        let g = words
            .next()
            .expect("Failed to read directional light g")
            .parse()?;
        let b = words
            .next()
            .expect("Failed to read directional light b")
            .parse()?;

        let light_color = Color::new(r, g, b, 1.0);
        let light = DirectionalLight::new(light_color, 1.0);
        let light = Light::Directional(light);
        let light_handle = model.lights.push(light);
        let point_node = Node::builder()
            .trs(
                Trs::builder()
                    .rotation(Quat::angle_between(Vec3::X_AXIS, -Vec3::new(x, y, z)))
                    .build(),
            )
            .light(light_handle)
            .build();

        let node_handle = model.nodes.push(point_node);
        model.root.children.push(node_handle);

        Ok(())
    }

    fn parse_attenuation<'w>(
        &mut self,
        mut words: impl Iterator<Item = &'w str>,
    ) -> Result<(), Box<dyn Error>> {
        let c = words.next().expect("Failed to read attenuation").parse()?;
        let l = words.next().expect("Failed to read attenuation").parse()?;
        let q = words.next().expect("Failed to read attenuation").parse()?;
        self.attenuation = Vec3::new(c, l, q);
        Ok(())
    }

    fn parse_maxdepth<'w>(
        &mut self,
        mut words: impl Iterator<Item = &'w str>,
    ) -> Result<(), Box<dyn Error>> {
        self.config.max_depth = words.next().expect("Failed to read maxdepth").parse()?;
        Ok(())
    }

    fn parse_integrator<'w>(
        &mut self,
        mut words: impl Iterator<Item = &'w str>,
    ) -> Result<(), Box<dyn Error>> {
        self.config.integrator = words.next().expect("Failed to read integrator").parse()?;
        Ok(())
    }

    fn parse_quadlight<'w>(
        &mut self,
        mut words: impl Iterator<Item = &'w str>,
        model: &mut Model,
    ) -> Result<(), Box<dyn Error>> {
        // Quad light translation
        let a = Vec3::new(
            words
                .next()
                .expect("Failed to read quad light a.x")
                .parse()?,
            words
                .next()
                .expect("Failed to read quad light a.y")
                .parse()?,
            words
                .next()
                .expect("Failed to read quad light a.z")
                .parse()?,
        );

        let ab = Vec3::new(
            words
                .next()
                .expect("Failed to read quad light ab.x")
                .parse()?,
            words
                .next()
                .expect("Failed to read quad light ab.y")
                .parse()?,
            words
                .next()
                .expect("Failed to read quad light ab.z")
                .parse()?,
        );

        let ac = Vec3::new(
            words
                .next()
                .expect("Failed to read quad light ac.x")
                .parse()?,
            words
                .next()
                .expect("Failed to read quad light ac.y")
                .parse()?,
            words
                .next()
                .expect("Failed to read quad light ac.z")
                .parse()?,
        );

        let color = Color::new(
            words
                .next()
                .expect("Failed to read quad light color.r")
                .parse()?,
            words
                .next()
                .expect("Failed to read quad light color.g")
                .parse()?,
            words
                .next()
                .expect("Failed to read quad light color.b")
                .parse()?,
            1.0,
        );

        let material_handle = model
            .phong_materials
            .push(PhongMaterial::builder().emission(color).build());
        let material_handle = model.materials.push(Material::Phong(material_handle));

        let quad_light = QuadLight::new(ab, ac, color, material_handle);
        let light = Light::Quad(quad_light);
        let light_handle = model.lights.push(light);
        let point_node = Node::builder()
            .trs(Trs::builder().translation(a).build())
            .light(light_handle)
            .build();
        let node_handle = model.nodes.push(point_node);
        model.root.children.push(node_handle);

        Ok(())
    }
    fn parse_light_samples<'w>(
        &mut self,
        mut words: impl Iterator<Item = &'w str>,
    ) -> Result<(), Box<dyn Error>> {
        self.config.light_samples = words
            .next()
            .expect("Failed to read light samples")
            .parse()?;
        Ok(())
    }

    fn parse_light_stratify<'w>(
        &mut self,
        mut words: impl Iterator<Item = &'w str>,
    ) -> Result<(), Box<dyn Error>> {
        let word = words.next().expect("Failed to read light_stratify");
        self.config.light_stratify = word == "on";
        Ok(())
    }

    fn parse_spp<'w>(
        &mut self,
        mut words: impl Iterator<Item = &'w str>,
    ) -> Result<(), Box<dyn Error>> {
        self.config.samples_per_pixel = words.next().expect("Failed to read spp").parse()?;
        Ok(())
    }

    fn parse_next_event_estimation<'w>(
        &mut self,
        mut words: impl Iterator<Item = &'w str>,
    ) -> Result<(), Box<dyn Error>> {
        let word = words.next().expect("Failed to read nexteventestimation");
        self.config.next_event_estimation = word == "on";
        Ok(())
    }

    fn parse_roussian_roulette<'w>(
        &mut self,
        mut words: impl Iterator<Item = &'w str>,
    ) -> Result<(), Box<dyn Error>> {
        let word = words.next().expect("Failed to read russianroulette");
        self.config.russian_roulette = word == "on";
        Ok(())
    }

    fn parse_line(&mut self, line: String, model: &mut Model) -> Result<(), Box<dyn Error>> {
        // Skip comments
        if line.starts_with('#') {
            return Ok(());
        }
        // Skip empty lines
        if line.find(char::is_alphanumeric).is_none() {
            return Ok(());
        }

        // Get words one by one
        let mut words = line.split(' ').filter(|w| !w.is_empty());

        let command = words.next();

        match command {
            Some("size") => self.parse_size(words)?,
            Some("camera") => Self::parse_camera(words, model)?,
            Some("maxverts") => self.parse_maxverts(words)?,
            Some("vertex") => self.parse_vertex(words)?,
            Some("tri") => self.parse_tri(words)?,
            Some("ambient") => self.parse_ambient(words, model)?,
            Some("sphere") => self.parse_sphere(words, model)?,
            Some("translate") => self.parse_translate(words)?,
            Some("rotate") => self.parse_rotate(words)?,
            Some("scale") => self.parse_scale(words)?,
            Some("pushTransform") => self.transform.push(Trs::default()),
            Some("popTransform") => {
                self.process_primitive(model);
                if !self.transform.is_empty() {
                    _ = self.transform.pop().unwrap()
                }
            }
            Some("emission") => self.parse_emission(words, model)?,
            Some("diffuse") => self.parse_diffuse(words, model)?,
            Some("specular") => self.parse_specular(words, model)?,
            Some("shininess") => self.parse_shininess(words, model)?,
            Some("point") => self.parse_point(words, model)?,
            Some("directional") => self.parse_directional(words, model)?,
            Some("attenuation") => self.parse_attenuation(words)?,
            Some("maxdepth") => self.parse_maxdepth(words)?,
            Some("integrator") => self.parse_integrator(words)?,
            Some("quadLight") => self.parse_quadlight(words, model)?,
            Some("lightsamples") => self.parse_light_samples(words)?,
            Some("lightstratify") => self.parse_light_stratify(words)?,
            Some("spp") => self.parse_spp(words)?,
            Some("nexteventestimation") => self.parse_next_event_estimation(words)?,
            Some("russianroulette") => self.parse_roussian_roulette(words)?,
            _ => log::warn!("Skipping command: {}", line),
        }

        Ok(())
    }

    /// Processes latest pending primitive from the temporary model to the output model
    fn process_primitive(&mut self, model: &mut Model) {
        if let Some(mut primitive) = self.temp_model.primitives.pop() {
            if let Some(geometry) = self.temp_model.geometries.get(primitive.geometry) {
                primitive.geometry = model.geometries.push(geometry.clone());
            }

            let phong_material_handle = model.phong_materials.push(self.temp_material.clone());
            let material = Material::Phong(phong_material_handle);
            primitive.material = model.materials.push(material);

            let primitive_handle = model.primitives.push(primitive);
            let mesh = Mesh::builder().primitive(primitive_handle).build();
            let mesh_handle = model.meshes.push(mesh);

            // Handle transform stack
            let mut trs = Trs::default();
            for transform in self.transform.iter() {
                trs.left_mul(transform);
            }

            let node = Node::builder().trs(trs).mesh(mesh_handle).build();
            let node_handle = model.nodes.push(node);
            model.root.children.push(node_handle);
        }
    }

    pub fn build(mut self) -> Result<(Model, SdtfConfig), Box<dyn Error>> {
        let mut model = Model::default();

        if let Some(string) = self.string.clone() {
            let bytes = string.as_bytes();
            let reader = BufReader::new(Box::new(bytes));
            for line in reader.lines() {
                self.parse_line(line?, &mut model)?;
            }
        } else if let Some(path) = self.path.clone() {
            let path_str = path.to_string_lossy();
            let msg = format!("Loading UCSD scene from: {}", path_str);
            log::info!("{}", msg);
            let file = File::open(&path).expect(&msg);
            let reader = BufReader::new(Box::new(file));
            for line in reader.lines() {
                self.parse_line(line?, &mut model)?;
            }
        } else {
            log::error!("No path or string provided to load UCSD scene");
            panic!();
        }

        self.process_primitive(&mut model);

        Ok((model, self.config))
    }
}

impl Model {
    pub fn load_sdtf_path(path: impl AsRef<Path>) -> Result<(Model, SdtfConfig), Box<dyn Error>> {
        let (model, config) = SdtfBuilder::new().path(path).build()?;
        Ok((model, config))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn size() {
        let (_, config) = SdtfBuilder::new()._str("size 320 240").build().unwrap();

        assert_eq!(config.width, 320);
        assert_eq!(config.height, 240);
    }

    #[test]
    fn camera() {
        let (model, _config) = SdtfBuilder::new()
            ._str("camera -4 -4 4 1 0 0 0 1 0 30")
            .build()
            .unwrap();

        let camera = &model.cameras[0];
        assert_eq!(camera.yfov_radians, 30.0f32.to_radians());
    }

    #[test]
    fn triangle() {
        let (model, _) = SdtfBuilder::new()
            ._str(
                r#"
maxverts 3
vertex -1 -1 0
vertex +1 -1 0
vertex +1 +1 0
tri 0 1 2"#,
            )
            .build()
            .unwrap();

        let geometry_handle = model.primitives[0].geometry;
        let geometry = model.get_geometry(geometry_handle).unwrap();
        let Geometry::TriangleMesh(triangles) = geometry else {
            panic!("Failed to get triangles");
        };
        assert_eq!(triangles.vertices.len(), 3);
    }

    #[test]
    fn load1() {
        let (model, _config) = SdtfBuilder::new()
            .path(tests::get_model_path().join("sdtf/1/scene1.sdtf"))
            .build()
            .unwrap();

        let geometry_handle = model.primitives[0].geometry;
        let geomtry = model.get_geometry(geometry_handle).unwrap();
        let Geometry::TriangleMesh(triangles) = geomtry else {
            panic!("Failed to get triangles");
        };
        // One primitive with two triangles hence six vertices
        assert_eq!(triangles.vertices.len(), 6);
    }

    #[test]
    fn load2() {
        let (model, _config) = SdtfBuilder::new()
            .path(tests::get_model_path().join("sdtf/1/scene2.sdtf"))
            .build()
            .unwrap();

        let geometry_handle = model.primitives[0].geometry;
        let geometry = model.get_geometry(geometry_handle).unwrap();
        let Geometry::TriangleMesh(triangles) = geometry else {
            panic!("Failed to get triangles");
        };
        assert_eq!(triangles.vertices.len(), 6);
        assert_eq!(model.primitives.len(), 27);
    }
}
