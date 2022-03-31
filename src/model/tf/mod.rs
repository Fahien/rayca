// Copyright © 2022-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

pub mod mesh;
pub mod vertex;
pub use mesh::*;
pub use vertex::*;

use std::{error::Error, path::Path};

use gltf::Gltf;
use owo_colors::OwoColorize;
use rayon::iter::{ParallelBridge, ParallelIterator};

use crate::{Color, GgxMaterial, Handle, Image, Node, Pack, Quat, Sampler, Texture, Timer, Vec3};

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

struct UriBuffers {
    data: Vec<Vec<u8>>,
}

impl UriBuffers {
    fn new(gltf: &Gltf, parent_dir: &Path) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            data: Self::load_uri_buffers(gltf, parent_dir)?,
        })
    }

    fn load_uri_buffers(gltf: &Gltf, parent_dir: &Path) -> Result<Vec<Vec<u8>>, Box<dyn Error>> {
        let mut uri_buffers = vec![];
        for buffer in gltf.buffers() {
            match buffer.source() {
                gltf::buffer::Source::Uri(uri) => {
                    let uri = parent_dir.join(uri);
                    let data = std::fs::read(uri)?;
                    assert!(buffer.index() == uri_buffers.len());
                    uri_buffers.push(data);
                }
                _ => unimplemented!(),
            }
        }

        Ok(uri_buffers)
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
        vertices: &mut Vec<GltfVertex>,
        accessor: &gltf::Accessor,
    ) -> Result<(), Box<dyn Error>> {
        let positions = self.get_slices(accessor);
        vertices.resize(positions.len(), GltfVertex::default());
        for (i, position) in positions.into_iter().enumerate() {
            vertices[i].pos.x = position[0];
            vertices[i].pos.y = position[1];
            vertices[i].pos.z = position[2];
        }
        Ok(())
    }

    fn load_uvs(
        &self,
        vertices: &mut Vec<GltfVertex>,
        accessor: &gltf::Accessor,
    ) -> Result<(), Box<dyn Error>> {
        let uvs = self.get_slices(accessor);
        vertices.resize(uvs.len(), GltfVertex::default());
        for (i, uv) in uvs.into_iter().enumerate() {
            vertices[i].uv.x = uv[0];
            vertices[i].uv.y = uv[1];
        }
        Ok(())
    }

    fn load_normals(
        &self,
        vertices: &mut Vec<GltfVertex>,
        accessor: &gltf::Accessor,
    ) -> Result<(), Box<dyn Error>> {
        let normals = self.get_slices(accessor);
        vertices.resize(normals.len(), GltfVertex::default());
        for (i, normal) in normals.into_iter().enumerate() {
            vertices[i].normal.x = normal[0];
            vertices[i].normal.y = normal[1];
            vertices[i].normal.z = normal[2];
        }
        Ok(())
    }

    fn load_colors(
        &self,
        vertices: &mut Vec<GltfVertex>,
        accessor: &gltf::Accessor,
    ) -> Result<(), Box<dyn Error>> {
        let colors = self.get_slices(accessor);
        vertices.resize(colors.len(), GltfVertex::default());
        for (i, color) in colors.into_iter().enumerate() {
            vertices[i].color.r = color[0];
            vertices[i].color.g = color[1];
            vertices[i].color.b = color[2];
            vertices[i].color.a = if color.len() == 4 { color[3] } else { 1.0 };
        }
        Ok(())
    }

    fn get_data_start<'b>(&'b self, accessor: &gltf::Accessor) -> &'b [u8] {
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

        let data = &self.data[buffer.index()];
        &data[offset..end_offset]
    }
}

#[derive(Default)]
pub struct GltfModel {
    pub textures: Pack<Texture>,
    pub samplers: Pack<Sampler>,
    pub images: Pack<Image>,
    pub materials: Pack<GgxMaterial>,
    pub primitives: Pack<GltfPrimitive>,
    pub meshes: Pack<GltfMesh>,
    pub nodes: Pack<Node>,
    pub root: Node,
}

impl GltfModel {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<GltfModel, Box<dyn Error>> {
        let mut ret = Self::default();

        let gltf = Gltf::open(path.as_ref())?;
        let parent_dir = path
            .as_ref()
            .parent()
            .ok_or("Failed to get parent directory")?;
        ret.load_images(&gltf, parent_dir);
        ret.load_textures(&gltf);
        ret.load_materials(&gltf)?;
        let uri_buffers = UriBuffers::new(&gltf, parent_dir)?;
        ret.load_meshes(&gltf, &uri_buffers)?;
        ret.load_nodes(&gltf);

        Ok(ret)
    }

    pub fn load_images(&mut self, gltf: &Gltf, parent_dir: &Path) {
        let mut timer = Timer::new();

        let mut vec: Vec<Image> = gltf
            .images()
            .enumerate()
            .par_bridge()
            .map(|(id, image)| {
                match image.source() {
                    gltf::image::Source::View { .. } => todo!("Implement image source view"),
                    gltf::image::Source::Uri { uri, .. } => {
                        // Join gltf parent dir to URI
                        let path = parent_dir.join(uri);
                        let mut i = Image::load_png(&path);
                        i.id = id;
                        i
                    }
                }
            })
            .collect();

        vec.sort_by_key(|image| image.id);

        println!(
            "{:>12} images from file in {:.2}s",
            "Loaded".green().bold(),
            timer.get_delta().as_secs_f32()
        );

        self.images = Pack::from(vec);
    }

    pub fn load_textures(&mut self, gltf: &Gltf) {
        let vec: Vec<Texture> = gltf
            .textures()
            .map(|gtexture| {
                let image = Handle::new(gtexture.source().index());
                let sampler = Handle::none();
                Texture::new(image, sampler)
            })
            .collect();
        self.textures = Pack::from(vec);
    }

    pub fn load_materials(&mut self, gltf: &Gltf) -> Result<(), Box<dyn Error>> {
        for gmaterial in gltf.materials() {
            let mut material = GgxMaterial::new();

            let pbr = gmaterial.pbr_metallic_roughness();

            // Load base color
            let gcolor = pbr.base_color_factor();
            let color = Color::new(gcolor[0], gcolor[1], gcolor[2], gcolor[3]);
            material.color = color;

            // Load albedo
            if let Some(gtexture) = pbr.base_color_texture() {
                material.albedo = Handle::new(gtexture.texture().index());
            }

            self.materials.push(material);
        }

        Ok(())
    }

    fn load_vertices(
        &self,
        uri_buffers: &UriBuffers,
        gprimitive: &gltf::Primitive,
    ) -> Result<Vec<GltfVertex>, Box<dyn Error>> {
        let mut vertices = vec![];

        let mode = gprimitive.mode();
        assert!(mode == gltf::mesh::Mode::Triangles);

        // Load normals first, so we can process tangents later
        for (semantic, accessor) in gprimitive.attributes() {
            if semantic == gltf::mesh::Semantic::Normals {
                uri_buffers.load_normals(&mut vertices, &accessor)?;
            }
        }

        for (semantic, accessor) in gprimitive.attributes() {
            match semantic {
                gltf::mesh::Semantic::Positions => {
                    uri_buffers.load_positions(&mut vertices, &accessor)?
                }
                gltf::mesh::Semantic::TexCoords(_) => {
                    uri_buffers.load_uvs(&mut vertices, &accessor)?
                }
                gltf::mesh::Semantic::Colors(_) => {
                    uri_buffers.load_colors(&mut vertices, &accessor)?
                }
                _ => println!("Semantic not implemented {:?}", semantic),
            }
        }

        Ok(vertices)
    }

    fn load_primitive(
        &mut self,
        uri_buffers: &UriBuffers,
        gprimitive: &gltf::Primitive,
    ) -> Result<Handle<GltfPrimitive>, Box<dyn Error>> {
        let vertices = self.load_vertices(uri_buffers, gprimitive)?;
        let (indices, index_size) = uri_buffers.load_indices(gprimitive);

        let material = if let Some(index) = gprimitive.material().index() {
            Handle::new(index)
        } else {
            Handle::none()
        };

        let primitive = GltfPrimitive::builder()
            .vertices(vertices)
            .indices(indices)
            .index_size(index_size)
            .material(material)
            .build();

        Ok(self.primitives.push(primitive))
    }

    fn load_meshes(&mut self, gltf: &Gltf, uri_buffers: &UriBuffers) -> Result<(), Box<dyn Error>> {
        for gmesh in gltf.meshes() {
            let primitive_handles = gmesh
                .primitives()
                .into_iter()
                .map(|gprimitive| {
                    self.load_primitive(uri_buffers, &gprimitive)
                        .expect("Failed to load a primitive")
                })
                .collect();

            let mesh = GltfMesh::new(primitive_handles);
            self.meshes.push(mesh);
        }
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

        node_builder.build()
    }

    fn load_nodes(&mut self, gltf: &Gltf) {
        // Load scene
        let scene = gltf.scenes().next().unwrap();
        self.root = Self::create_root(&scene);

        // Load nodes
        for gnode in gltf.nodes() {
            let node = Self::create_node(&gnode);
            self.nodes.push(node);
        }
    }
}

#[cfg(test)]
mod test {
    use owo_colors::{OwoColorize, Stream::Stdout};

    use crate::GltfModel;

    #[test]
    fn load_gltf() {
        assert!(GltfModel::load("test").is_err());

        let path = "tests/model/box/box.gltf";
        if let Err(err) = GltfModel::load(path) {
            panic!(
                "{}: Failed to load \"{}\": {}",
                "ERROR".if_supports_color(Stdout, |text| text.red()),
                path,
                err
            );
        }
    }

    #[test]
    fn load() {
        let model = GltfModel::load("tests/model/suzanne/suzanne.gltf").unwrap();
        assert!(model.images.len() == 2);
    }
}
