// Copyright © 2022-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

pub mod mesh;
pub mod vertex;
pub use mesh::*;
pub use vertex::*;

use std::{error::Error, path::Path};

use gltf::Gltf;

use crate::Pack;

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
    pub primitives: Pack<GltfPrimitive>,
    pub meshes: Pack<GltfMesh>,
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
        ret.load_meshes(&gltf, &uri_buffers)?;

        Ok(ret)
    }

    fn load_meshes(&mut self, gltf: &Gltf, uri_buffers: &UriBuffers) -> Result<(), Box<dyn Error>> {
        for gmesh in gltf.meshes() {
            let mut primitive_handles = vec![];

            for gprimitive in gmesh.primitives() {
                let mut vertices = vec![];

                let mode = gprimitive.mode();
                assert!(mode == gltf::mesh::Mode::Triangles);

                for (semantic, accessor) in gprimitive.attributes() {
                    match semantic {
                        gltf::mesh::Semantic::Positions => {
                            uri_buffers.load_positions(&mut vertices, &accessor)?
                        }
                        _ => println!("Semantic not implemented {:?}", semantic),
                    }
                }

                let mut indices = vec![];
                if let Some(accessor) = gprimitive.indices() {
                    let data_type = accessor.data_type();

                    // Data type can vary
                    let data = uri_buffers.get_data_start(&accessor);
                    let d = &data[0];
                    let length = accessor.count() * data_type_as_size(data_type);
                    // Use bytes regardless of the index data type
                    let slice: &[u8] =
                        unsafe { std::slice::from_raw_parts(d as *const u8 as _, length) };
                    indices = Vec::from(slice);
                }

                let primitive = GltfPrimitive::builder()
                    .vertices(vertices)
                    .indices(indices)
                    .build();
                let primitive_handle = self.primitives.push(primitive);
                primitive_handles.push(primitive_handle);
            }

            let mesh = GltfMesh::new(primitive_handles);
            self.meshes.push(mesh);
        }

        Ok(())
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
