// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::{collections::HashMap, error::Error, io::BufWriter, mem::offset_of, path::Path};

use base64::Engine;
use rayca_math::Trs;

use crate::*;

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

fn data_type_to_component_type(data_type: gltf::accessor::DataType) -> ComponentType {
    match data_type {
        gltf::accessor::DataType::I8 => ComponentType::I8,
        gltf::accessor::DataType::U8 => ComponentType::U8,
        gltf::accessor::DataType::I16 => ComponentType::I16,
        gltf::accessor::DataType::U16 => ComponentType::U16,
        gltf::accessor::DataType::U32 => ComponentType::U32,
        gltf::accessor::DataType::F32 => ComponentType::F32,
    }
}

struct UriBuffers {
    data: Vec<Vec<u8>>,
}

impl UriBuffers {
    fn new(
        gltf: &gltf::Gltf,
        parent_dir: Option<&Path>,
        assets: &Assets,
    ) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            data: Self::load_uri_buffers(gltf, parent_dir, assets)?,
        })
    }

    fn load_uri_buffers(
        gltf: &gltf::Gltf,
        parent_dir: Option<&Path>,
        assets: &Assets,
    ) -> Result<Vec<Vec<u8>>, Box<dyn Error>> {
        let mut uri_buffers = vec![];
        for buffer in gltf.buffers() {
            match buffer.source() {
                gltf::buffer::Source::Uri(uri) => {
                    const DATA_URI: &str = "data:application/octet-stream;base64,";

                    let data = if uri.starts_with(DATA_URI) {
                        let (_, data_base64) = uri.split_at(DATA_URI.len());
                        base64::engine::general_purpose::STANDARD.decode(data_base64)?
                    } else if let Some(parent_dir) = &parent_dir {
                        let uri = parent_dir.join(uri);
                        assets.load(uri)?.into_bytes()
                    } else {
                        assets.load(uri)?.into_bytes()
                    };
                    assert_eq!(buffer.index(), uri_buffers.len());
                    uri_buffers.push(data);
                }
                _ => unimplemented!(),
            }
        }

        Ok(uri_buffers)
    }

    fn load_indices(&self, gprimitive: &gltf::Primitive) -> (Vec<u8>, ComponentType) {
        let mut indices = vec![];
        let mut index_type = ComponentType::U8;

        if let Some(accessor) = gprimitive.indices() {
            let data_type = accessor.data_type();
            index_type = data_type_to_component_type(data_type);

            // Data type can vary
            let data = self.get_data_start(&accessor);
            let d = &data[0];
            let len = accessor.count() * data_type_as_size(data_type);
            // Use bytes regardless of the index data type
            let slice: &[u8] = unsafe { std::slice::from_raw_parts(d as *const u8 as _, len) };
            indices = Vec::from(slice);
        }

        (indices, index_type)
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
            vertices[i].ext.uv.x = uv[0];
            vertices[i].ext.uv.y = uv[1];
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
            vertices[i].ext.normal = Vec3::new(normal[0], normal[1], normal[2]);
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
            vertices[i].ext.color.r = color[0];
            vertices[i].ext.color.g = color[1];
            vertices[i].ext.color.b = color[2];
            vertices[i].ext.color.a = if color.len() == 4 { color[3] } else { 1.0 };
        }
        Ok(())
    }

    fn load_tangents(&self, vertices: &mut [Vertex], accessor: &gltf::Accessor) {
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

            vertex.ext.tangent = Vec3::new(tangent[0], tangent[1], tangent[2]);

            // Compute bitangent as for glTF 2.0 spec
            vertex.ext.bitangent = vertex.ext.normal.cross(vertex.ext.tangent) * tangent[3];
        });
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

impl Model {
    fn load_gltf(
        gltf: gltf::Gltf,
        path: Option<&Path>,
        assets: &Assets,
    ) -> Result<Self, Box<dyn Error>> {
        let mut timer = Timer::new();

        let path_str = if let Some(path) = path {
            path.to_string_lossy().to_string()
        } else {
            String::new()
        };

        let parent_dir = if let Some(path) = path {
            Some(path.parent().ok_or("Failed to get parent directory")?)
        } else {
            None
        };

        let mut ret = Self::default();
        ret.load_images(&gltf, parent_dir, assets);
        ret.load_textures(&gltf);
        ret.load_materials(&gltf)?;
        let uri_buffers = UriBuffers::new(&gltf, parent_dir, assets)?;
        ret.load_meshes(&gltf, &uri_buffers)?;
        ret.load_cameras(&gltf);
        ret.load_nodes(&gltf);

        log::info!(
            "Loaded {} in {:.2}ms",
            path_str,
            timer.get_delta().as_millis()
        );
        Ok(ret)
    }

    pub fn load_gltf_path<P: AsRef<Path>>(
        path: P,
        assets: &Assets,
    ) -> Result<Self, Box<dyn Error>> {
        let data = assets.load(path.as_ref())?.into_bytes();
        let gltf = gltf::Gltf::from_slice(&data)?;
        Self::load_gltf(gltf, Some(path.as_ref()), assets)
    }

    pub fn load_gltf_data(data: &[u8], assets: &Assets) -> Result<Self, Box<dyn Error>> {
        let gltf = gltf::Gltf::from_slice(data)?;
        Self::load_gltf(gltf, None, assets)
    }

    pub fn load_images(&mut self, gltf: &gltf::Gltf, parent_dir: Option<&Path>, assets: &Assets) {
        let mut timer = Timer::new();

        use rayon::prelude::{ParallelBridge, ParallelIterator};
        let images_iter = gltf.images().enumerate().par_bridge();

        let image_map: HashMap<usize, Image> = images_iter
            .map(|(id, image)| {
                match image.source() {
                    gltf::image::Source::View { .. } => todo!("Implement image source view"),
                    gltf::image::Source::Uri { uri, .. } => {
                        const DATA_URI: &str = "data:image/png;base64,";

                        let image = if uri.starts_with(DATA_URI) {
                            let (_, data_base64) = uri.split_at(DATA_URI.len());
                            let data = base64::engine::general_purpose::STANDARD
                                .decode(data_base64)
                                .expect("Failed to decode base64 image data");
                            Image::load_data(&data)
                        } else if let Some(parent_dir) = parent_dir {
                            // Join gltf parent dir to URI
                            let path = parent_dir.join(uri);
                            Image::load_file(path, assets).expect("Failed to load image file")
                        } else {
                            Image::load_file(uri, assets).expect("Failed to load image file")
                        };

                        (id, image)
                    }
                }
            })
            .collect();

        // The images at this point are not sorted by ID, so we need to sort them
        let mut pairs: Vec<(usize, Image)> = image_map.into_iter().collect();
        pairs.sort_by_key(|(id, _)| *id);
        let sorted_images: Vec<Image> = pairs.into_iter().map(|(_, image)| image).collect();

        log::info!(
            "Loaded images from file in {:.2}s",
            timer.get_delta().as_secs_f32()
        );

        self.images = Pack::from(sorted_images);
    }

    pub fn load_textures(&mut self, gltf: &gltf::Gltf) {
        let vec: Vec<Texture> = gltf
            .textures()
            .map(|gtexture| {
                let image = Handle::from(gtexture.source().index());
                let sampler = Handle::NONE;
                Texture::new(image, sampler)
            })
            .collect();
        self.textures = Pack::from(vec);
    }

    pub fn load_materials(&mut self, gltf: &gltf::Gltf) -> Result<(), Box<dyn Error>> {
        for gmaterial in gltf.materials() {
            // If this is a Phong material, its definition is in extras
            // if let Some(extras) = gmaterial.extras() {
            //     if let Ok(material) = serde_json::from_str::<PhongMaterial>(extras.get()) {
            //         self.phong_materials.push(material);
            //         continue;
            //     }
            // }

            // Default behaviour is parsing a Pbr material
            let mut material = PbrMaterial::builder();

            let pbr = gmaterial.pbr_metallic_roughness();

            // Load base color
            let gcolor = pbr.base_color_factor();
            let color = Color::new(gcolor[0], gcolor[1], gcolor[2], gcolor[3]);
            material = material.color(color);

            // Load albedo
            if let Some(gtexture) = pbr.base_color_texture() {
                material = material.albedo(Handle::from(gtexture.texture().index()));
            }

            // Load normal
            if let Some(gtexture) = gmaterial.normal_texture() {
                material = material.normal(Handle::from(gtexture.texture().index()));
            }

            // Load metallic roughness factors and texture
            material = material.metallic_factor(pbr.metallic_factor());
            material = material.roughness_factor(pbr.roughness_factor());
            if let Some(gtexture) = pbr.metallic_roughness_texture() {
                material = material.metallic_roughness(Handle::from(gtexture.texture().index()));
            }

            let pbr_material_handle = self.pbr_materials.push(material.build());
            self.materials.push(Material::Pbr(pbr_material_handle));
        }

        Ok(())
    }

    fn load_vertices(
        &self,
        uri_buffers: &UriBuffers,
        gprimitive: &gltf::Primitive,
    ) -> Result<Vec<Vertex>, Box<dyn Error>> {
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
                gltf::mesh::Semantic::Normals => (), // Already loaded
                gltf::mesh::Semantic::Tangents => {
                    uri_buffers.load_tangents(&mut vertices, &accessor)
                }
                _ => println!("Skipping semantic: {:?}", semantic),
            }
        }

        Ok(vertices)
    }

    fn load_primitive(
        &mut self,
        uri_buffers: &UriBuffers,
        gprimitive: &gltf::Primitive,
    ) -> Result<Handle<Primitive>, Box<dyn Error>> {
        let vertices = self.load_vertices(uri_buffers, gprimitive)?;
        let (indices, index_size) = uri_buffers.load_indices(gprimitive);

        // All glTF primitives are triangles, so we can create a geometry with a triangle mesh
        let triangle_mesh = TriangleMesh::builder()
            .vertices(vertices)
            .indices(
                TriangleIndices::builder()
                    .indices(indices)
                    .index_type(index_size.into())
                    .build(),
            )
            .build();

        // Triangle mesh are wrapped in a geometry
        let geometry = Geometry::TriangleMesh(triangle_mesh);
        let geometry_handle = self.geometries.push(geometry);

        let material = if let Some(index) = gprimitive.material().index() {
            index as usize
        } else {
            usize::MAX
        };

        let primitive = Primitive::builder()
            .geometry(geometry_handle)
            .material(material.into())
            .build();

        Ok(self.primitives.push(primitive))
    }

    fn load_meshes(
        &mut self,
        gltf: &gltf::Gltf,
        uri_buffers: &UriBuffers,
    ) -> Result<(), Box<dyn Error>> {
        for gmesh in gltf.meshes() {
            let primitive_handles = gmesh
                .primitives()
                .map(|gprimitive| {
                    self.load_primitive(uri_buffers, &gprimitive)
                        .expect("Failed to load a primitive")
                })
                .collect();

            let mesh = Mesh::builder().primitives(primitive_handles).build();
            self.meshes.push(mesh);
        }
        Ok(())
    }

    fn load_cameras(&mut self, gltf: &gltf::Gltf) {
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
            self.cameras.push(camera);
        }
    }

    fn create_node(gnode: &gltf::Node) -> Node {
        let transform = gnode.transform().decomposed();

        let translation = &transform.0;
        let translation = Vec3::new(translation[0], translation[1], translation[2]);

        let rotation = &transform.1;
        let rotation = Quat::new(rotation[0], rotation[1], rotation[2], rotation[3]);

        let scale = &transform.2;
        let scale = Vec3::new(scale[0], scale[1], scale[2]);

        let mesh = gnode.mesh().map(|m| Handle::from(m.index()));
        let camera = gnode.camera().map(|c| Handle::from(c.index()));

        Node::builder()
            .name(gnode.name().unwrap_or("Unknown").to_string())
            .children(
                gnode
                    .children()
                    .map(|gchild| Handle::from(gchild.index()))
                    .collect(),
            )
            .trs(
                Trs::builder()
                    .translation(translation)
                    .rotation(rotation)
                    .scale(scale)
                    .build(),
            )
            .maybe_mesh(mesh)
            .maybe_camera(camera)
            .build()
    }

    fn load_nodes(&mut self, gltf: &gltf::Gltf) {
        // Load scene
        let gscene = gltf.scenes().next().unwrap();
        self.root.children = gscene
            .nodes()
            .into_iter()
            .map(|n| n.index().into())
            .collect();

        // Load nodes
        for gnode in gltf.nodes() {
            let node = Self::create_node(&gnode);
            self.nodes.push(node);
        }
    }

    /// Returns the last node with a camera
    pub fn get_mut_last_camera(&mut self) -> Option<&mut Node> {
        self.nodes
            .iter_mut()
            .rev()
            .find(|node| node.camera.is_some())
    }

    /// Stores the model as a glTF file in the current working directory.
    /// Two files will be named after the model's name with a `.gltf` extension
    /// and a `.bin` extension.
    pub fn store_gltf_file<P: AsRef<Path>>(&self, dir: P) -> std::fmt::Result {
        let store_model = StoreModel::new(self);

        let bin_path = dir.as_ref().join(&store_model.buffer.uri);
        std::fs::write(bin_path, &store_model.buffer.data)
            .expect("Failed to write binary buffer file");

        let mut json_string = String::default();
        use std::fmt::Write;
        write!(
            &mut json_string,
            "{{ \"asset\": {{ \"version\": \"2.0\" }},"
        )?;
        write!(&mut json_string, "{store_model}")?;

        // Images
        write!(&mut json_string, ", \"images\": [")?;
        for (i, image) in self.images.iter().enumerate() {
            if i > 0 {
                write!(&mut json_string, ",")?;
            }
            write!(&mut json_string, "{image}")?;
        }
        write!(&mut json_string, "]")?;

        // Textures
        write!(&mut json_string, ", \"textures\": [")?;
        for (i, texture) in self.textures.iter().enumerate() {
            if i > 0 {
                write!(&mut json_string, ",")?;
            }
            write!(&mut json_string, "{texture}")?;
        }
        write!(&mut json_string, "]")?;

        // Materials
        write!(&mut json_string, ", \"materials\": [")?;
        for (i, material) in self.pbr_materials.iter().enumerate() {
            if i > 0 {
                write!(&mut json_string, ",")?;
            }
            write!(&mut json_string, "{material}")?;
        }
        write!(&mut json_string, "]")?;

        write!(&mut json_string, ", \"nodes\": [")?;
        for (i, node) in self.nodes.iter().enumerate() {
            if i > 0 {
                write!(&mut json_string, ",")?;
            }
            write!(&mut json_string, "{node}")?;
        }

        write!(&mut json_string, "]")?;
        write!(&mut json_string, ", \"scenes\": [ {{ \"nodes\": [")?;
        for (i, child) in self.root.children.iter().enumerate() {
            if i > 0 {
                write!(&mut json_string, ",")?;
            }
            write!(&mut json_string, "{}", child.id)?;
        }
        write!(&mut json_string, "] }} ]")?;

        write!(&mut json_string, "}}")?;

        let file_path = dir.as_ref().join(self.get_uri());
        std::fs::write(file_path, json_string).expect("Failed to write glTF file");
        Ok(())
    }
}

#[derive(Default)]
struct StorePrimitive {
    attributes: HashMap<gltf::mesh::Semantic, Handle<Accessor>>,
    indices: Handle<Accessor>,
    material: Handle<Material>,
}

fn semantic_to_string(semantic: &gltf::mesh::Semantic) -> &'static str {
    match semantic {
        gltf::mesh::Semantic::Positions => "POSITION",
        gltf::mesh::Semantic::Normals => "NORMAL",
        gltf::mesh::Semantic::Tangents => "TANGENT",
        gltf::mesh::Semantic::TexCoords(_) => "TEXCOORD_0",
        gltf::mesh::Semantic::Colors(_) => "COLOR_0",
        _ => "UNKNOWN",
    }
}

impl std::fmt::Display for StorePrimitive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ \"attributes\": {{")?;
        for (i, (semantic, accessor)) in self.attributes.iter().enumerate() {
            if i > 0 {
                write!(f, ",")?;
            }
            write!(f, "\"{}\": {}", semantic_to_string(semantic), accessor.id)?;
        }
        write!(f, "}}, \"indices\": {}", self.indices.id)?;
        if self.material.is_valid() {
            write!(f, ", \"material\": {}", self.material.id)?;
        }
        write!(f, "}}")
    }
}

#[derive(Default)]
pub struct StoreModel {
    /// The main buffer will contain vertices and indices, this is going to be saved as `.bin`
    pub buffer: Buffer,
    pub buffer_views: Pack<BufferView>,
    accessors: Pack<Accessor>,
    primitives: Pack<StorePrimitive>,
    meshes: Pack<Mesh>,
}

impl StoreModel {
    fn new(model: &Model) -> Self {
        let mut store_model = Self::default();
        store_model.buffer.uri = format!("{}.bin", model.name);
        store_model.meshes = model.meshes.clone();

        // Generate primitives for infomation to file, while appending vertex data to the main buffer
        for primitive in model.primitives.iter() {
            let mut store_primitive = StorePrimitive::default();
            store_primitive.material = primitive.material;

            let geometry = model.geometries.get(primitive.geometry).unwrap();
            let triangle_mesh = geometry.as_triangle_mesh().unwrap();

            // Buffer views should be created for vertices and indices
            let buffer_view = store_model.buffer.extend_from_bytes(
                &triangle_mesh.vertices,
                std::mem::size_of::<Vertex>(),
                BufferViewTarget::ArrayBuffer,
            );
            let vertex_buffer_handle = store_model.buffer_views.push(buffer_view);

            // Accessors are generated here
            let vertex_count = triangle_mesh.vertices.len();
            let mut position_accessor = Accessor::builder()
                .buffer_view(vertex_buffer_handle)
                .offset(0)
                .component_type(ComponentType::F32)
                .count(vertex_count)
                .accessor_type(AccessorType::Vec3)
                .build();

            let positions: Vec<&AccessorVec3<f32>> = position_accessor.as_slice(&store_model);
            use buffer::AccessorOrd;
            let min_pos = positions
                .iter()
                .fold(AccessorVec3::<f32>::max_value(), |acc, &v| acc.min(v));
            let max_pos = positions
                .iter()
                .fold(AccessorVec3::<f32>::min_value(), |acc, &v| acc.max(v));
            position_accessor.min.replace(min_pos.to_string());
            position_accessor.max.replace(max_pos.to_string());

            let position_accessor_handle = store_model.accessors.push(position_accessor);
            store_primitive
                .attributes
                .insert(gltf::mesh::Semantic::Positions, position_accessor_handle);

            let color_accessor = Accessor::new(
                vertex_buffer_handle,
                offset_of!(Vertex, ext.color) as usize,
                ComponentType::F32,
                vertex_count,
                AccessorType::Vec4,
            );
            let color_accessor_handle = store_model.accessors.push(color_accessor);
            store_primitive
                .attributes
                .insert(gltf::mesh::Semantic::Colors(0), color_accessor_handle);

            let normal_accessor = Accessor::new(
                vertex_buffer_handle,
                offset_of!(Vertex, ext.normal) as usize,
                ComponentType::F32,
                vertex_count,
                AccessorType::Vec3,
            );
            let normal_accessor_handle = store_model.accessors.push(normal_accessor);
            store_primitive
                .attributes
                .insert(gltf::mesh::Semantic::Normals, normal_accessor_handle);

            let uv_accessor = Accessor::new(
                vertex_buffer_handle,
                offset_of!(Vertex, ext.uv) as usize,
                ComponentType::F32,
                vertex_count,
                AccessorType::Vec2,
            );
            let uv_accessor_handle = store_model.accessors.push(uv_accessor);
            store_primitive
                .attributes
                .insert(gltf::mesh::Semantic::TexCoords(0), uv_accessor_handle);

            let buffer_view = store_model.buffer.extend_from_bytes(
                triangle_mesh.indices.get_indices::<u8>(),
                0,
                BufferViewTarget::ElementArrayBuffer,
            );
            let index_buffer_handle = store_model.buffer_views.push(buffer_view);

            let index_accessor = Accessor::new(
                index_buffer_handle,
                0,
                triangle_mesh.indices.index_type.into(),
                triangle_mesh.indices.get_index_count(),
                AccessorType::Scalar,
            );
            let index_accessor_handle = store_model.accessors.push(index_accessor);
            store_primitive.indices = index_accessor_handle;

            store_model.primitives.push(store_primitive);
        }

        store_model
    }
}

impl std::fmt::Display for StoreModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"buffers\": [")?;
        write!(
            f,
            "{{ \"byteLength\": {}, \"uri\": \"{}\" }}",
            self.buffer.data.len(),
            self.buffer.uri
        )?;
        write!(f, "],")?;

        write!(f, "\"bufferViews\": [")?;
        for (i, buffer_view) in self.buffer_views.iter().enumerate() {
            if i > 0 {
                write!(f, ",")?;
            }
            write!(f, "{buffer_view}",)?;
        }
        write!(f, "],")?;

        write!(f, "\"accessors\": [")?;
        for (i, accessor) in self.accessors.iter().enumerate() {
            if i > 0 {
                write!(f, ",")?;
            }
            write!(f, "{accessor}",)?;
        }
        write!(f, "],")?;

        write!(f, "\"meshes\": [")?;

        for (i, mesh) in self.meshes.iter().enumerate() {
            if i > 0 {
                write!(f, ",")?;
            }
            write!(f, "{{")?;
            write!(f, "\"primitives\": [")?;
            for (j, primitive) in mesh.primitives.iter().enumerate() {
                if j > 0 {
                    write!(f, ",")?;
                }
                let store_primitive = self.primitives.get(primitive.id.into()).unwrap();
                write!(f, "{store_primitive}")?;
            }
            write!(f, "]")?;
            write!(f, "}}")?;
        }

        write!(f, "]")
    }
}

impl From<gltf::buffer::Target> for BufferViewTarget {
    fn from(value: gltf::buffer::Target) -> Self {
        match value {
            gltf::buffer::Target::ArrayBuffer => BufferViewTarget::ArrayBuffer,
            gltf::buffer::Target::ElementArrayBuffer => BufferViewTarget::ElementArrayBuffer,
        }
    }
}

impl From<gltf::buffer::View<'_>> for BufferView {
    fn from(value: gltf::buffer::View) -> Self {
        let target = value
            .target()
            .map_or(BufferViewTarget::None, BufferViewTarget::from);

        Self::new(
            value.buffer().index().into(),
            value.offset(),
            value.length(),
            value.stride().unwrap_or_default(),
            target,
        )
    }
}

impl StoreScene {
    pub fn load_glx_file<P: AsRef<Path>>(file_path: P) -> std::io::Result<Self> {
        let uri = file_path.as_ref();
        let file = std::fs::File::open(uri)?;
        let reader = std::io::BufReader::new(file);
        let scene = serde_json::from_reader(reader)?;
        Ok(scene)
    }

    pub fn store_glx_file<P: AsRef<Path>>(&self, file_path: P) -> std::io::Result<()> {
        let glx_file = std::fs::File::create(file_path)?;
        let stream = BufWriter::new(glx_file);
        serde_json::to_writer_pretty(stream, self)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    fn load_gltf() {
        let assets = Assets::new();
        assert!(Model::load_gltf_path("test", &assets).is_err());

        let path = tests::get_model_path().join("box/box.gltf");
        if let Err(err) = Model::load_gltf_path(&path, &assets) {
            panic!("Failed to load `{}`: {}", path.display(), err);
        }
    }

    #[test]
    fn load_images() {
        let assets = Assets::new();
        let model_path = tests::get_model_path().join("suzanne/suzanne.gltf");
        let model = Model::load_gltf_path(model_path, &assets).unwrap();
        assert_eq!(model.images.len(), 2);
        assert!(!model.root.children.is_empty());
    }

    #[test]
    fn load_phong() {
        let assets = Assets::new();
        let model_path = tests::get_model_path().join("box/box-phong.gltf");
        let model = Model::load_gltf_path(model_path, &assets).unwrap();
        assert!(model.materials.len() == 1);
    }

    #[test]
    fn store_scene() {
        let mut scene = StoreScene::new("TestScene");
        let hmodel = scene.models.push(ModelSource::new("TestModel"));
        scene
            .nodes
            .push(Node::builder().model(hmodel.id.into()).build());

        let glx_path = tests::get_artifact_path().join("test-scene.glx");
        scene
            .store_glx_file(&glx_path)
            .expect("Failed to store scene");
        let loaded_scene = StoreScene::load_glx_file(glx_path).expect("Failed to load scene");
        assert_eq!(scene.name, loaded_scene.name);
        assert_eq!(scene.models[0].uri, loaded_scene.models[0].uri);
    }
}
