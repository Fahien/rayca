// Copyright © 2022-2024
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::{marker::PhantomData, ops::Range};

use crate::*;

#[derive(Default)]
pub struct AABB {
    pub a: Point3,
    pub b: Point3,
}

impl AABB {
    pub fn new(a: Point3, b: Point3) -> Self {
        Self { a, b }
    }

    fn area(&self) -> f32 {
        let e = self.b - self.a; // box extent
        e.simd[0] * e.simd[1] + e.simd[1] * e.simd[2] + e.simd[2] * e.simd[0]
    }

    fn grow(&mut self, p: &Point3) {
        self.a = self.a.min(p);
        self.b = self.b.max(p);
    }

    fn grow_triangle(&mut self, triangle: &BvhTriangle) {
        self.grow(&triangle.vertices[0].pos);
        self.grow(&triangle.vertices[1].pos);
        self.grow(&triangle.vertices[2].pos);
    }

    fn grow_sphere(&mut self, sphere: &BvhSphere, trs: &Trs) {
        let radius = sphere.get_radius();
        let center = trs * sphere.center;
        self.grow(&(center + Vec3::new(-radius, 0.0, 0.0)));
        self.grow(&(center + Vec3::new(radius, 0.0, 0.0)));
        self.grow(&(center + Vec3::new(0.0, -radius, 0.0)));
        self.grow(&(center + Vec3::new(0.0, radius, 0.0)));
        self.grow(&(center + Vec3::new(0.0, 0.0, -radius)));
        self.grow(&(center + Vec3::new(0.0, 0.0, radius)));
    }

    fn grow_primitive(&mut self, model: &Model, primitive: &BvhPrimitive) {
        match &primitive.geometry {
            BvhGeometry::Triangle(triangle) => {
                self.grow_triangle(triangle);
            }
            BvhGeometry::Sphere(sphere) => {
                let trs = model.solved_trs.get(&primitive.node).unwrap();
                self.grow_sphere(sphere, &trs.trs);
            }
        }
    }

    /// Slab test. We do not care where we hit the box; only info we need is a yes/no answer.
    fn intersects(&self, ray: &Ray) -> f32 {
        let origin_vec = Vec3::from(ray.origin);
        let t1 = (self.a - origin_vec) * ray.rdir;
        let t2 = (self.b - origin_vec) * ray.rdir;

        let vmax = t1.max(&t2);
        let vmin = t1.min(&t2);

        let tmax = vmax.simd[0].min(vmax.simd[1].min(vmax.simd[2]));
        let tmin = vmin.simd[0].max(vmin.simd[1].max(vmin.simd[2]));

        if tmax >= tmin && tmax > 0.0 {
            tmin
        } else {
            f32::MAX
        }
    }
}

#[derive(Clone, Copy)]
pub struct BvhRange<T> {
    offset: u32,
    count: u32,
    phantom: PhantomData<T>,
}

impl<T> Default for BvhRange<T> {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

impl<T> BvhRange<T> {
    pub fn new(offset: u32, count: u32) -> Self {
        Self {
            offset,
            count,
            phantom: PhantomData,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn len(&self) -> usize {
        self.count as usize
    }

    pub fn split_off(&mut self, offset: usize) -> Self {
        let o = offset as u32;
        let right_count = self.count - o;
        self.count = o;
        Self::new(self.offset + o, right_count)
    }
}

impl<T> IntoIterator for BvhRange<T> {
    type Item = usize;
    type IntoIter = Range<usize>;

    fn into_iter(self) -> Self::IntoIter {
        self.offset as usize..(self.offset as usize + self.count as usize)
    }
}

impl<T> IntoIterator for &BvhRange<T> {
    type Item = usize;
    type IntoIter = Range<usize>;

    fn into_iter(self) -> Self::IntoIter {
        self.offset as usize..(self.offset as usize + self.count as usize)
    }
}

#[derive(Default)]
pub struct BvhNode {
    bounds: AABB,

    left: Handle<BvhNode>,
    right: Handle<BvhNode>,

    primitives: BvhRange<BvhPrimitive>,
}

impl BvhNode {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_leaf(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }

    pub fn set_primitives(
        &mut self,
        model: &Model,
        primitives_range: BvhRange<BvhPrimitive>,
        primitives: &mut [BvhPrimitive],
        max_depth: usize,
        nodes: &mut Pack<BvhNode>,
    ) {
        let mut timer = Timer::new();
        self.set_primitives_recursive(model, primitives_range, primitives, max_depth, 0, nodes);
        log::info!("BVH built in {:.2}ms", timer.get_delta().as_millis());
    }

    /// Surface Area Heuristics:
    /// The cost of a split is proportional to the summed cost of intersecting the two
    /// resulting boxes, including the triangles they store.
    fn evaluate_sah(
        &self,
        model: &Model,
        axis: Axis3,
        pos: f32,
        primitives: &[BvhPrimitive],
    ) -> f32 {
        // determine triangle counts and bounds for this split candidate
        let mut left_box = AABB::default();
        let mut right_box = AABB::default();
        let mut left_count = 0;
        let mut right_count = 0;

        for pri_index in &self.primitives {
            let pri = &primitives[pri_index];
            let centroid = pri.centroid(model);
            if centroid[axis] < pos {
                left_count += 1;
                left_box.grow_primitive(model, pri);
            } else {
                right_count += 1;
                right_box.grow_primitive(model, pri);
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
    fn find_best_split_plane(
        &self,
        model: &Model,
        primitives: &[BvhPrimitive],
    ) -> (Axis3, f32, f32) {
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
                let cost = self.evaluate_sah(model, axis, candidate_pos, primitives);
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

    fn set_primitives_recursive(
        &mut self,
        model: &Model,
        primitives_range: BvhRange<BvhPrimitive>,
        primitives: &mut [BvhPrimitive],
        max_depth: usize,
        level: usize,
        nodes: &mut Pack<BvhNode>,
    ) {
        assert!(!primitives.is_empty());
        self.primitives = primitives_range;

        self.bounds.a = Point3::new(f32::MAX, f32::MAX, f32::MAX);
        self.bounds.b = Point3::new(f32::MIN, f32::MIN, f32::MIN);

        // Visit each vertex of the primitives to find the lowest and highest x, y, and z
        for pri_index in &self.primitives {
            let pri = &primitives[pri_index];
            self.bounds.a = self.bounds.a.min(&pri.min(model));
            self.bounds.b = self.bounds.b.max(&pri.max(model));
        }

        if level >= max_depth {
            return;
        }

        // Surface Area Heuristics
        let (split_axis, split_pos, split_cost) = self.find_best_split_plane(model, primitives);

        let no_split_cost = self.calculate_cost();
        if split_cost > no_split_cost {
            return;
        }

        // Partition-in-place to obtain two groups of triangles on both sides of the split plane
        let mut i = 0;
        let mut j = self.primitives.len();
        while i < j {
            let index = self.primitives.offset as usize + i;

            let centroid = primitives[index].centroid(model);
            if centroid[split_axis] < split_pos {
                i += 1;
            } else {
                let jndex = self.primitives.offset as usize + j;
                primitives.swap(index, jndex - 1);
                j -= 1;
            }
        }

        // Create child nodes for each half
        let left_count = i;
        let right_count = self.primitives.len() - left_count;
        if left_count > 0 && right_count > 0 {
            let right_primitives = self.primitives.split_off(left_count);
            let left_primitives = self.primitives.split_off(0);

            // Create two nodes
            let mut left_child = BvhNode::new();
            left_child.set_primitives_recursive(
                model,
                left_primitives,
                primitives,
                max_depth,
                level + 1,
                nodes,
            );

            let mut right_child = BvhNode::new();
            right_child.set_primitives_recursive(
                model,
                right_primitives,
                primitives,
                max_depth,
                level + 1,
                nodes,
            );

            self.left = nodes.push(left_child);
            self.right = nodes.push(right_child);
        }
    }

    fn intersects<'b>(
        &'b self,
        model: &Model,
        ray: &Ray,
        bvh: &'b Bvh,
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
                let pri = &bvh.primitives[pri_index];
                if let Some(hit) = pri.intersects(model, ray) {
                    if hit.depth < depth {
                        depth = hit.depth;
                        ret = Some((hit, pri));
                    }
                }
            }
        } else {
            if let Some(left_node) = bvh.nodes.get(self.left) {
                if let Some((hit, pri)) = left_node.intersects(model, ray, bvh, triangle_count) {
                    if hit.depth < depth {
                        depth = hit.depth;
                        ret = Some((hit, pri));
                    }
                }
            }

            if let Some(right_node) = bvh.nodes.get(self.right) {
                if let Some((hit, pri)) = right_node.intersects(model, ray, bvh, triangle_count) {
                    if hit.depth < depth {
                        ret = Some((hit, pri));
                    }
                }
            }
        }

        ret
    }
}

pub struct BvhBuilder {
    primitives: Vec<BvhPrimitive>,
    max_depth: usize,
}

impl Default for BvhBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl BvhBuilder {
    pub fn new() -> Self {
        Self {
            primitives: vec![],
            max_depth: usize::MAX,
        }
    }

    pub fn primitives(mut self, primitives: Vec<BvhPrimitive>) -> Self {
        self.primitives = primitives;
        self
    }

    pub fn max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = max_depth;
        self
    }

    pub fn build(self, model: &Model) -> Bvh {
        Bvh::new(model, self.primitives, self.max_depth)
    }
}

pub struct Bvh {
    pub root: BvhNode,
    pub nodes: Pack<BvhNode>,
    pub triangle_count: usize,

    pub primitives: Vec<BvhPrimitive>,
}

impl Bvh {
    pub fn builder() -> BvhBuilder {
        BvhBuilder::new()
    }

    pub fn new(model: &Model, mut primitives: Vec<BvhPrimitive>, max_depth: usize) -> Self {
        let mut nodes = Pack::new();

        let mut root = BvhNode::new();
        root.bounds = AABB::new(
            Point3::new(f32::MAX, f32::MAX, f32::MAX),
            Point3::new(f32::MIN, f32::MIN, f32::MIN),
        );
        let range = BvhRange::new(0, primitives.len() as u32);
        root.set_primitives(model, range, &mut primitives, max_depth, &mut nodes);

        Self {
            root,
            nodes,
            triangle_count: 0,
            primitives,
        }
    }

    pub fn intersects_iter(&self, model: &Model, ray: &Ray) -> Option<(Hit, &BvhPrimitive)> {
        let mut node = &self.root;
        let mut stack = vec![];

        let mut ret_hit = None;
        let mut max_depth = f32::MAX;

        loop {
            if node.is_leaf() {
                for pri_index in &node.primitives {
                    let pri = &self.primitives[pri_index];
                    if let Some(hit) = pri.intersects(model, ray) {
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

            let mut child1 = self.nodes.get(node.left).unwrap();
            let mut child2 = self.nodes.get(node.right).unwrap();
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

    pub fn intersects(&self, model: &Model, ray: &Ray) -> Option<(Hit, &BvhPrimitive)> {
        let mut triangle_count = 0;
        self.root.intersects(model, ray, self, &mut triangle_count)
    }

    pub fn intersects_stats(
        &self,
        model: &Model,
        ray: &Ray,
        triangle_count: &mut usize,
    ) -> Option<(Hit, &BvhPrimitive)> {
        self.root.intersects(model, ray, self, triangle_count)
    }
}

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    fn simple() {
        let mut model = Model::new();
        let triangle_prim = Primitive::unit_triangle();
        let mesh = Mesh::builder()
            .primitives(vec![model.primitives.push(triangle_prim)])
            .build();
        let node = model
            .nodes
            .push(Node::builder().mesh(model.meshes.push(mesh)).build());
        model.root.children.push(node);
        let primitives = model.collect();

        let bvh = Bvh::builder().primitives(primitives).build(&model);
        assert!(bvh.nodes.is_empty());
        assert!(!bvh.root.left.is_valid());
        assert!(!bvh.root.right.is_valid());
        assert!(!bvh.root.primitives.is_empty());
    }

    #[test]
    fn two_children() {
        let mut model = Model::new();

        let left_triangle_prim = model.primitives.push(
            Primitive::builder()
                .vertices(vec![
                    Vertex::new(-4.0, 0.0, 0.0),
                    Vertex::new(-2.0, 0.0, 0.0),
                    Vertex::new(-3.0, 0.3, 0.0),
                ])
                .indices(vec![0, 1, 2])
                .build(),
        );
        let right_triangle_prim = model.primitives.push(Primitive::unit_triangle());

        let mesh = Mesh::builder()
            .primitives(vec![left_triangle_prim, right_triangle_prim])
            .build();
        let node = model
            .nodes
            .push(Node::builder().mesh(model.meshes.push(mesh)).build());
        model.root.children.push(node);
        let primitives = model.collect();

        let bvh = Bvh::builder().primitives(primitives).build(&model);
        assert!(!bvh.nodes.is_empty());
        assert!(bvh.root.left.is_valid());
        assert!(bvh.root.right.is_valid());
        assert!(bvh.root.primitives.is_empty());
    }
}
