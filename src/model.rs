// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::{
    error::Error,
    path::{Path, PathBuf},
};

use gltf::Gltf;

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

        self.load_uri_buffers()?;
        self.load_meshes(&mut model)?;

        Ok(model)
    }

    fn load_meshes(&self, model: &mut Model) -> Result<(), Box<dyn Error>> {
        for gmesh in self.gltf.meshes() {
            let mut primitive_handles = vec![];

            for gprimitive in gmesh.primitives() {
                let mut vertices = vec![];

                let mode = gprimitive.mode();
                assert!(mode == gltf::mesh::Mode::Triangles);

                for (semantic, accessor) in gprimitive.attributes() {
                    match semantic {
                        gltf::mesh::Semantic::Positions => {
                            self.load_positions(&mut vertices, &accessor)?
                        }
                        _ => println!("Semantic not implemented {:?}", semantic),
                    }
                }

                let mut indices = vec![];
                if let Some(accessor) = gprimitive.indices() {
                    let data_type = accessor.data_type();

                    // Data type can vary
                    let data = self.get_data_start(&accessor);
                    let d = &data[0];
                    let length = accessor.count() * data_type_as_size(data_type);
                    // Use bytes regardless of the index data type
                    let slice: &[u8] =
                        unsafe { std::slice::from_raw_parts(d as *const u8 as _, length) };
                    indices = Vec::from(slice);
                }

                let primitive = Primitive::builder()
                    .vertices(vertices)
                    .indices(indices)
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
}

#[derive(Default)]
pub struct Model {
    pub id: usize,
    pub primitives: Pack<Primitive>,
    pub meshes: Pack<Mesh>,
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
}
