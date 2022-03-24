// Copyright © 2022-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

pub mod mesh;
pub mod vertex;
pub use mesh::*;
pub use vertex::*;

use std::{error::Error, path::Path};

use gltf::Gltf;

use crate::{Color, GgxMaterial, Handle, Node, Pack, Quat, Vec3};

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

    fn load_positions(
        &self,
        vertices: &mut Vec<GltfVertex>,
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
                vertices.push(GltfVertex::default())
            }
            vertices[i].pos.x = position[0];
            vertices[i].pos.y = position[1];
            vertices[i].pos.z = position[2];
        }

        Ok(())
    }

    fn load_normals(
        &self,
        vertices: &mut Vec<GltfVertex>,
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
                vertices.push(GltfVertex::default())
            }
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
                vertices.push(GltfVertex::default())
            }
            vertices[i].color.r = color[0];
            vertices[i].color.g = color[1];
            vertices[i].color.b = color[2];
            vertices[i].color.a = if len == 4 { color[3] } else { 1.0 };
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
        let uri_buffers = UriBuffers::new(&gltf, parent_dir)?;
        ret.load_materials(&gltf)?;
        ret.load_meshes(&gltf, &uri_buffers)?;
        ret.load_nodes(&gltf);

        Ok(ret)
    }

    pub fn load_materials(&mut self, gltf: &Gltf) -> Result<(), Box<dyn Error>> {
        for gmaterial in gltf.materials() {
            let mut material = GgxMaterial::new();

            let pbr = gmaterial.pbr_metallic_roughness();

            // Load base color
            let gcolor = pbr.base_color_factor();
            let color = Color::new(gcolor[0], gcolor[1], gcolor[2], gcolor[3]);
            material.color = color;

            self.materials.push(material);
        }

        Ok(())
    }

    fn load_meshes(&mut self, gltf: &Gltf, uri_buffers: &UriBuffers) -> Result<(), Box<dyn Error>> {
        for gmesh in gltf.meshes() {
            let mut primitive_handles = vec![];

            for gprimitive in gmesh.primitives() {
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
                        gltf::mesh::Semantic::Colors(_) => {
                            uri_buffers.load_colors(&mut vertices, &accessor)?
                        }
                        _ => println!("Semantic not implemented {:?}", semantic),
                    }
                }

                let (indices, index_size) = uri_buffers.load_indices(&gprimitive);

                let material = gprimitive
                    .material()
                    .index()
                    .map_or(Handle::none(), Handle::new);

                let primitive = GltfPrimitive::builder()
                    .vertices(vertices)
                    .indices(indices)
                    .index_size(index_size)
                    .material(material)
                    .build();
                let primitive_handle = self.primitives.push(primitive);
                primitive_handles.push(primitive_handle);
            }

            let mesh = GltfMesh::new(primitive_handles);
            self.meshes.push(mesh);
        }

        Ok(())
    }

    fn load_nodes(&mut self, gltf: &Gltf) {
        // Load scene
        let scene = gltf.scenes().next().unwrap();
        self.root = Node::builder()
            .name("Root".into())
            .children(
                scene
                    .nodes()
                    .map(|gchild| Handle::new(gchild.index()))
                    .collect(),
            )
            .build();

        // Load nodes
        for gnode in gltf.nodes() {
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
            self.nodes.push(node);
        }
    }
}

#[cfg(test)]
mod test {
    use owo_colors::{OwoColorize, Stream::Stdout};

    use super::*;

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
}
