// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

#[derive(Clone)]
/// If the node has primitives it means it is a leaf.
/// Left node index is `triangles.offset`, when the node has no primitives.
/// Right node index is just `left_node_index + 1`.
pub struct BvhNode {
    pub bounds: AABB,

    primitives: BvhRange<BvhPrimitive>,
}

impl Default for BvhNode {
    fn default() -> Self {
        Self {
            bounds: AABB::default(),
            primitives: BvhRange::default(),
        }
    }
}

impl BvhNode {
    pub fn new(blas: &Blas, primitives: BvhRange<BvhPrimitive>, scene: &SceneDrawInfo) -> Self {
        let mut bounds = AABB::new(
            Point3::new(f32::MAX, f32::MAX, f32::MAX),
            Point3::new(f32::MIN, f32::MIN, f32::MIN),
        );

        bounds.grow_range(blas, primitives, scene);

        Self { bounds, primitives }
    }

    pub fn get_left_child_index(&self) -> usize {
        assert!(!self.is_leaf());
        self.primitives.offset as usize
    }

    pub fn get_right_child_index(&self) -> usize {
        self.get_left_child_index() + 1
    }

    pub fn set_left_child_index(&mut self, index: u32) {
        self.primitives.offset = index;
        self.primitives.count = 0;
    }

    pub fn is_leaf(&self) -> bool {
        // 1 is unused, so we use it as a marker for no children
        self.has_primitives()
    }

    pub fn has_primitives(&self) -> bool {
        !self.primitives.is_empty()
    }

    /// Surface Area Heuristics:
    /// The cost of a split is proportional to the summed cost of intersecting the two
    /// resulting boxes, including the triangles they store.
    fn evaluate_sah(&self, blas: &Blas, axis: Axis3, pos: f32, scene: &SceneDrawInfo) -> f32 {
        // determine triangle counts and bounds for this split candidate
        let mut left_box = AABB::default();
        let mut right_box = AABB::default();
        let mut left_count = 0;
        let mut right_count = 0;

        for pri_index in self.primitives {
            let pri = &blas.model.primitives[pri_index];
            let centroid = pri.get_centroid(scene);
            if centroid[axis] < pos {
                left_count += 1;
                left_box.grow_primitive(scene, pri);
            } else {
                right_count += 1;
                right_box.grow_primitive(scene, pri);
            }
        }

        let cost = left_count as f32 * left_box.area() + right_count as f32 * right_box.area();
        if cost > 0.0 {
            cost
        } else {
            f32::MAX
        }
    }

    /// Finds the optimal split plane position and axis
    /// - Returns (split axis, split pos, split cost)
    fn find_best_split_plane(&self, blas: &Blas, scene: &SceneDrawInfo) -> (Axis3, f32, f32) {
        const ALL_AXIS: [Axis3; 3] = [Axis3::X, Axis3::Y, Axis3::Z];

        let mut best_cost = f32::MAX;
        let mut best_axis = Axis3::X;
        let mut split_pos = 0.0;

        for axis in ALL_AXIS {
            let bounds_min = self.bounds.a[axis];
            let bounds_max = self.bounds.b[axis];
            if bounds_min == bounds_max {
                continue;
            }

            // TODO tweak this
            const AREA_COUNT: i32 = 64;
            let scale = (bounds_max - bounds_min) / AREA_COUNT as f32;

            for i in 1..AREA_COUNT {
                let candidate_pos = bounds_min + i as f32 * scale;
                let cost = self.evaluate_sah(blas, axis, candidate_pos, scene);
                if cost < best_cost {
                    best_cost = cost;
                    best_axis = axis;
                    split_pos = candidate_pos;
                }
            }
        }

        (best_axis, split_pos, best_cost)
    }

    fn calculate_cost(&self) -> f32 {
        self.primitives.len() as f32 * self.bounds.area()
    }

    fn intersects<'b>(
        &'b self,
        scene: &SceneDrawInfo,
        ray: &Ray,
        blas: &'b Blas,
        triangle_count: &mut usize,
    ) -> Option<(Hit, &'b BvhPrimitive)> {
        let d = self.bounds.intersects(ray);
        if d == f32::MAX {
            return None;
        }

        let mut ret = None;

        let mut depth = f32::INFINITY;

        if self.is_leaf() {
            *triangle_count += self.primitives.len();

            for pri_index in &self.primitives {
                let pri = &blas.model.primitives[pri_index];
                if let Some(mut hit) = pri.intersects(scene, ray) {
                    if hit.depth < depth {
                        depth = hit.depth;
                        hit.primitive = pri_index as u32;
                        ret = Some((hit, pri));
                    }
                }
            }
        } else {
            let left_node = &blas.nodes[self.get_left_child_index()];
            if let Some((hit, pri)) = left_node.intersects(scene, ray, blas, triangle_count) {
                if hit.depth < depth {
                    depth = hit.depth;
                    ret = Some((hit, pri));
                }
            }

            let right_node = &blas.nodes[self.get_right_child_index()];
            if let Some((hit, pri)) = right_node.intersects(scene, ray, blas, triangle_count) {
                if hit.depth < depth {
                    ret = Some((hit, pri));
                }
            }
        }

        ret
    }
}

pub struct BlasBuilder {
    model: BvhModel,
    max_depth: u8,
}

impl Default for BlasBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl BlasBuilder {
    pub fn new() -> Self {
        Self {
            model: BvhModel::default(),
            max_depth: u8::MAX,
        }
    }

    pub fn model(mut self, model: BvhModel) -> Self {
        self.model = model;
        self
    }

    pub fn max_depth(mut self, max_depth: u8) -> Self {
        self.max_depth = max_depth;
        self
    }

    pub fn build(self, scene: &SceneDrawInfo) -> Blas {
        Blas::new(self.max_depth, scene, self.model)
    }
}

#[derive(Default)]
pub struct Blas {
    pub max_depth: u8,

    /// First node in the vector is root, second node is unused.
    pub nodes: Vec<BvhNode>,
    pub triangle_count: usize,

    pub model: BvhModel,
}

impl Blas {
    pub fn builder() -> BlasBuilder {
        BlasBuilder::new()
    }

    fn new(max_depth: u8, scene: &SceneDrawInfo, model: BvhModel) -> Self {
        let mut timer = Timer::new();

        let mut ret = Self {
            max_depth,
            model,
            ..Default::default()
        };

        ret.set_primitives(scene);

        log::debug!("Created BLAS in {:.2}s", timer.get_delta().as_secs_f32());

        ret
    }

    pub fn set_primitives(&mut self, scene: &SceneDrawInfo) {
        self.nodes.clear();

        let primitive_range = BvhRange::new(0, self.model.primitives.len() as u32);

        // Node at index 0 is root, at the beginning, it contains the whole scene
        let root_node = BvhNode::new(self, primitive_range, scene);
        self.nodes.push(root_node);

        // Node at index 1 is unused
        let dummy_node = BvhNode::default();
        self.nodes.push(dummy_node);

        self.set_primitives_recursive(0, 0, scene);
    }

    fn set_primitives_recursive(&mut self, node_index: usize, level: u8, scene: &SceneDrawInfo) {
        if level >= self.max_depth {
            return;
        }

        // Clone node to store in self.nodes[node_index] later again
        let mut node = self.nodes[node_index].clone();
        assert!(node.has_primitives());

        // Surface Area Heuristics
        let (split_axis, split_pos, split_cost) = node.find_best_split_plane(self, scene);

        let no_split_cost = node.calculate_cost();
        if split_cost > no_split_cost {
            return;
        }

        // Partition-in-place to obtain two groups of triangles on both sides of the split plane
        let mut i_tri = node.primitives.get_start();
        let mut j_tri = node.primitives.get_end();
        while i_tri < j_tri {
            let centroid = &self.model.primitives[i_tri].get_centroid(scene);
            if centroid[split_axis] < split_pos {
                i_tri += 1;
            } else {
                self.model.primitives.swap(i_tri, j_tri - 1);
                j_tri -= 1;
            }
        }

        // Create child nodes for each half
        let tri_left_count = i_tri as u32 - node.primitives.offset;
        let tri_right_count = node.primitives.count - tri_left_count;

        if tri_left_count > 0 && tri_right_count > 0 {
            let left_primitives = BvhRange::new(node.primitives.offset, tri_left_count);
            let right_primitives =
                BvhRange::new(node.primitives.offset + tri_left_count, tri_right_count);

            // This node is not a leaf, hence does not contain primitives anymore
            node.primitives = BvhRange::default();
            node.set_left_child_index(self.nodes.len() as u32);

            // Create two nodes
            let left_child = BvhNode::new(self, left_primitives, scene);
            let right_child = BvhNode::new(self, right_primitives, scene);
            self.nodes.push(left_child);
            self.nodes.push(right_child);

            self.set_primitives_recursive(node.get_left_child_index(), level + 1, scene);
            self.set_primitives_recursive(node.get_right_child_index(), level + 1, scene);
        }

        // Save current node
        self.nodes[node_index] = node;
    }

    pub fn get_root(&self) -> &BvhNode {
        &self.nodes[0]
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty() || self.get_root().is_leaf()
    }

    pub fn intersects_iter(
        &self,
        scene: &SceneDrawInfo,
        ray: &Ray,
    ) -> Option<(Hit, &BvhPrimitive)> {
        let mut node = self.get_root();
        let mut stack = vec![];

        let mut ret_hit = None;
        let mut max_depth = f32::MAX;

        loop {
            if node.is_leaf() {
                for pri_index in &node.primitives {
                    let pri = &self.model.primitives[pri_index];
                    if let Some(hit) = pri.intersects(scene, ray) {
                        if hit.depth < max_depth {
                            max_depth = hit.depth;
                            ret_hit = Some((hit, pri));
                        }
                    }
                }
                if stack.is_empty() {
                    break;
                } else {
                    node = stack.pop().unwrap();
                }

                continue;
            }

            let mut child1 = &self.nodes[node.get_left_child_index()];
            let mut child2 = &self.nodes[node.get_right_child_index()];
            let mut dist1 = child1.bounds.intersects(ray);
            let mut dist2 = child2.bounds.intersects(ray);

            if dist1 > dist2 {
                std::mem::swap(&mut dist1, &mut dist2);
                std::mem::swap(&mut child1, &mut child2);
            }
            if dist1 == f32::MAX {
                if stack.is_empty() {
                    break;
                } else {
                    node = stack.pop().unwrap();
                }
            } else {
                node = child1;
                if dist2 != f32::MAX {
                    stack.push(child2);
                }
            }
        }

        ret_hit
    }

    pub fn intersects(&self, scene: &SceneDrawInfo, ray: &Ray) -> Option<(Hit, &BvhPrimitive)> {
        let mut triangle_count = 0;
        self.get_root()
            .intersects(scene, ray, self, &mut triangle_count)
    }

    pub fn intersects_stats(
        &self,
        scene: &SceneDrawInfo,
        ray: &Ray,
        triangle_count: &mut usize,
    ) -> Option<(Hit, &BvhPrimitive)> {
        self.get_root().intersects(scene, ray, self, triangle_count)
    }
}

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    fn simple() {
        let mut scene = Scene::default();

        let mut model = Model::default();
        let geometry_handle = model
            .geometries
            .push(Geometry::TriangleMesh(TriangleMesh::unit()));

        let primitive_handle = model
            .primitives
            .push(Primitive::builder().geometry(geometry_handle).build());
        let mesh = Mesh::builder().primitive(primitive_handle).build();
        let node = model
            .nodes
            .push(Node::builder().mesh(model.meshes.push(mesh)).build());
        model.root.children.push(node);

        scene.push_model(model);

        let scene_draw_info = SceneDrawInfo::new(&scene);

        let model = BvhModel::from_model(
            scene_draw_info.model_draw_infos.values().next().unwrap(),
            &scene_draw_info,
        );

        let blas = Blas::builder().model(model).build(&scene_draw_info);
        assert!(blas.is_empty());
        assert!(blas.get_root().is_leaf()); // no children
        assert!(!blas.get_root().primitives.is_empty());
    }

    #[test]
    fn two_children() {
        let mut model = Model::default();
        let triangle_mesh = TriangleMesh::builder()
            .vertices(vec![
                Vertex::builder()
                    .position(Point3::new(-4.0, 0.0, 0.0))
                    .build(),
                Vertex::builder()
                    .position(Point3::new(-2.0, 0.0, 0.0))
                    .build(),
                Vertex::builder()
                    .position(Point3::new(-3.0, 0.3, 0.0))
                    .build(),
            ])
            .indices(TriangleIndices::builder().indices(vec![0, 1, 2]).build())
            .build();
        let geometry_handle = model.geometries.push(Geometry::TriangleMesh(triangle_mesh));
        let right_triangle_prim = model
            .primitives
            .push(Primitive::builder().geometry(geometry_handle).build());

        let right_geometry_handle = model
            .geometries
            .push(Geometry::TriangleMesh(TriangleMesh::unit()));
        let left_triangle_prim = model
            .primitives
            .push(Primitive::builder().geometry(right_geometry_handle).build());

        let mesh = Mesh::builder()
            .primitives(vec![left_triangle_prim, right_triangle_prim])
            .build();
        let node = model
            .nodes
            .push(Node::builder().mesh(model.meshes.push(mesh)).build());
        model.root.children.push(node);

        let mut scene = Scene::default();
        let model_handle = scene.models.push(model);
        let node = Node::builder().model(model_handle).build();
        let node_handle = scene.nodes.push(node);
        scene.root.children.push(node_handle);

        let scene_draw_info = SceneDrawInfo::new(&scene);

        let model = BvhModel::from_model(
            scene_draw_info.model_draw_infos.values().next().unwrap(),
            &scene_draw_info,
        );

        let blas = Blas::builder().model(model).build(&scene_draw_info);
        assert!(!blas.nodes.is_empty());
        assert!(!blas.get_root().is_leaf()); // has children
        assert!(blas.get_root().primitives.is_empty());
    }
}
