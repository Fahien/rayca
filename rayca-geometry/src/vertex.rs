// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use rayca_math::*;

pub struct VertexExtBuilder {
    color: Color,
    normal: Vec3,
    tangent: Vec3,
    bitangent: Vec3,
    uv: Vec2,
}

impl Default for VertexExtBuilder {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            normal: Vec3::Z_AXIS,
            tangent: Vec3::default(),
            bitangent: Vec3::default(),
            uv: Vec2::default(),
        }
    }
}

impl VertexExtBuilder {
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn normal(mut self, normal: Vec3) -> Self {
        self.normal = normal;
        self
    }

    pub fn tangent(mut self, tangent: Vec3) -> Self {
        self.tangent = tangent;
        self
    }

    pub fn bitangent(mut self, bitangent: Vec3) -> Self {
        self.bitangent = bitangent;
        self
    }

    pub fn uv(mut self, uv: Vec2) -> Self {
        self.uv = uv;
        self
    }

    pub fn build(self) -> VertexExt {
        VertexExt {
            color: self.color,
            normal: self.normal,
            tangent: self.tangent,
            bitangent: self.bitangent,
            uv: self.uv,
        }
    }
}

#[repr(C, align(16))]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct VertexExt {
    pub color: Color,
    pub normal: Vec3,
    pub tangent: Vec3,
    pub bitangent: Vec3,
    pub uv: Vec2,
}

impl VertexExt {
    pub fn builder() -> VertexExtBuilder {
        VertexExtBuilder::default()
    }

    pub fn new(color: Color, uv: Vec2, normal: Vec3, tangent: Vec3, bitangent: Vec3) -> Self {
        Self {
            color,
            uv,
            normal,
            tangent,
            bitangent,
        }
    }
}

impl Default for VertexExt {
    fn default() -> Self {
        Self::new(
            Color::WHITE,
            Vec2::default(),
            Vec3::Z_AXIS,
            Vec3::default(),
            Vec3::default(),
        )
    }
}

#[derive(Default)]
pub struct VertexBuilder {
    pos: Point3,
    ext_builder: VertexExtBuilder,
}

impl VertexBuilder {
    pub fn position(mut self, pos: Point3) -> Self {
        self.pos = pos;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.ext_builder = self.ext_builder.color(color);
        self
    }

    pub fn normal(mut self, normal: Vec3) -> Self {
        self.ext_builder = self.ext_builder.normal(normal);
        self
    }

    pub fn uv(mut self, uv: Vec2) -> Self {
        self.ext_builder = self.ext_builder.uv(uv);
        self
    }

    pub fn build(self) -> Vertex {
        Vertex {
            pos: self.pos,
            ext: self.ext_builder.build(),
        }
    }
}

#[repr(C, align(16))]
#[derive(Debug, Clone, PartialEq)]
pub struct Vertex {
    pub pos: Point3,
    pub ext: VertexExt,
}

impl Vertex {
    pub fn builder() -> VertexBuilder {
        VertexBuilder::default()
    }

    pub fn new(
        pos: Point3,
        color: Color,
        uv: Vec2,
        normal: Vec3,
        tangent: Vec3,
        bitangent: Vec3,
    ) -> Self {
        Self {
            pos,
            ext: VertexExt::new(color, uv, normal, tangent, bitangent),
        }
    }
}

impl Default for Vertex {
    fn default() -> Self {
        Self::new(
            Point3::default(),
            Color::WHITE,
            Vec2::default(),
            Vec3::Z_AXIS,
            Vec3::default(),
            Vec3::default(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_ext_builder_defaults() {
        let ext = VertexExtBuilder::default().build();
        assert_eq!(ext.color, Color::WHITE);
        assert_eq!(ext.normal, Vec3::Z_AXIS);
        assert_eq!(ext.tangent, Vec3::default());
        assert_eq!(ext.bitangent, Vec3::default());
        assert_eq!(ext.uv, Vec2::default());
    }

    #[test]
    fn test_vertex_ext_builder_custom() {
        let color = Color::from_rgb(0.1, 0.2, 0.3);
        let normal = Vec3::new(1.0, 0.0, 0.0);
        let ext = VertexExtBuilder::default()
            .color(color)
            .normal(normal)
            .build();
        assert_eq!(ext.color, color);
        assert_eq!(ext.normal, normal);
    }

    #[test]
    fn test_vertex_builder_defaults() {
        let vertex = VertexBuilder::default().build();
        assert_eq!(vertex.pos, Point3::default());
        assert_eq!(vertex.ext, VertexExt::default());
    }

    #[test]
    fn test_vertex_new() {
        let pos = Point3::new(1.0, 2.0, 3.0);
        let color = Color::from_rgb(0.5, 0.5, 0.5);
        let uv = Vec2::new(0.1, 0.2);
        let normal = Vec3::new(0.0, 1.0, 0.0);
        let tangent = Vec3::new(1.0, 0.0, 0.0);
        let bitangent = Vec3::new(0.0, 0.0, 1.0);
        let vertex = Vertex::new(pos, color, uv, normal, tangent, bitangent);
        assert_eq!(vertex.pos, pos);
        assert_eq!(vertex.ext.color, color);
        assert_eq!(vertex.ext.uv, uv);
        assert_eq!(vertex.ext.normal, normal);
        assert_eq!(vertex.ext.tangent, tangent);
        assert_eq!(vertex.ext.bitangent, bitangent);
    }
}
