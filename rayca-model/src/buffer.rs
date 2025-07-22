// Copyright © 2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use bon::Builder;

use crate::{gltf_loader::StoreModel, *};

#[derive(Default, Clone)]
pub struct Buffer {
    pub uri: String,
    pub data: Vec<u8>,
}

impl Buffer {
    pub fn new(uri: String, data: Vec<u8>) -> Self {
        Self { uri, data }
    }

    pub fn extend_from_bytes<B: AsBytes>(
        &mut self,
        bytes: &B,
        stride: usize,
        target: BufferViewTarget,
    ) -> BufferView {
        let offset = self.data.len();
        self.data.extend_from_slice(bytes.as_bytes());
        // Always align up to 4 bytes
        let padding = 4 - (self.data.len() % 4);
        if padding < 4 {
            self.data.extend(vec![0; padding]);
        }
        let size = self.data.len() - offset;

        BufferView::new(
            0.into(), // Handle<Buffer> is not used here, so we use a dummy value
            offset,
            size,
            stride,
            target,
        )
    }
}

impl std::ops::Deref for Buffer {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl std::ops::DerefMut for Buffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

#[derive(Default, Clone)]
pub struct BufferView {
    pub buffer: Handle<Buffer>,
    pub offset: usize,
    pub size: usize,
    pub stride: usize,
    pub target: BufferViewTarget,
}

impl BufferView {
    pub fn new(
        buffer: Handle<Buffer>,
        offset: usize,
        size: usize,
        stride: usize,
        target: BufferViewTarget,
    ) -> Self {
        Self {
            buffer,
            offset,
            size,
            stride,
            target,
        }
    }
}

impl std::fmt::Display for BufferView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{ \"buffer\": {}, \"byteOffset\": {}, \"byteLength\": {}, \"target\": {}",
            self.buffer.id, self.offset, self.size, self.target as u32
        )?;
        if self.stride > 0 {
            write!(f, ", \"byteStride\": {}", self.stride)?;
        }
        write!(f, " }}")
    }
}

#[repr(u32)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum BufferViewTarget {
    #[default]
    None = 0,
    ArrayBuffer = 34962,
    ElementArrayBuffer = 34963,
}

impl From<u32> for BufferViewTarget {
    fn from(value: u32) -> Self {
        match value {
            34962 => BufferViewTarget::ArrayBuffer,
            34963 => BufferViewTarget::ElementArrayBuffer,
            _ => BufferViewTarget::None,
        }
    }
}

#[derive(Default, Builder)]
pub struct Accessor {
    pub buffer_view: Handle<BufferView>,

    /// The offset relative to the start of the buffer view in bytes.
    pub offset: usize,

    /// The datatype of the accessor’s components.
    pub component_type: ComponentType,

    /// The number of elements referenced by this accessor.
    pub count: usize,

    /// Specifies if the accessor’s elements are scalars, vectors, or matrices.
    pub accessor_type: AccessorType,

    /// Optional minimum value for the accessor.
    pub min: Option<String>,

    /// Optional maximum value for the accessor.
    pub max: Option<String>,
}

impl Accessor {
    pub fn new(
        buffer_view: Handle<BufferView>,
        offset: usize,
        component_type: ComponentType,
        count: usize,
        accessor_type: AccessorType,
    ) -> Self {
        Self {
            buffer_view,
            offset,
            component_type,
            count,
            accessor_type,
            min: None,
            max: None,
        }
    }

    pub fn get_bytes<'a>(&self, model: &'a StoreModel) -> &'a [u8] {
        let view = model.buffer_views.get(self.buffer_view).unwrap();

        let offset = view.offset + self.offset;
        assert!(offset + view.size <= model.buffer.len());

        &model.buffer.data[offset..offset + view.size]
    }

    pub fn get_stride(&self, model: &StoreModel) -> usize {
        let view = model.buffer_views.get(self.buffer_view).unwrap();
        if view.stride > 0 {
            view.stride
        } else {
            self.component_type.get_size() * self.accessor_type.get_dimension_count()
        }
    }

    pub fn as_slice<T>(&self, model: &StoreModel) -> Vec<&T> {
        let data = self.get_bytes(model);
        let stride = self.get_stride(model);
        let dimension_count = self.accessor_type.get_dimension_count();
        let component_size = self.component_type.get_size();

        let t_size = std::mem::size_of::<T>();
        assert_eq!(dimension_count * component_size, t_size);

        let mut ret = vec![];

        for i in 0..self.count {
            let offset = i * stride;
            assert!(offset < data.len());
            let d = &data[offset..offset + t_size];
            let elem: &T = unsafe { std::mem::transmute(d.as_ptr()) };
            ret.push(elem);
        }

        ret
    }
}

impl std::fmt::Display for Accessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{ \"bufferView\": {}, \"byteOffset\": {}, \"componentType\": {}, \"count\": {}, \"type\": \"{}\"",
            self.buffer_view.id,
            self.offset,
            self.component_type as u32,
            self.count,
            self.accessor_type,
        )?;

        if let Some(min_value) = &self.min {
            write!(f, ", \"min\": {}", min_value)?;
        }
        if let Some(max_value) = &self.max {
            write!(f, ", \"max\": {}", max_value)?;
        }
        write!(f, " }}")?;
        Ok(())
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum AccessorType {
    #[default]
    Scalar,
    Vec2,
    Vec3,
    Vec4,
    Mat2,
    Mat3,
    Mat4,
}

impl AccessorType {
    pub fn get_dimension_count(self) -> usize {
        match self {
            AccessorType::Scalar => 1,
            AccessorType::Vec2 => 2,
            AccessorType::Vec3 => 3,
            AccessorType::Vec4 => 4,
            AccessorType::Mat2 => 4,
            AccessorType::Mat3 => 9,
            AccessorType::Mat4 => 16,
        }
    }
}

impl std::fmt::Display for AccessorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccessorType::Scalar => write!(f, "SCALAR"),
            AccessorType::Vec2 => write!(f, "VEC2"),
            AccessorType::Vec3 => write!(f, "VEC3"),
            AccessorType::Vec4 => write!(f, "VEC4"),
            AccessorType::Mat2 => write!(f, "MAT2"),
            AccessorType::Mat3 => write!(f, "MAT3"),
            AccessorType::Mat4 => write!(f, "MAT4"),
        }
    }
}

pub trait AccessorOrd: Copy {
    fn min_value() -> Self;
    fn max_value() -> Self;

    fn min(&self, other: &Self) -> Self;
    fn max(&self, other: &Self) -> Self;
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct AccessorVec2<T: Copy> {
    pub x: T,
    pub y: T,
}

impl<T: Copy + std::fmt::Display> std::fmt::Display for AccessorVec2<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.x, self.y)
    }
}

impl AccessorOrd for AccessorVec2<f32> {
    fn min_value() -> Self {
        AccessorVec2 {
            x: f32::MIN,
            y: f32::MIN,
        }
    }

    fn max_value() -> Self {
        AccessorVec2 {
            x: f32::MAX,
            y: f32::MAX,
        }
    }

    fn min(&self, other: &Self) -> Self {
        AccessorVec2 {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
        }
    }

    fn max(&self, other: &Self) -> Self {
        AccessorVec2 {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct AccessorVec3<T: Copy> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl std::fmt::Display for AccessorVec3<f32> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}, {}]", self.x, self.y, self.z)
    }
}

impl AccessorOrd for AccessorVec3<f32> {
    fn min_value() -> Self {
        AccessorVec3 {
            x: f32::MIN,
            y: f32::MIN,
            z: f32::MIN,
        }
    }

    fn max_value() -> Self {
        AccessorVec3 {
            x: f32::MAX,
            y: f32::MAX,
            z: f32::MAX,
        }
    }

    fn min(&self, other: &Self) -> Self {
        AccessorVec3 {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
            z: self.z.min(other.z),
        }
    }

    fn max(&self, other: &Self) -> Self {
        AccessorVec3 {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
            z: self.z.max(other.z),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct AccessorVec4<T: Copy> {
    pub x: T,
    pub y: T,
    pub z: T,
    pub w: T,
}

impl std::fmt::Display for AccessorVec4<f32> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}, {}, {}]", self.x, self.y, self.z, self.w)
    }
}

impl AccessorOrd for AccessorVec4<f32> {
    fn min_value() -> Self {
        AccessorVec4 {
            x: f32::MIN,
            y: f32::MIN,
            z: f32::MIN,
            w: f32::MIN,
        }
    }

    fn max_value() -> Self {
        AccessorVec4 {
            x: f32::MAX,
            y: f32::MAX,
            z: f32::MAX,
            w: f32::MAX,
        }
    }

    fn min(&self, other: &Self) -> Self {
        AccessorVec4 {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
            z: self.z.min(other.z),
            w: self.w.min(other.w),
        }
    }

    fn max(&self, other: &Self) -> Self {
        AccessorVec4 {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
            z: self.z.max(other.z),
            w: self.w.max(other.w),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct AccessorMat2<T: Copy> {
    pub row0: AccessorVec2<T>,
    pub row1: AccessorVec2<T>,
}

impl std::fmt::Display for AccessorMat2<f32> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[[{}, {}], [{}, {}]]",
            self.row0.x, self.row0.y, self.row1.x, self.row1.y
        )
    }
}

impl AccessorOrd for AccessorMat2<f32> {
    fn min_value() -> Self {
        AccessorMat2 {
            row0: AccessorVec2::min_value(),
            row1: AccessorVec2::min_value(),
        }
    }

    fn max_value() -> Self {
        AccessorMat2 {
            row0: AccessorVec2::max_value(),
            row1: AccessorVec2::max_value(),
        }
    }

    fn min(&self, other: &Self) -> Self {
        AccessorMat2 {
            row0: self.row0.min(&other.row0),
            row1: self.row1.min(&other.row1),
        }
    }

    fn max(&self, other: &Self) -> Self {
        AccessorMat2 {
            row0: self.row0.max(&other.row0),
            row1: self.row1.max(&other.row1),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct AccessorMat3<T: Copy> {
    pub row0: AccessorVec3<T>,
    pub row1: AccessorVec3<T>,
    pub row2: AccessorVec3<T>,
}

impl std::fmt::Display for AccessorMat3<f32> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[[{}, {}, {}], [{}, {}, {}], [{}, {}, {}]]",
            self.row0.x,
            self.row0.y,
            self.row0.z,
            self.row1.x,
            self.row1.y,
            self.row1.z,
            self.row2.x,
            self.row2.y,
            self.row2.z
        )
    }
}

impl AccessorOrd for AccessorMat3<f32> {
    fn min_value() -> Self {
        AccessorMat3 {
            row0: AccessorVec3::min_value(),
            row1: AccessorVec3::min_value(),
            row2: AccessorVec3::min_value(),
        }
    }

    fn max_value() -> Self {
        AccessorMat3 {
            row0: AccessorVec3::max_value(),
            row1: AccessorVec3::max_value(),
            row2: AccessorVec3::max_value(),
        }
    }

    fn min(&self, other: &Self) -> Self {
        AccessorMat3 {
            row0: self.row0.min(&other.row0),
            row1: self.row1.min(&other.row1),
            row2: self.row2.min(&other.row2),
        }
    }

    fn max(&self, other: &Self) -> Self {
        AccessorMat3 {
            row0: self.row0.max(&other.row0),
            row1: self.row1.max(&other.row1),
            row2: self.row2.max(&other.row2),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct AccessorMat4<T: Copy> {
    pub row0: AccessorVec4<T>,
    pub row1: AccessorVec4<T>,
    pub row2: AccessorVec4<T>,
    pub row3: AccessorVec4<T>,
}

impl std::fmt::Display for AccessorMat4<f32> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[[{}, {}, {}, {}], [{}, {}, {}, {}], [{}, {}, {}, {}], [{}, {}, {}, {}]]",
            self.row0.x,
            self.row0.y,
            self.row0.z,
            self.row0.w,
            self.row1.x,
            self.row1.y,
            self.row1.z,
            self.row1.w,
            self.row2.x,
            self.row2.y,
            self.row2.z,
            self.row2.w,
            self.row3.x,
            self.row3.y,
            self.row3.z,
            self.row3.w
        )
    }
}

impl AccessorOrd for AccessorMat4<f32> {
    fn min_value() -> Self {
        AccessorMat4 {
            row0: AccessorVec4::min_value(),
            row1: AccessorVec4::min_value(),
            row2: AccessorVec4::min_value(),
            row3: AccessorVec4::min_value(),
        }
    }

    fn max_value() -> Self {
        AccessorMat4 {
            row0: AccessorVec4::max_value(),
            row1: AccessorVec4::max_value(),
            row2: AccessorVec4::max_value(),
            row3: AccessorVec4::max_value(),
        }
    }

    fn min(&self, other: &Self) -> Self {
        AccessorMat4 {
            row0: self.row0.min(&other.row0),
            row1: self.row1.min(&other.row1),
            row2: self.row2.min(&other.row2),
            row3: self.row3.min(&other.row3),
        }
    }

    fn max(&self, other: &Self) -> Self {
        AccessorMat4 {
            row0: self.row0.max(&other.row0),
            row1: self.row1.max(&other.row1),
            row2: self.row2.max(&other.row2),
            row3: self.row3.max(&other.row3),
        }
    }
}
