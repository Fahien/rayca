// Copyright © 2022-2024
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::{
    collections::HashMap,
    error::Error,
    ops::Deref,
    path::{Path, PathBuf},
};

use gltf::Gltf;

#[cfg(feature = "parallel")]
use rayon::iter::{ParallelBridge, ParallelIterator};

use super::*;

fn data_type_as_size(data_type: gltf::accessor::DataType) -> usize {
    match data_type {
        gltf::accessor::DataType::I8 => 1,
        gltf::accessor::DataType::U8 => 1,
        gltf::accessor::DataType::I16 => 2,
        gltf::accessor::DataType::U16 => 2,
        gltf::accessor::DataType::U32 => 4,
        gltf::accessor::DataType::F32 => 4,
    }
}

fn dimensions_as_size(dimensions: gltf::accessor::Dimensions) -> usize {
    match dimensions {
        gltf::accessor::Dimensions::Scalar => 1,
        gltf::accessor::Dimensions::Vec2 => 2,
        gltf::accessor::Dimensions::Vec3 => 3,
        gltf::accessor::Dimensions::Vec4 => 4,
        gltf::accessor::Dimensions::Mat2 => 4,
        gltf::accessor::Dimensions::Mat3 => 9,
        gltf::accessor::Dimensions::Mat4 => 16,
    }
}

fn get_stride(accessor: &gltf::Accessor) -> usize {
    if let Some(view) = accessor.view() {
        if let Some(stride) = view.stride() {
            return stride;
        }
    }

    data_type_as_size(accessor.data_type()) * dimensions_as_size(accessor.dimensions())
}

#[derive(Default)]
pub struct ModelBuilder {
    uri_buffers: Vec<Vec<u8>>,
    parent_dir: Option<PathBuf>,
    gltf: Option<Gltf>,
}

impl ModelBuilder {
    pub fn new() -> Self {
        Self {
            uri_buffers: vec![],
            parent_dir: None,
            gltf: None,
        }
    }

    /// Creates a model loading a GLTF file
    pub fn path<P: AsRef<Path>>(mut self, path: P) -> Result<Self, Box<dyn Error>> {
        self.parent_dir = Some(
            path.as_ref()
                .parent()
                .ok_or("Failed to get parent directory")?
                .into(),
        );
        self.gltf = Some(Gltf::open(path)?);

        Ok(self)
    }

    pub fn data(mut self, data: &[u8]) -> Result<Self, Box<dyn Error>> {
        self.gltf = Some(Gltf::from_slice(data)?);
        Ok(self)
    }

    pub fn load_images(&mut self, images: &mut Pack<Image>) {
        if self.gltf.is_none() {
            return;
        }

        let gltf = self.gltf.as_ref().unwrap();

        let mut timer = Timer::new();

        #[cfg(feature = "parallel")]
        let images_iter = gltf.images().enumerate().par_bridge();
        #[cfg(not(feature = "parallel"))]
        let images_iter = gltf.images().enumerate();

        let mut vec: Vec<Image> = images_iter
            .map(|(id, image)| {
                match image.source() {
                    gltf::image::Source::View { .. } => todo!("Implement image source view"),
                    gltf::image::Source::Uri { uri, .. } => {
                        const DATA_URI: &str = "data:image/png;base64,";

                        let mut image = if uri.starts_with(DATA_URI) {
                            let (_, data_base64) = uri.split_at(DATA_URI.len());
                            let data = base64::decode(data_base64)
                                .expect("Failed to decode base64 image data");

                            Image::load_png_data(&data)
                        } else if let Some(parent_dir) = &self.parent_dir {
                            // Join gltf parent dir to URI
                            let path = parent_dir.join(uri);
                            Image::load_file(path)
                        } else {
                            unimplemented!()
                        };

                        image.id = id;
                        image
                    }
                }
            })
            .collect();

        vec.sort_by_key(|image| image.id);

        print_info!(
            "Loaded",
            "images from file in {:.2}s",
            timer.get_delta().as_secs_f32()
        );

        *images = Pack::from(vec);
    }

    pub fn load_textures(&mut self, textures: &mut Pack<Texture>) {
        if self.gltf.is_none() {
            return;
        }
        let gltf = self.gltf.as_ref().unwrap();

        let vec: Vec<Texture> = gltf
            .textures()
            .map(|gtexture| {
                let image = Handle::new(gtexture.source().index());
                let sampler = Handle::none();
                Texture::new(image, sampler)
            })
            .collect();

        *textures = Pack::from(vec);
    }

    fn load_uri_buffers(&mut self) -> Result<(), Box<dyn Error>> {
        if self.gltf.is_none() {
            return Ok(());
        }
        let gltf = self.gltf.as_ref().unwrap();

        for buffer in gltf.buffers() {
            match buffer.source() {
                gltf::buffer::Source::Uri(uri) => {
                    const DATA_URI: &str = "data:application/octet-stream;base64,";

                    let data = if uri.starts_with(DATA_URI) {
                        let (_, data_base64) = uri.split_at(DATA_URI.len());
                        base64::decode(data_base64)?
                    } else if let Some(parent_dir) = &self.parent_dir {
                        let uri = parent_dir.join(uri);
                        std::fs::read(uri)?
                    } else {
                        unimplemented!();
                    };
                    assert!(buffer.index() == self.uri_buffers.len());
                    self.uri_buffers.push(data);
                }
                _ => unimplemented!(),
            }
        }

        Ok(())
    }

    fn get_data_start(&self, accessor: &gltf::Accessor) -> &[u8] {
        let view = accessor.view().unwrap();
        let view_len = view.length();

        let buffer = view.buffer();
        if let gltf::buffer::Source::Bin = buffer.source() {
            unimplemented!()
        }

        let view_offset = view.offset();
        let accessor_offset = accessor.offset();
        let offset = accessor_offset + view_offset;
        assert!(offset < buffer.length());
        let end_offset = view_offset + view_len;
        assert!(end_offset <= buffer.length());

        let data = &self.uri_buffers[buffer.index()];
        &data[offset..end_offset]
    }

    pub fn build(&mut self) -> Result<Model, Box<dyn Error>> {
        let mut model = Model::new();

        self.load_images(&mut model.images);
        self.load_textures(&mut model.textures);
        self.load_uri_buffers()?;
        self.load_materials(&mut model.materials)?;
        self.load_meshes(&mut model)?;
        self.load_cameras(&mut model.cameras)?;
        self.load_nodes(&mut model);

        // TODO collect lights from glTF file

        Ok(model)
    }

    pub fn load_cameras(&mut self, cameras: &mut Pack<Camera>) -> Result<(), Box<dyn Error>> {
        if self.gltf.is_none() {
            return Ok(());
        }
        let gltf = self.gltf.as_ref().unwrap();

        for gcamera in gltf.cameras() {
            let camera = match gcamera.projection() {
                gltf::camera::Projection::Perspective(p) => {
                    let aspect_ratio = p.aspect_ratio().unwrap_or(1.0);
                    let yfov = p.yfov();
                    let near = p.znear();
                    if let Some(far) = p.zfar() {
                        Camera::finite_perspective(aspect_ratio, yfov, near, far)
                    } else {
                        Camera::infinite_perspective(aspect_ratio, yfov, near)
                    }
                }
                gltf::camera::Projection::Orthographic(o) => {
                    let width = o.xmag();
                    let height = o.ymag();
                    let near = o.znear();
                    let far = o.zfar();
                    Camera::orthographic(width, height, near, far)
                }
            };
            cameras.push(camera);
        }
        Ok(())
    }

    pub fn load_materials(&mut self, materials: &mut Pack<Material>) -> Result<(), Box<dyn Error>> {
        if self.gltf.is_none() {
            return Ok(());
        }
        let gltf = self.gltf.as_ref().unwrap();

        for gmaterial in gltf.materials() {
            let mut material = Material::new();

            let pbr = gmaterial.pbr_metallic_roughness();

            // Load base color
            let gcolor = pbr.base_color_factor();
            let color = Color::new(gcolor[0], gcolor[1], gcolor[2], gcolor[3]);
            material.color = color;

            // Load albedo
            if let Some(gtexture) = pbr.base_color_texture() {
                material.albedo_texture = Handle::new(gtexture.texture().index());
            }

            // Load normal
            if let Some(gtexture) = gmaterial.normal_texture() {
                material.normal_texture = Handle::new(gtexture.texture().index());
            }

            // Load metallic roughness factors and texture
            material.metallic_factor = pbr.metallic_factor();
            material.roughness_factor = pbr.roughness_factor();
            if let Some(gtexture) = pbr.metallic_roughness_texture() {
                material.metallic_roughness_texture = Handle::new(gtexture.texture().index());
            }

            materials.push(material);
        }

        Ok(())
    }

    fn load_vertices(&self, gprimitive: &gltf::Primitive) -> Result<Vec<Vertex>, Box<dyn Error>> {
        let mut vertices = vec![];

        let mode = gprimitive.mode();
        assert!(mode == gltf::mesh::Mode::Triangles);

        // Load normals first, so we can process tangents later
        for (semantic, accessor) in gprimitive.attributes() {
            if semantic == gltf::mesh::Semantic::Normals {
                self.load_normals(&mut vertices, &accessor)?;
            }
        }

        for (semantic, accessor) in gprimitive.attributes() {
            match semantic {
                gltf::mesh::Semantic::Positions => self.load_positions(&mut vertices, &accessor)?,
                gltf::mesh::Semantic::TexCoords(_) => self.load_uvs(&mut vertices, &accessor)?,
                gltf::mesh::Semantic::Colors(_) => self.load_colors(&mut vertices, &accessor)?,
                gltf::mesh::Semantic::Normals => (), // Already loaded
                gltf::mesh::Semantic::Tangents => self.load_tangents(&mut vertices, &accessor)?,
                _ => print_warning!("Skipping", "semantic: {:?}", semantic),
            }
        }

        Ok(vertices)
    }

    fn load_indices(&self, gprimitive: &gltf::Primitive) -> (Vec<u8>, usize) {
        let mut indices = vec![];
        let mut index_size = 1;

        if let Some(accessor) = gprimitive.indices() {
            let data_type = accessor.data_type();
            index_size = data_type_as_size(data_type);

            // Data type can vary
            let data = self.get_data_start(&accessor);
            let d = &data[0];
            let len = accessor.count() * data_type_as_size(data_type);
            // Use bytes regardless of the index data type
            let slice: &[u8] = unsafe { std::slice::from_raw_parts(d as *const u8 as _, len) };
            indices = Vec::from(slice);
        }

        (indices, index_size)
    }

    fn load_primitive(
        &self,
        model: &mut Model,
        gprimitive: &gltf::Primitive,
    ) -> Result<Handle<Primitive>, Box<dyn Error>> {
        let vertices = self.load_vertices(gprimitive)?;
        let (indices, index_size) = self.load_indices(gprimitive);

        let material = gprimitive
            .material()
            .index()
            .map_or(Handle::none(), Handle::new);

        let primitive = Primitive::builder()
            .vertices(vertices)
            .indices(indices)
            .index_size(index_size)
            .material(material)
            .build();

        Ok(model.primitives.push(primitive))
    }

    fn load_meshes(&self, model: &mut Model) -> Result<(), Box<dyn Error>> {
        if self.gltf.is_none() {
            return Ok(());
        }
        let gltf = self.gltf.as_ref().unwrap();

        for gmesh in gltf.meshes() {
            let primitive_handles = gmesh
                .primitives()
                .into_iter()
                .map(|gprimitive| {
                    self.load_primitive(model, &gprimitive)
                        .expect("Failed to load a primitive")
                })
                .collect();

            let mesh = Mesh::new(primitive_handles);
            model.meshes.push(mesh);
        }
        Ok(())
    }

    fn get_slices<'g>(&self, accessor: &'g gltf::Accessor) -> Vec<&'g [f32]> {
        let data_type = accessor.data_type();
        assert!(data_type == gltf::accessor::DataType::F32);

        let count = accessor.count();
        let dimensions = accessor.dimensions();
        let len = match dimensions {
            gltf::accessor::Dimensions::Vec2 => 2,
            gltf::accessor::Dimensions::Vec3 => 3,
            gltf::accessor::Dimensions::Vec4 => 4,
            _ => panic!("Invalid dimensions"),
        };

        let data = self.get_data_start(accessor);
        let stride = get_stride(accessor);

        let mut ret = vec![];

        for i in 0..count {
            let offset = i * stride;
            assert!(offset < data.len());
            let d = &data[offset];
            let slice = unsafe { std::slice::from_raw_parts::<f32>(d as *const u8 as _, len) };
            ret.push(slice);
        }

        ret
    }

    fn load_positions(
        &self,
        vertices: &mut Vec<Vertex>,
        accessor: &gltf::Accessor,
    ) -> Result<(), Box<dyn Error>> {
        let positions = self.get_slices(accessor);
        vertices.resize(positions.len(), Vertex::default());
        for (i, position) in positions.into_iter().enumerate() {
            vertices[i].pos = Point3::new(position[0], position[1], position[2]);
        }
        Ok(())
    }

    fn load_uvs(
        &self,
        vertices: &mut Vec<Vertex>,
        accessor: &gltf::Accessor,
    ) -> Result<(), Box<dyn Error>> {
        let uvs = self.get_slices(accessor);
        vertices.resize(uvs.len(), Vertex::default());
        for (i, uv) in uvs.into_iter().enumerate() {
            vertices[i].uv.x = uv[0];
            vertices[i].uv.y = uv[1];
        }
        Ok(())
    }

    fn load_normals(
        &self,
        vertices: &mut Vec<Vertex>,
        accessor: &gltf::Accessor,
    ) -> Result<(), Box<dyn Error>> {
        let normals = self.get_slices(accessor);
        vertices.resize(normals.len(), Vertex::default());
        for (i, normal) in normals.into_iter().enumerate() {
            vertices[i].normal = Vec3::new(normal[0], normal[1], normal[2]);
        }
        Ok(())
    }

    fn load_colors(
        &self,
        vertices: &mut Vec<Vertex>,
        accessor: &gltf::Accessor,
    ) -> Result<(), Box<dyn Error>> {
        let colors = self.get_slices(accessor);
        vertices.resize(colors.len(), Vertex::default());
        for (i, color) in colors.into_iter().enumerate() {
            vertices[i].color.r = color[0];
            vertices[i].color.g = color[1];
            vertices[i].color.b = color[2];
            vertices[i].color.a = if color.len() == 4 { color[3] } else { 1.0 };
        }
        Ok(())
    }

    fn load_tangents(
        &self,
        vertices: &mut [Vertex],
        accessor: &gltf::Accessor,
    ) -> Result<(), Box<dyn Error>> {
        let data_type = accessor.data_type();
        assert!(data_type == gltf::accessor::DataType::F32);
        let count = accessor.count();
        assert_eq!(vertices.len(), count);
        let dimensions = accessor.dimensions();
        assert!(dimensions == gltf::accessor::Dimensions::Vec4);

        let view = accessor.view().unwrap();
        let target = view.target().unwrap_or(gltf::buffer::Target::ArrayBuffer);
        assert!(target == gltf::buffer::Target::ArrayBuffer);

        let data = self.get_data_start(accessor);
        let stride = get_stride(accessor);

        vertices.iter_mut().enumerate().for_each(|(index, vertex)| {
            let offset = index * stride;
            assert!(offset < data.len());
            let d = &data[offset];
            let tangent = unsafe { std::slice::from_raw_parts::<f32>(d as *const u8 as _, 4) };

            vertex.tangent = Vec3::new(tangent[0], tangent[1], tangent[2]);

            // Compute bitangent as for glTF 2.0 spec
            vertex.bitangent = vertex.normal.cross(&vertex.tangent) * tangent[3];
        });

        Ok(())
    }

    fn create_root(scene: &gltf::Scene) -> Node {
        Node::builder()
            .name("Root".into())
            .children(
                scene
                    .nodes()
                    .map(|gchild| Handle::new(gchild.index()))
                    .collect(),
            )
            .build()
    }

    fn create_node(gnode: &gltf::Node) -> Node {
        let transform = gnode.transform().decomposed();

        let translation = &transform.0;
        let translation = Vec3::new(translation[0], translation[1], translation[2]);

        let rotation = &transform.1;
        let rotation = Quat::new(rotation[0], rotation[1], rotation[2], rotation[3]);

        let scale = &transform.2;
        let scale = Vec3::new(scale[0], scale[1], scale[2]);

        let mut node_builder = Node::builder()
            .id(gnode.index())
            .name(gnode.name().unwrap_or("Unknown").into())
            .children(
                gnode
                    .children()
                    .map(|gchild| Handle::new(gchild.index()))
                    .collect(),
            )
            .translation(translation)
            .rotation(rotation)
            .scale(scale);

        if let Some(mesh) = gnode.mesh() {
            node_builder = node_builder.mesh(Handle::new(mesh.index()));
        }

        if let Some(camera) = gnode.camera() {
            node_builder = node_builder.camera(Handle::new(camera.index()));
        }

        node_builder.build()
    }

    fn load_nodes(&self, model: &mut Model) {
        if self.gltf.is_none() {
            return;
        }
        let gltf = self.gltf.as_ref().unwrap();

        // Load scene
        let scene = gltf.scenes().next().unwrap();
        model.root = Self::create_root(&scene);

        // Load nodes
        for gnode in gltf.nodes() {
            let node = Self::create_node(&gnode);
            model.nodes.push(node);
        }
    }
}

/// Solved transforms in world space, ready to be used by the renderer
#[repr(transparent)]
pub struct SolvedTrs {
    pub trs: Trs,
}

impl SolvedTrs {
    pub fn new(trs: Trs) -> Self {
        Self { trs }
    }
}

impl Deref for SolvedTrs {
    type Target = Trs;

    fn deref(&self) -> &Self::Target {
        &self.trs
    }
}

#[derive(Default)]
pub struct Model {
    pub id: usize,
    pub width: u32,
    pub height: u32,
    pub samplers: Pack<Sampler>,
    pub images: Pack<Image>,
    pub textures: Pack<Texture>,
    pub materials: Pack<Material>,
    pub primitives: Pack<Primitive>,
    pub meshes: Pack<Mesh>,
    pub cameras: Pack<Camera>,
    pub lights: Pack<Light>,
    pub nodes: Pack<Node>,
    pub root: Node,

    // Cleared every frame
    pub solved_trs: HashMap<Handle<Node>, SolvedTrs>,
    pub camera_nodes: Vec<Handle<Node>>,
    pub light_nodes: Vec<Handle<Node>>,
}

impl Model {
    pub fn builder() -> ModelBuilder {
        ModelBuilder::new()
    }

    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    /// Takes a loaded model and appends all its objects to the objects of the current model
    pub fn append(&mut self, mut model: Model) {
        // Create a new root node for the new model
        let new_model_root = self.nodes.push(model.root);
        self.root.children.push(new_model_root);

        let sampler_offset = self.samplers.append(&mut model.samplers);
        let image_offset = self.images.append(&mut model.images);
        // Update sampler and image handles
        for texture in model.textures.iter_mut() {
            texture.sampler.offset(sampler_offset);
            texture.image.offset(image_offset);
        }

        let texture_offset = self.textures.append(&mut model.textures);
        // Update texture handles
        for material in model.materials.iter_mut() {
            material.albedo_texture.offset(texture_offset);
            material.metallic_roughness_texture.offset(texture_offset);
        }

        let mat_offset = self.materials.append(&mut model.materials);
        // Update material handles
        for prim in model.primitives.iter_mut() {
            prim.material.offset(mat_offset);
        }

        let prim_offset = self.primitives.append(&mut model.primitives);
        // Update primitive handles
        for mesh in model.meshes.iter_mut() {
            for primitive_handle in &mut mesh.primitives {
                primitive_handle.offset(prim_offset);
            }
        }

        let light_offset = self.lights.append(&mut model.lights);
        let camera_offset = self.cameras.append(&mut model.cameras);
        let mesh_offset = self.meshes.append(&mut model.meshes);
        let node_offset = self.nodes.len();
        // Update mesh and node handles
        for node in model.nodes.iter_mut() {
            node.light.offset(light_offset);
            node.camera.offset(camera_offset);
            node.mesh.offset(mesh_offset);
            for children in &mut node.children {
                children.offset(node_offset);
            }
        }

        self.nodes.append(&mut model.nodes);
        let new_model_root = self.nodes.get_mut(new_model_root).unwrap();
        for children in &mut new_model_root.children {
            children.offset(node_offset);
        }
    }

    fn traverse(
        &self,
        solved_trs: &mut HashMap<Handle<Node>, SolvedTrs>,
        transform: Trs,
        node: Handle<Node>,
    ) {
        let current_node = self.nodes.get(node).unwrap();
        let current_transform = &transform * &current_node.trs;
        solved_trs.insert(node, SolvedTrs::new(current_transform.clone()));

        for child in &current_node.children {
            self.traverse(solved_trs, current_transform.clone(), *child);
        }
    }

    fn collect_trs(&mut self) {
        let mut ret = HashMap::new();
        for node in self.root.children.iter() {
            self.traverse(&mut ret, self.root.trs.clone(), *node);
        }
        self.solved_trs = ret;
    }

    pub fn collect(&mut self) -> Vec<BvhPrimitive> {
        self.collect_trs();

        let mut primitives = vec![];
        self.camera_nodes.clear();
        self.light_nodes.clear();

        for (node_handle, _) in &self.solved_trs {
            // Collect primitives
            let node = self.nodes.get(*node_handle).unwrap();
            if let Some(mesh) = self.meshes.get(node.mesh) {
                for prim_handle in mesh.primitives.iter() {
                    let prim = self.primitives.get(*prim_handle).unwrap();
                    let prims = prim.primitives(*node_handle, prim.material, self);
                    primitives.extend(prims);
                }
            }

            // Collect cameras
            if node.camera.valid() {
                self.camera_nodes.push(*node_handle);
            }

            // Collect light nodes
            if node.light.valid() {
                self.light_nodes.push(*node_handle);
            }
        }

        primitives
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn load() {
        let model = Model::builder()
            .path("tests/model/suzanne/suzanne.gltf")
            .unwrap()
            .build()
            .unwrap();

        assert!(model.images.len() == 2);
    }
}
