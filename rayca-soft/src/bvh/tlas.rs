// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

#[derive(Default)]
pub struct TlasNodeBuilder {
    bounds: AABB,
    left: Handle<TlasNode>,
    right: Handle<TlasNode>,
    blas: BvhRange<BlasNode>,
}

impl TlasNodeBuilder {
    fn _bounds(mut self, bounds: AABB) -> Self {
        self.bounds = bounds;
        self
    }

    fn _blas(mut self, blas: BvhRange<BlasNode>) -> Self {
        self.blas = blas;
        self
    }

    pub fn left(mut self, left: Handle<TlasNode>) -> Self {
        self.left = left;
        self
    }

    pub fn right(mut self, right: Handle<TlasNode>) -> Self {
        self.right = right;
        self
    }

    pub fn build(self) -> TlasNode {
        TlasNode::new(self.bounds, self.left, self.right, self.blas)
    }
}

#[derive(Default)]
pub struct TlasNode {
    bounds: AABB,

    pub left: Handle<TlasNode>,
    pub right: Handle<TlasNode>,

    blas: BvhRange<BlasNode>,
}

impl TlasNode {
    fn new(
        bounds: AABB,
        left: Handle<TlasNode>,
        right: Handle<TlasNode>,
        blas: BvhRange<BlasNode>,
    ) -> Self {
        Self {
            bounds,
            left,
            right,
            blas,
        }
    }

    pub fn builder() -> TlasNodeBuilder {
        TlasNodeBuilder::default()
    }

    pub fn is_leaf(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }

    fn replace_models_recursive(&mut self, blas_range: BvhRange<BlasNode>, tlas: &mut Tlas) {
        assert!(!blas_range.is_empty());
        self.blas = blas_range;

        self.bounds.a = Point3::new(f32::MAX, f32::MAX, f32::MAX);
        self.bounds.b = Point3::new(f32::MIN, f32::MIN, f32::MIN);

        // Visit each BLAS to find the lowest and highest x, y, and z
        let a = blas_range.offset;
        let b = a + blas_range.count;
        for i in a..b {
            let blas_node = &tlas.blas_nodes[i as usize];
            let blas = tlas.blass.get(blas_node.blas).unwrap();
            self.bounds.a = self.bounds.a.min(blas.get_root().bounds.a);
            self.bounds.b = self.bounds.b.max(blas.get_root().bounds.b);
        }

        // Split AABB along its longest axis
        let extent = self.bounds.b - self.bounds.a;
        let mut axis = Axis3::X;
        if extent.get_y() > extent.get_x() {
            axis = Axis3::Y;
        }
        if extent.get_z() > extent[axis] {
            axis = Axis3::Z
        }
        let split_pos = self.bounds.a[axis] + extent[axis] * 0.5;

        // Partition-in-place to obtain two groups of models on both sides of the split plane
        let mut i = blas_range.offset as usize;
        let mut j = (blas_range.offset + blas_range.count) as usize;
        while i < j {
            let blas_node = &tlas.blas_nodes[i];
            let bvh = tlas.blass.get(blas_node.blas).unwrap();
            if bvh.get_root().bounds.get_centroid()[axis] < split_pos {
                i += 1;
            } else {
                tlas.blas_nodes.swap(i, j - 1);
                j -= 1;
            }
        }

        // Create child nodes for each half
        let left_count = i as u32 - blas_range.offset;
        let right_count = blas_range.count - left_count;
        if left_count > 0 && right_count > 0 {
            let right_models_range = BvhRange::new(blas_range.offset + left_count, right_count);
            let left_models_range = BvhRange::new(blas_range.offset, left_count);

            // Create two nodes
            let mut left_child = TlasNode::default();
            left_child.replace_models_recursive(left_models_range, tlas);

            let mut right_child = TlasNode::default();
            right_child.replace_models_recursive(right_models_range, tlas);

            self.left = tlas.tlas_nodes.push(left_child);
            self.right = tlas.tlas_nodes.push(right_child);
            self.blas.count = 0;
        }
    }

    fn intersects(&self, scene: &SceneDrawInfo, ray: &Ray, tlas: &Tlas) -> Option<Hit> {
        if self.bounds.intersects(ray) == f32::MAX {
            return None;
        }

        let mut ret = None;

        let mut depth = f32::INFINITY;

        if self.is_leaf() {
            let a = self.blas.offset;
            let b = a + self.blas.count;
            for i in a..b {
                let blas_node = &tlas.blas_nodes[i as usize];
                let bvh = tlas.blass.get(blas_node.blas)?;
                if let Some(mut hit) = bvh.intersects(scene, ray) {
                    if hit.depth < depth {
                        depth = hit.depth;
                        hit.blas = i;
                        ret = Some(hit);
                    }
                }
            }
        } else {
            // TODO: refactor in common code for both nodes
            if let Some(left_node) = tlas.tlas_nodes.get(self.left) {
                if let Some(hit) = left_node.intersects(scene, ray, tlas) {
                    if hit.depth < depth {
                        depth = hit.depth;
                        ret = Some(hit);
                    }
                }
            }

            if let Some(right_node) = tlas.tlas_nodes.get(self.right) {
                if let Some(hit) = right_node.intersects(scene, ray, tlas) {
                    if hit.depth < depth {
                        ret = Some(hit);
                    }
                }
            }
        }

        ret
    }
}

/// The idea of a BLAS node is that we can reorder them in their vector container
/// while keeping the BLAS and its model in their place.
#[derive(Copy, Clone)]
pub struct BlasNode {
    pub blas: Handle<Blas>,
    pub model: Handle<Model>,
}

impl BlasNode {
    pub fn new(blas: Handle<Blas>, model: Handle<Model>) -> Self {
        Self { blas, model }
    }
}

pub struct TlasBuilder {
    scene: BvhScene,
    max_depth: u8,
}

impl Default for TlasBuilder {
    fn default() -> Self {
        Self {
            scene: BvhScene::default(),
            max_depth: u8::MAX,
        }
    }
}

impl TlasBuilder {
    pub fn scene(mut self, scene: BvhScene) -> Self {
        self.scene = scene;
        self
    }

    pub fn max_depth(mut self, max_depth: u8) -> Self {
        self.max_depth = max_depth;
        self
    }

    pub fn build(self, scene_draw_info: &SceneDrawInfo) -> Tlas {
        Tlas::new(scene_draw_info, self.scene, self.max_depth)
    }
}

/// Top Level Acceleration Structure
#[derive(Default)]
pub struct Tlas {
    pub root: TlasNode,

    /// TODO: Use index 0 as root, ignore 1, and children from 2 onwards
    pub tlas_nodes: Pack<TlasNode>,

    /// BLAS nodes are referenced by TLAS nodes
    pub blas_nodes: Vec<BlasNode>,

    /// Bounding Volume Hierarchies should have a 1-1 mapping with model geometries
    /// Bottom Level Acceleration Structures refer a BVH and a Model
    pub blass: Pack<Blas>,
}

impl Tlas {
    pub fn builder() -> TlasBuilder {
        TlasBuilder::default()
    }

    pub fn new(scene_draw_info: &SceneDrawInfo, scene: BvhScene, max_depth: u8) -> Self {
        let mut ret = Self::default();

        let model_count = scene.models.len();
        for (i, model) in scene.models.into_iter().enumerate() {
            let blas_handle = ret.blass.push(
                Blas::builder()
                    .model(model)
                    .max_depth(max_depth)
                    .build(scene_draw_info),
            );
            ret.blas_nodes
                .push(BlasNode::new(blas_handle, Handle::from(i)))
        }

        // TODO: make a function which returns a range of handles from a pack
        let mut root = TlasNode::default();
        let range = BvhRange::new(0, model_count as u32);
        root.replace_models_recursive(range, &mut ret);
        ret.root = root;
        ret
    }

    pub fn intersects(&self, scene: &SceneDrawInfo, ray: Ray) -> Option<Hit> {
        assert!(self.root.blas.count > 0 || self.root.left.is_some() || self.root.right.is_some());
        self.root.intersects(scene, &ray, self)
    }

    pub fn get_blas(&self, blas: u32) -> &Blas {
        let blas_node = &self.blas_nodes[blas as usize];
        self.blass.get(blas_node.blas).unwrap()
    }

    pub fn get_primitive(&self, hit: &Hit) -> &BvhPrimitive {
        let blas = self.get_blas(hit.blas);
        blas.model.get_primitive(hit.primitive)
    }
}
