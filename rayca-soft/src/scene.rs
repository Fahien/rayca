// Copyright © 2022-2024
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::{collections::HashMap, error::Error, path::Path};

#[cfg(feature = "parallel")]
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};

use super::*;

#[derive(Default)]
pub struct SoftRenderer {
    pub config: Config,
}

impl SoftRenderer {
    /// This can be used for default values which are not defined in any other model in the scene
    pub fn create_default_model() -> Model {
        let mut model = Model::default();

        // Add 1 camera
        let camera = Camera::default();
        let camera_handle = model.cameras.push(camera);
        let camera_node = Node::builder()
            .camera(camera_handle)
            .trs(Trs::builder().translation(Vec3::new(0.0, 0.0, 4.0)).build())
            .build();
        let camera_node_handle = model.nodes.push(camera_node);
        model.root.children.push(camera_node_handle);

        // Add 2 point lights
        let mut light = Light::point();
        light.set_intensity(64.0);
        let light_handle = model.lights.push(light);

        let light_node = Node::builder()
            .trs(
                Trs::builder()
                    .translation(Vec3::new(-1.0, 2.0, 1.0))
                    .build(),
            )
            .light(light_handle)
            .build();
        let light_node_handle = model.nodes.push(light_node);
        model.root.children.push(light_node_handle);

        let point_light_node = Node::builder()
            .light(light_handle)
            .trs(Trs::builder().translation(Vec3::new(1.0, 2.0, 1.0)).build())
            .build();
        let light_node_handle = model.nodes.push(point_light_node);
        model.root.children.push(light_node_handle);

        model
    }

    pub fn new_with_config(config: Config) -> Self {
        let mut scene = Self::default();
        scene.config = config;
        scene
    }

    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let mut timer = Timer::new();
        let path_str = path.as_ref().to_string_lossy().to_string();

        // Open glTF model
        let assets = Assets::new();
        let _model = Model::load_gltf_path(path, &assets)?;

        log::info!(
            "Loaded {} in {:.2}ms",
            path_str,
            timer.get_delta().as_millis()
        );
        Ok(())
    }
}

fn draw_pixel<I: AsRef<dyn Integrator>>(
    integrator: &I,
    scene: &SceneDrawInfo,
    ray: Ray,
    bvh: &Bvh,
    pixel: &mut RGBA8,
) -> usize {
    let triangle_count = 0;
    if let Some(pixel_color) = integrator.as_ref().trace(scene, ray, bvh, 0) {
        // No over operation here as transparency should be handled by the lighting model
        *pixel = pixel_color.into();
    }
    triangle_count
}

impl Draw for SoftRenderer {
    fn draw(&mut self, scene: &Scene, image: &mut Image) {
        let scene_draw_info = SceneDrawInfo::new(scene);

        let primitives = BvhPrimitive::from_scene(&scene_draw_info);

        // Build BVH
        let mut bvh_builder = Bvh::builder().primitives(primitives);
        if !self.config.bvh {
            bvh_builder = bvh_builder.max_depth(0);
        }
        let bvh = bvh_builder.build(&scene_draw_info);

        let mut timer = Timer::new();

        let width = image.width() as f32;
        let height = image.height() as f32;

        let inv_width = 1.0 / width;
        let inv_height = 1.0 / height;

        assert!(!scene_draw_info.camera_draw_infos.is_empty());
        let camera_draw_info = scene_draw_info.camera_draw_infos[0];
        let camera = scene_draw_info.get_camera(camera_draw_info);
        let camera_trs = scene_draw_info.get_world_trs(camera_draw_info);

        let aspectratio = width / height;
        let angle = camera.get_angle();

        #[cfg(feature = "parallel")]
        let row_iter = image.pixels_mut().into_par_iter();
        #[cfg(not(feature = "parallel"))]
        let row_iter = image.pixels_mut().into_iter();

        row_iter.enumerate().for_each(|(y, row)| {
            #[cfg(feature = "parallel")]
            let pixel_iter = row.into_par_iter();
            #[cfg(not(feature = "parallel"))]
            let pixel_iter = row.into_iter();

            pixel_iter.enumerate().for_each(|(x, pixel)| {
                // Generate primary ray
                let xx = (2.0 * ((x as f32 + 0.5) * inv_width) - 1.0) * angle * aspectratio;
                let yy = (1.0 - 2.0 * ((y as f32 + 0.5) * inv_height)) * angle;
                let mut dir = Vec3::new(xx, yy, -1.0);
                dir.normalize();
                let origin = Point3::new(0.0, 0.0, 0.0);
                let ray = &camera_trs.trs * Ray::new(origin, dir);

                draw_pixel(&self.config.integrator, &scene_draw_info, ray, &bvh, pixel);
            });
        });

        log::info!("Rendered in {:.2}ms", timer.get_delta().as_millis());
    }
}

pub struct SceneDrawInfo<'a> {
    /// World transforms for each node that can be used by a renderer. This is a map because the
    /// renderer may not need all nodes. Some nodes do not have cameras, lights, or meshes, so we
    /// only store the transforms for the nodes that are actually used.
    pub world_trss: HashMap<NodeDrawInfo, WorldTrs>,

    /// List of nodes with meshes
    pub mesh_draw_infos: Vec<MeshDrawInfo>,

    /// List of nodes with cameras
    pub camera_draw_infos: Vec<CameraDrawInfo>,

    /// List of nodes with lights
    pub light_draw_infos: Vec<LightDrawInfo>,

    scene: &'a Scene,
}

// Safety: This struct is safe to send and share between threads because it only contains references to
// the scene, which is immutable. The `Scene` itself is not modified after being created, so it is safe to
// share across threads.
unsafe impl<'a> Send for SceneDrawInfo<'a> {}
unsafe impl<'a> Sync for SceneDrawInfo<'a> {}

impl<'a> SceneDrawInfo<'a> {
    pub fn new(scene: &'a Scene) -> Self {
        let mut ret = Self {
            world_trss: HashMap::new(),
            mesh_draw_infos: Vec::new(),
            camera_draw_infos: Vec::new(),
            light_draw_infos: Vec::new(),
            scene,
        };

        ret.traverse_scene();

        ret
    }

    fn traverse_scene(&mut self) {
        let root_trs = self.scene.root.trs.clone();
        for node in self.scene.root.children.clone().into_iter() {
            self.traverse_scene_nodes(&root_trs, node);
        }
    }

    fn traverse_scene_nodes(&mut self, transform: &Trs, node: Handle<Node>) {
        let current_node = self.scene.nodes.get(node).unwrap();
        let current_trs = transform * &current_node.trs;

        if let Some(model_handle) = current_node.model {
            let model = self.scene.get_model(model_handle).unwrap();
            self.traverse_model(&current_trs, model_handle, model);
        }

        for child in current_node.children.clone().into_iter() {
            self.traverse_scene_nodes(&current_trs, child);
        }
    }

    fn traverse_model(&mut self, transform: &Trs, model_handle: Handle<Model>, model: &Model) {
        let current_trs = transform * &model.root.trs;
        for node in model.root.children.clone().into_iter() {
            self.traverse_model_nodes(&current_trs, node, model_handle, model);
        }
    }

    fn traverse_model_nodes(
        &mut self,
        transform: &Trs,
        node: Handle<Node>,
        model_handle: Handle<Model>,
        model: &Model,
    ) {
        let current_node = model.nodes.get(node).unwrap();
        let current_transform = transform * &current_node.trs;

        if current_node.mesh.is_some()
            || current_node.camera.is_some()
            || current_node.light.is_some()
        {
            let node_draw_info = NodeDrawInfo::new(node, model_handle);
            self.world_trss
                .insert(node_draw_info, WorldTrs::new(current_transform.clone()));
        }
        if current_node.mesh.is_some() {
            let mesh_draw_info = MeshDrawInfo::new(node, model_handle);
            self.mesh_draw_infos.push(mesh_draw_info);
        }
        if current_node.light.is_some() {
            let light_draw_info = LightDrawInfo::new(node, model_handle);
            self.light_draw_infos.push(light_draw_info);
        }
        if current_node.camera.is_some() {
            let camera_draw_info = CameraDrawInfo::new(node, model_handle);
            self.camera_draw_infos.push(camera_draw_info);
        }

        for child in current_node.children.clone().into_iter() {
            self.traverse_model_nodes(&current_transform, child, model_handle, model);
        }
    }

    pub fn get_world_trs(&self, node: NodeDrawInfo) -> &WorldTrs {
        self.world_trss.get(&node).unwrap()
    }

    pub fn get_model(&self, model: Handle<Model>) -> &Model {
        self.scene.models.get(model).unwrap()
    }

    pub fn get_mesh(&self, mesh_draw_info: MeshDrawInfo) -> &Mesh {
        let node = self.get_node(mesh_draw_info);
        let model = self.get_model(mesh_draw_info.model);
        model.get_mesh(node.mesh.unwrap()).unwrap()
    }

    pub fn get_node(&self, node_draw_info: NodeDrawInfo) -> &Node {
        self.get_model(node_draw_info.model)
            .get_node(node_draw_info.node)
            .unwrap()
    }

    pub fn get_camera(&self, camera_draw_info: CameraDrawInfo) -> &Camera {
        let node = self.get_node(camera_draw_info);
        self.get_model(camera_draw_info.model)
            .get_camera(node.camera.unwrap())
            .unwrap()
    }

    pub fn get_light(&self, light_draw_info: LightDrawInfo) -> &Light {
        let node = self.get_node(light_draw_info);
        self.get_model(light_draw_info.model)
            .get_light(node.light.unwrap())
            .unwrap()
    }
}

#[derive(Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct NodeDrawInfo {
    pub node: Handle<Node>,
    pub model: Handle<Model>,
}

impl NodeDrawInfo {
    pub fn new(node: Handle<Node>, model: Handle<Model>) -> Self {
        Self { node, model }
    }
}

pub type MeshDrawInfo = NodeDrawInfo;
pub type CameraDrawInfo = NodeDrawInfo;
pub type LightDrawInfo = NodeDrawInfo;
