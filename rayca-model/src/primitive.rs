// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(u32)]
pub enum PrimitiveMode {
    Points = 0,
    Lines = 1,
    LineLoop = 2,
    LineStrip = 3,
    #[default]
    Triangles = 4,
    TriangleStrip = 5,
    TriangleFan = 6,
}

#[derive(Clone, Default)]

pub struct PrimitiveIndicesBuilder {
    indices: Vec<u8>,
    pub index_type: ComponentType,
}

impl PrimitiveIndicesBuilder {
    pub fn indices(mut self, indices: Vec<u8>) -> Self {
        self.indices = indices;
        self
    }

    pub fn index_type(mut self, index_type: ComponentType) -> Self {
        self.index_type = index_type;
        self
    }

    pub fn build(self) -> PrimitiveIndices {
        PrimitiveIndices {
            indices: self.indices,
            index_type: self.index_type,
        }
    }
}

#[derive(Debug, Clone, Default)]

pub struct PrimitiveIndices {
    pub indices: Vec<u8>,
    pub index_type: ComponentType,
}

impl PrimitiveIndices {
    pub fn builder() -> PrimitiveIndicesBuilder {
        PrimitiveIndicesBuilder::default()
    }

    pub fn get_index_count(&self) -> usize {
        self.indices.len() / self.index_type.get_size()
    }
}

#[derive(Clone)]
pub struct PrimitiveBuilder {
    mode: PrimitiveMode,

    vertices: Vec<Vertex>,
    indices: Option<PrimitiveIndices>,

    material: Handle<Material>,
}

impl Default for PrimitiveBuilder {
    fn default() -> Self {
        Self {
            mode: Default::default(),
            vertices: Default::default(),
            indices: None,
            material: Default::default(),
        }
    }
}

impl PrimitiveBuilder {
    pub fn mode(mut self, mode: PrimitiveMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn vertices(mut self, vertices: Vec<Vertex>) -> Self {
        self.vertices = vertices;
        self
    }

    pub fn indices(mut self, indices: PrimitiveIndices) -> Self {
        self.indices.replace(indices);
        self
    }

    pub fn quad(self, uv_scale: Vec2) -> Self {
        self.vertices(vec![
            Vertex::builder()
                .position(Point3::new(-0.5, -0.5, 0.0))
                .color(Color::WHITE)
                .normal(Vec3::Z_AXIS)
                .uv(Vec2::new(0.0, 1.0) * uv_scale)
                .build(),
            Vertex::builder()
                .position(Point3::new(0.5, -0.5, 0.0))
                .color(Color::WHITE)
                .normal(Vec3::Z_AXIS)
                .uv(Vec2::new(1.0, 1.0) * uv_scale)
                .build(),
            Vertex::builder()
                .position(Point3::new(0.5, 0.5, 0.0))
                .color(Color::WHITE)
                .normal(Vec3::Z_AXIS)
                .uv(Vec2::new(1.0, 0.0) * uv_scale)
                .build(),
            Vertex::builder()
                .position(Point3::new(-0.5, 0.5, 0.0))
                .color(Color::WHITE)
                .normal(Vec3::Z_AXIS)
                .uv(Vec2::new(0.0, 0.0) * uv_scale)
                .build(),
        ])
        .indices(
            PrimitiveIndices::builder()
                .indices(vec![0, 1, 2, 2, 3, 0])
                .build(),
        )
    }

    pub fn cube(self) -> Self {
        self.vertices(vec![
            // Front
            Vertex::builder()
                .position(Point3::new(-0.5, -0.5, 0.5))
                .color(Color::WHITE)
                .normal(Vec3::Z_AXIS)
                .uv(Vec2::new(0.0, 0.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(0.5, -0.5, 0.5))
                .color(Color::WHITE)
                .normal(Vec3::Z_AXIS)
                .uv(Vec2::new(1.0, 0.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(0.5, 0.5, 0.5))
                .color(Color::WHITE)
                .normal(Vec3::Z_AXIS)
                .uv(Vec2::new(1.0, 1.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(-0.5, 0.5, 0.5))
                .color(Color::WHITE)
                .normal(Vec3::Z_AXIS)
                .uv(Vec2::new(0.0, 1.0))
                .build(),
            // Right
            Vertex::builder()
                .position(Point3::new(0.5, -0.5, 0.5))
                .color(Color::WHITE)
                .normal(Vec3::X_AXIS)
                .uv(Vec2::new(0.0, 0.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(0.5, -0.5, -0.5))
                .color(Color::WHITE)
                .normal(Vec3::X_AXIS)
                .uv(Vec2::new(1.0, 0.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(0.5, 0.5, -0.5))
                .color(Color::WHITE)
                .normal(Vec3::X_AXIS)
                .uv(Vec2::new(1.0, 1.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(0.5, 0.5, 0.5))
                .color(Color::WHITE)
                .normal(Vec3::X_AXIS)
                .uv(Vec2::new(0.0, 1.0))
                .build(),
            // Back
            Vertex::builder()
                .position(Point3::new(0.5, -0.5, -0.5))
                .color(Color::WHITE)
                .normal(-Vec3::Z_AXIS)
                .uv(Vec2::new(0.0, 0.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(-0.5, -0.5, -0.5))
                .color(Color::WHITE)
                .normal(-Vec3::Z_AXIS)
                .uv(Vec2::new(1.0, 0.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(-0.5, 0.5, -0.5))
                .color(Color::WHITE)
                .normal(-Vec3::Z_AXIS)
                .uv(Vec2::new(1.0, 1.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(0.5, 0.5, -0.5))
                .color(Color::WHITE)
                .normal(-Vec3::Z_AXIS)
                .uv(Vec2::new(0.0, 1.0))
                .build(),
            // Left
            Vertex::builder()
                .position(Point3::new(-0.5, -0.5, -0.5))
                .color(Color::WHITE)
                .normal(-Vec3::X_AXIS)
                .uv(Vec2::new(0.0, 0.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(-0.5, -0.5, 0.5))
                .color(Color::WHITE)
                .normal(-Vec3::X_AXIS)
                .uv(Vec2::new(1.0, 0.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(-0.5, 0.5, 0.5))
                .color(Color::WHITE)
                .normal(-Vec3::X_AXIS)
                .uv(Vec2::new(1.0, 1.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(-0.5, 0.5, -0.5))
                .color(Color::WHITE)
                .normal(-Vec3::X_AXIS)
                .uv(Vec2::new(0.0, 1.0))
                .build(),
            // Top
            Vertex::builder()
                .position(Point3::new(-0.5, 0.5, 0.5))
                .color(Color::WHITE)
                .normal(Vec3::Y_AXIS)
                .uv(Vec2::new(0.0, 0.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(0.5, 0.5, 0.5))
                .color(Color::WHITE)
                .normal(Vec3::Y_AXIS)
                .uv(Vec2::new(1.0, 0.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(0.5, 0.5, -0.5))
                .color(Color::WHITE)
                .normal(Vec3::Y_AXIS)
                .uv(Vec2::new(1.0, 1.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(-0.5, 0.5, -0.5))
                .color(Color::WHITE)
                .normal(Vec3::Y_AXIS)
                .uv(Vec2::new(0.0, 1.0))
                .build(),
            // Bottom
            Vertex::builder()
                .position(Point3::new(-0.5, -0.5, -0.5))
                .color(Color::WHITE)
                .normal(-Vec3::Y_AXIS)
                .uv(Vec2::new(0.0, 0.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(0.5, -0.5, -0.5))
                .color(Color::WHITE)
                .normal(-Vec3::Y_AXIS)
                .uv(Vec2::new(1.0, 0.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(0.5, -0.5, 0.5))
                .color(Color::WHITE)
                .normal(-Vec3::Y_AXIS)
                .uv(Vec2::new(1.0, 1.0))
                .build(),
            Vertex::builder()
                .position(Point3::new(-0.5, -0.5, 0.5))
                .color(Color::WHITE)
                .normal(-Vec3::Y_AXIS)
                .uv(Vec2::new(0.0, 1.0))
                .build(),
        ])
        .indices(
            PrimitiveIndices::builder()
                .indices(vec![
                    0, 1, 2, 0, 2, 3, // front face
                    4, 5, 6, 4, 6, 7, // right
                    8, 9, 10, 8, 10, 11, // back
                    12, 13, 14, 12, 14, 15, // left
                    16, 17, 18, 16, 18, 19, // top
                    20, 21, 22, 20, 22, 23, // bottom
                ])
                .index_type(ComponentType::U8)
                .build(),
        )
    }

    pub fn material(mut self, material: Handle<Material>) -> Self {
        self.material = material;
        self
    }

    pub fn build(self) -> Primitive {
        Primitive {
            mode: self.mode,
            vertices: self.vertices,
            indices: self.indices,
            material: self.material,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Primitive {
    pub mode: PrimitiveMode,

    pub vertices: Vec<Vertex>,
    pub indices: Option<PrimitiveIndices>,

    pub material: Handle<Material>,
}

impl Default for Primitive {
    fn default() -> Self {
        Primitive::builder().build()
    }
}

impl Primitive {
    pub fn builder() -> PrimitiveBuilder {
        PrimitiveBuilder::default()
    }
}
