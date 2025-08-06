// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use bon::Builder;
use num_traits::NumCast;

use crate::*;

pub struct TriangleBuilder {
    vertices: [Point3; 3],
}

impl Default for TriangleBuilder {
    fn default() -> Self {
        Self {
            vertices: [
                Point3::new(-1.0, 0.0, 0.0),
                Point3::new(1.0, 0.0, 0.0),
                Point3::new(0.0, 1.0, 0.0),
            ],
        }
    }
}

impl TriangleBuilder {
    pub fn a(mut self, a: Point3) -> Self {
        self.vertices[0] = a;
        self
    }

    pub fn b(mut self, b: Point3) -> Self {
        self.vertices[1] = b;
        self
    }

    pub fn c(mut self, c: Point3) -> Self {
        self.vertices[2] = c;
        self
    }

    pub fn build(self) -> Triangle {
        Triangle::new(self.vertices)
    }
}

#[repr(C, align(16))]
pub struct Triangle {
    vertices: [Point3; 3],
    centroid: Vec3,
}

impl Triangle {
    pub fn builder() -> TriangleBuilder {
        TriangleBuilder::default()
    }

    pub fn new(vertices: [Point3; 3]) -> Self {
        let centroid =
            (Vec3::from(vertices[0]) + Vec3::from(vertices[1]) + Vec3::from(vertices[2])) * 0.3333;
        Self { vertices, centroid }
    }

    /// Returns the i-th vertex in transformed from model space to world space
    /// applying the model transform `trs`.
    pub fn get_vertex(&self, i: usize, trs: &Trs) -> Point3 {
        trs * self.vertices[i]
    }
}

impl Default for Triangle {
    fn default() -> Self {
        Self::new([
            Point3::new(-1.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        ])
    }
}

impl Triangle {
    /// [Ray-triangle intersection](https://www.scratchapixel.com/lessons/3d-basic-rendering/ray-tracing-rendering-a-triangle/ray-triangle-intersection-geometric-solution)
    pub fn intersects(&self, trs: &Trs, ray: &Ray) -> Option<Hit> {
        let v0 = Vec3::from(self.get_vertex(0, trs));
        let v1 = Vec3::from(self.get_vertex(1, trs));
        let v2 = Vec3::from(self.get_vertex(2, trs));

        // Plane's normal
        let v0v1 = v1 - v0;
        let v0v2 = v2 - v0;
        // No need to normalize
        let n = v0v1.cross(v0v2);

        // Back-face test
        if ray.dir.dot(n) > 0.0 {
            return None;
        }

        let denom = n.dot(n);

        // Step 1: finding P

        // Check if ray and plane are parallel
        let n_dot_ray_dir = n.dot(ray.dir);
        if n_dot_ray_dir.abs() < f32::EPSILON {
            // Parallel do not intersect
            return None;
        }
        // Compute d parameter using equation 2
        let d = -n.dot(&v0);

        // Compute t (equation 3)
        let t = -(n.dot(Vec3::from(ray.origin)) + d) / n_dot_ray_dir;

        // Check if the triangle is behind the ray
        if t < 0.0 {
            return None;
        }

        // Compute the intersection point using equation 1
        let p = ray.origin + ray.dir * t;

        // Step 2: inside-outside test

        // Edge 0
        let edge0 = v1 - v0;
        let vp0 = Vec3::from(p - v0);
        // Vector perpendicular to triangle's plane
        let c = edge0.cross(vp0);
        if n.dot(c) < 0.0 {
            return None; // P is on the right side
        }

        // Edge 1
        let edge1 = v2 - v1;
        let vp1 = Vec3::from(p - v1);
        let c = edge1.cross(vp1);
        let u = n.dot(c);
        if u < 0.0 {
            return None; // P is on the right side
        }

        // Edge 2
        let edge2 = v0 - v2;
        let vp2 = Vec3::from(p - v2);
        let c = edge2.cross(vp2);
        let v = n.dot(c);
        if v < 0.0 {
            return None; // P is on the right side;
        }

        let uv = Vec2::new(u / denom, v / denom);

        // A triangle does not know its primitive index
        // Setting the primitive index is responsiblity of the caller
        let hit = Hit::new(ray.clone(), u32::MAX, u32::MAX, t, p, uv);
        Some(hit) // This ray hits the triangle
    }

    pub fn get_centroid(&self, trs: &Trs) -> Vec3 {
        trs * self.centroid
    }

    pub fn min(&self, trs: &Trs) -> Point3 {
        Point3::new(f32::MAX, f32::MAX, f32::MAX)
            .min(self.get_vertex(0, trs))
            .min(self.get_vertex(1, trs))
            .min(self.get_vertex(2, trs))
    }

    pub fn max(&self, trs: &Trs) -> Point3 {
        Point3::new(f32::MIN, f32::MIN, f32::MIN)
            .max(self.get_vertex(0, trs))
            .max(self.get_vertex(1, trs))
            .max(self.get_vertex(2, trs))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(u32)]
pub enum ComponentType {
    /// Byte
    I8 = 5120,

    #[default]
    /// Unsigned byte
    U8 = 5121,

    /// Short
    I16 = 5122,

    /// Unsigned short
    U16 = 5123,

    /// Unsigned int
    U32 = 5125,

    /// Float
    F32 = 5126,
}

impl ComponentType {
    pub fn get_size(self) -> usize {
        match self {
            ComponentType::I8 | ComponentType::U8 => 1,
            ComponentType::I16 | ComponentType::U16 => 2,
            ComponentType::U32 | ComponentType::F32 => 4,
        }
    }
}

#[derive(Builder, Debug, Clone, Default)]

pub struct TriangleIndices {
    indices: Vec<u8>,
    #[builder(default)]
    pub index_type: ComponentType,
}

impl TriangleIndices {
    pub fn get_index_count(&self) -> usize {
        self.indices.len() / self.index_type.get_size()
    }

    pub fn push(&mut self, index_bytes: &[u8]) {
        if index_bytes.len() != self.index_type.get_size() {
            panic!("Index size does not match the component type size");
        }
        self.indices.extend_from_slice(index_bytes);
    }

    pub fn expand_index_size(&mut self) {
        match self.index_type {
            ComponentType::U8 => {
                self.index_type = ComponentType::U16;
                let expanded_indices: Vec<u16> =
                    self.indices.clone().into_iter().map(|i| i as u16).collect();
                self.indices = Vec::from(unsafe {
                    std::slice::from_raw_parts(
                        expanded_indices.as_ptr() as *const u8,
                        expanded_indices.len() * 2,
                    )
                });
            }
            ComponentType::U16 => {
                self.index_type = ComponentType::U32;

                let indices16 = unsafe {
                    std::slice::from_raw_parts(
                        self.indices.as_ptr() as *const u16,
                        self.indices.len() / 2,
                    )
                };

                let expanded_indices: Vec<u32> = indices16.iter().map(|i| *i as u32).collect();
                self.indices = Vec::from(unsafe {
                    std::slice::from_raw_parts(
                        expanded_indices.as_ptr() as *const u8,
                        expanded_indices.len() * 4,
                    )
                });
            }
            _ => (),
        }
    }

    pub fn add_index(&mut self, last_index: usize) {
        // Check whether we need more bits for indices
        match self.index_type {
            ComponentType::U8 if last_index == std::u8::MAX as usize + 1 => {
                self.expand_index_size()
            }
            ComponentType::U16 if last_index == std::u16::MAX as usize + 1 => {
                self.expand_index_size()
            }
            _ if last_index == std::usize::MAX => {
                panic!("Yeah, you know, I can't really handle all these vertices..")
            }
            _ => (), // you good
        }

        match self.index_type {
            ComponentType::U8 => {
                self.indices.push(last_index as u8);
            }
            ComponentType::U16 => {
                let last_index: [u8; 2] = (last_index as u16).to_ne_bytes();
                self.indices.extend(last_index);
            }
            _ => {
                let last_index: [u8; 4] = (last_index as u32).to_ne_bytes();
                self.indices.extend(last_index);
            }
        }
    }

    pub fn set_indices(&mut self, indices: Vec<u8>) {
        self.indices = indices;
    }

    pub fn get_indices<Index: NumCast>(&self) -> &[Index] {
        let index_count = self.get_index_count();
        assert_eq!(std::mem::size_of::<Index>(), self.index_type.get_size());
        unsafe { std::slice::from_raw_parts(self.indices.as_ptr() as *const Index, index_count) }
    }
}

/// Collection of triangles defined by vertices and indices.
#[derive(Builder, Debug, Clone)]
pub struct TriangleMesh {
    pub vertices: Vec<Vertex>,
    pub indices: TriangleIndices,
}

impl Default for TriangleMesh {
    fn default() -> Self {
        Self::new(vec![], TriangleIndices::default())
    }
}

impl TriangleMesh {
    pub fn new(vertices: Vec<Vertex>, indices: TriangleIndices) -> Self {
        Self { vertices, indices }
    }

    pub fn unit() -> Self {
        Self::builder()
            .vertices(vec![
                Vertex::builder()
                    .position(Point3::new(-1.0, 0.0, 0.0))
                    .build(),
                Vertex::builder()
                    .position(Point3::new(1.0, 0.0, 0.0))
                    .build(),
                Vertex::builder()
                    .position(Point3::new(0.0, 1.0, 0.0))
                    .build(),
            ])
            .indices(TriangleIndices::builder().indices(vec![0, 1, 2]).build())
            .build()
    }

    pub fn quad(uv_scale: Vec2) -> Self {
        Self::builder()
            .vertices(vec![
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
                TriangleIndices::builder()
                    .indices(vec![0, 1, 2, 2, 3, 0])
                    .build(),
            )
            .build()
    }

    pub fn cube(self) -> Self {
        Self::builder()
            .vertices(vec![
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
                TriangleIndices::builder()
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
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intersect() {
        let triangle = Triangle::default();
        let ray = Ray::new(Point3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, -1.0));
        assert!(triangle.intersects(&Trs::IDENTITY, &ray).is_some());
        let ray = Ray::new(Point3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, 1.0));
        assert!(triangle.intersects(&Trs::IDENTITY, &ray).is_none());
    }

    #[test]
    fn centroid_and_min_max() {
        let triangle = Triangle::default();
        let trs = Trs::IDENTITY;
        let centroid = triangle.get_centroid(&trs);
        let expected_centroid = (Vec3::from(Point3::new(-1.0, 0.0, 0.0))
            + Vec3::from(Point3::new(1.0, 0.0, 0.0))
            + Vec3::from(Point3::new(0.0, 1.0, 0.0)))
            * 0.3333;
        assert!((centroid - expected_centroid).len() < 1e-3);

        let min = triangle.min(&trs);
        let max = triangle.max(&trs);
        assert_eq!(min, Point3::new(-1.0, 0.0, 0.0));
        assert_eq!(max, Point3::new(1.0, 1.0, 0.0));
    }

    #[test]
    #[should_panic]
    fn get_vertex_out_of_bounds() {
        let triangle = Triangle::default();
        let _ = triangle.get_vertex(3, &Trs::IDENTITY);
    }

    #[test]
    fn expand() {
        let mut indices = TriangleIndices::builder()
            .indices(vec![0, 1, 2])
            .index_type(ComponentType::U8)
            .build();
        assert_eq!(indices.indices, vec![0, 1, 2]);
        assert_eq!(indices.index_type, ComponentType::U8);

        indices.expand_index_size();
        assert_eq!(indices.index_type, ComponentType::U16);

        let indices_len = indices.get_index_count();
        let indices = unsafe {
            std::slice::from_raw_parts(indices.indices.as_ptr() as *const u16, indices_len)
        };
        assert_eq!(indices, vec![0, 1, 2]);
    }
}
