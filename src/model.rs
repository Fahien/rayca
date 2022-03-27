// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::{
    error::Error,
    path::{Path, PathBuf},
};

use gltf::Gltf;
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

pub struct ModelBuilder {
    uri_buffers: Vec<Vec<u8>>,
    parent_dir: PathBuf,
    gltf: Gltf,
}

impl ModelBuilder {
    /// Creates a model loading a GLTF file
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let ret = Self {
            uri_buffers: vec![],
            parent_dir: path
                .as_ref()
                .parent()
                .ok_or("Failed to get parent directory")?
                .into(),
            gltf: Gltf::open(path)?,
        };
        Ok(ret)
    }

    pub fn load_images(&mut self, images: &mut Pack<Image>) {
        let mut timer = Timer::new();

        let mut vec: Vec<Image> = self
            .gltf
            .images()
            .enumerate()
            .par_bridge()
            .map(|(id, image)| {
                match image.source() {
                    gltf::image::Source::View { .. } => todo!("Implement image source view"),
                    gltf::image::Source::Uri { uri, .. } => {
                        // Join gltf parent dir to URI
                        let path = self.parent_dir.join(uri);
                        let mut i = Image::load_png(&path);
                        i.id = id;
                        i
                    }
                }
            })
            .collect();

        vec.sort_by_key(|image| image.id);

        println!(
            "Loaded images from file ({}s)",
            timer.get_delta().as_secs_f32()
        );

        *images = Pack::from(vec);
    }

    pub fn load_textures(&mut self, textures: &mut Pack<Texture>) {
        let vec: Vec<Texture> = self
            .gltf
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
        for buffer in self.gltf.buffers() {
            match buffer.source() {
                gltf::buffer::Source::Uri(uri) => {
                    let uri = self.parent_dir.join(uri);
                    let data = std::fs::read(uri)?;
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
        self.load_nodes(&mut model);

        Ok(model)
    }

    pub fn load_materials(&mut self, materials: &mut Pack<Material>) -> Result<(), Box<dyn Error>> {
        for gmaterial in self.gltf.materials() {
            let mut material = Material::new();

            let pbr = gmaterial.pbr_metallic_roughness();

            // Load base color
            let gcolor = pbr.base_color_factor();
            let color = Color::new(gcolor[0], gcolor[1], gcolor[2], gcolor[3]);
            material.color = color;

            // Load albedo
            if let Some(gtexture) = pbr.base_color_texture() {
                material.albedo = Handle::new(gtexture.texture().index());
            }

            materials.push(material);
        }

        Ok(())
    }

    fn load_meshes(&self, model: &mut Model) -> Result<(), Box<dyn Error>> {
        for gmesh in self.gltf.meshes() {
            let mut primitive_handles = vec![];

            for gprimitive in gmesh.primitives() {
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
                        gltf::mesh::Semantic::Positions => {
                            self.load_positions(&mut vertices, &accessor)?
                        }
                        gltf::mesh::Semantic::TexCoords(_) => {
                            self.load_uvs(&mut vertices, &accessor)?
                        }
                        gltf::mesh::Semantic::Colors(_) => {
                            self.load_colors(&mut vertices, &accessor)?
                        }
                        _ => println!("Semantic not implemented {:?}", semantic),
                    }
                }

                let mut indices = vec![];
                let mut index_size = 1;
                if let Some(accessor) = gprimitive.indices() {
                    let data_type = accessor.data_type();
                    index_size = data_type_as_size(data_type);

                    // Data type can vary
                    let data = self.get_data_start(&accessor);
                    let d = &data[0];
                    let length = accessor.count() * data_type_as_size(data_type);
                    // Use bytes regardless of the index data type
                    let slice: &[u8] =
                        unsafe { std::slice::from_raw_parts(d as *const u8 as _, length) };
                    indices = Vec::from(slice);
                }

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
                let primitive_handle = model.primitives.push(primitive);
                primitive_handles.push(primitive_handle);
            }

            let mesh = Mesh::new(primitive_handles);
            model.meshes.push(mesh);
        }

        Ok(())
    }

    fn load_positions(
        &self,
        vertices: &mut Vec<Vertex>,
        accessor: &gltf::Accessor,
    ) -> Result<(), Box<dyn Error>> {
        let data_type = accessor.data_type();
        assert!(data_type == gltf::accessor::DataType::F32);
        let count = accessor.count();
        let dimensions = accessor.dimensions();
        assert!(dimensions == gltf::accessor::Dimensions::Vec3);

        let view = accessor.view().unwrap();

        let target = view.target().unwrap_or(gltf::buffer::Target::ArrayBuffer);
        assert!(target == gltf::buffer::Target::ArrayBuffer);

        let data = self.get_data_start(accessor);
        let stride = get_stride(accessor);

        for i in 0..count {
            let offset = i * stride;
            assert!(offset < data.len());
            let d = &data[offset];
            let position = unsafe { std::slice::from_raw_parts::<f32>(d as *const u8 as _, 3) };

            if vertices.len() <= i {
                vertices.push(Vertex::default())
            }
            vertices[i].pos.x = position[0];
            vertices[i].pos.y = position[1];
            vertices[i].pos.z = position[2];
        }

        Ok(())
    }

    fn load_uvs(
        &self,
        vertices: &mut Vec<Vertex>,
        accessor: &gltf::Accessor,
    ) -> Result<(), Box<dyn Error>> {
        let data_type = accessor.data_type();
        assert!(data_type == gltf::accessor::DataType::F32);
        let count = accessor.count();
        let dimensions = accessor.dimensions();
        assert!(dimensions == gltf::accessor::Dimensions::Vec2);

        let view = accessor.view().unwrap();

        let target = view.target().unwrap_or(gltf::buffer::Target::ArrayBuffer);
        assert!(target == gltf::buffer::Target::ArrayBuffer);

        let data = self.get_data_start(accessor);
        let stride = get_stride(accessor);

        for i in 0..count {
            let offset = i * stride;
            assert!(offset < data.len());
            let d = &data[offset];
            let uv = unsafe { std::slice::from_raw_parts::<f32>(d as *const u8 as _, 2) };

            if vertices.len() <= i {
                vertices.push(Vertex::default())
            }
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
        let data_type = accessor.data_type();
        assert!(data_type == gltf::accessor::DataType::F32);
        let count = accessor.count();
        let dimensions = accessor.dimensions();
        assert!(dimensions == gltf::accessor::Dimensions::Vec3);

        let view = accessor.view().unwrap();
        let target = view.target().unwrap_or(gltf::buffer::Target::ArrayBuffer);
        assert!(target == gltf::buffer::Target::ArrayBuffer);

        let data = self.get_data_start(accessor);
        let stride = get_stride(accessor);

        for i in 0..count {
            let offset = i * stride;
            assert!(offset < data.len());
            let d = &data[offset];
            let normal = unsafe { std::slice::from_raw_parts::<f32>(d as *const u8 as _, 3) };

            if vertices.len() <= i {
                vertices.push(Vertex::default())
            }
            vertices[i].normal.x = normal[0];
            vertices[i].normal.y = normal[1];
            vertices[i].normal.z = normal[2];
        }

        Ok(())
    }

    fn load_colors(
        &self,
        vertices: &mut Vec<Vertex>,
        accessor: &gltf::Accessor,
    ) -> Result<(), Box<dyn Error>> {
        let data_type = accessor.data_type();
        assert!(data_type == gltf::accessor::DataType::F32);
        let count = accessor.count();
        let dimensions = accessor.dimensions();
        assert!(dimensions == gltf::accessor::Dimensions::Vec3);
        let len = match dimensions {
            gltf::accessor::Dimensions::Vec3 => 3,
            gltf::accessor::Dimensions::Vec4 => 4,
            _ => panic!("Invalid color dimensions"),
        };

        let view = accessor.view().unwrap();
        let target = view.target().unwrap_or(gltf::buffer::Target::ArrayBuffer);
        assert!(target == gltf::buffer::Target::ArrayBuffer);

        let data = self.get_data_start(accessor);
        let stride = get_stride(accessor);

        for i in 0..count {
            let offset = i * stride;
            assert!(offset < data.len());
            let d = &data[offset];
            let color = unsafe { std::slice::from_raw_parts::<f32>(d as *const u8 as _, len) };

            if vertices.len() <= i {
                vertices.push(Vertex::default())
            }
            vertices[i].color.r = color[0];
            vertices[i].color.g = color[1];
            vertices[i].color.b = color[2];
            vertices[i].color.a = if len == 4 { color[3] } else { 1.0 };
        }

        Ok(())
    }

    fn load_nodes(&self, model: &mut Model) {
        // Load scene
        let scene = self.gltf.scenes().next().unwrap();
        model.root = Node::builder()
            .name("Root".into())
            .children(
                scene
                    .nodes()
                    .map(|gchild| Handle::new(gchild.index()))
                    .collect(),
            )
            .build();

        // Load nodes
        for gnode in self.gltf.nodes() {
            let mut node_builder = Node::builder()
                .id(gnode.index())
                .name(gnode.name().unwrap_or("Unknown").into())
                .children(
                    gnode
                        .children()
                        .map(|gchild| Handle::new(gchild.index()))
                        .collect(),
                );

            let transform = gnode.transform().decomposed();

            let translation = &transform.0;
            let translation = Vec3::new(translation[0], translation[1], translation[2]);
            node_builder = node_builder.translation(translation);

            // xyzw
            let rotation = &transform.1;
            let rotation = Quat::new(rotation[0], rotation[1], rotation[2], rotation[3]);
            node_builder = node_builder.rotation(rotation);

            let scale = &transform.2;
            let scale = Vec3::new(scale[0], scale[1], scale[2]);
            node_builder = node_builder.scale(scale);

            if let Some(mesh) = gnode.mesh() {
                node_builder = node_builder.mesh(Handle::new(mesh.index()));
            }

            let node = node_builder.build();
            model.nodes.push(node);
        }
    }
}

#[derive(Default)]
pub struct Model {
    pub id: usize,
    pub samplers: Pack<Sampler>,
    pub images: Pack<Image>,
    pub textures: Pack<Texture>,
    pub materials: Pack<Material>,
    pub primitives: Pack<Primitive>,
    pub meshes: Pack<Mesh>,
    pub nodes: Pack<Node>,
    pub root: Node,
}

impl Model {
    pub fn builder<P: AsRef<Path>>(path: P) -> Result<ModelBuilder, Box<dyn Error>> {
        ModelBuilder::new(path)
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
            material.albedo.offset(texture_offset);
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

        let mesh_offset = self.meshes.append(&mut model.meshes);
        let node_offset = self.nodes.len();
        // Update mesh and node handles
        for node in model.nodes.iter_mut() {
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
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn load() {
        let model = Model::builder("tests/model/suzanne/suzanne.gltf")
            .unwrap()
            .build()
            .unwrap();

        assert!(model.images.len() == 2);
    }
}
