// Copyright © 2022-2024
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

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

    fn grow_sphere(&mut self, sphere: &BvhSphere) {
        let radius = sphere.get_radius();
        self.grow(&(sphere.center + Vec3::new(-radius, 0.0, 0.0)));
        self.grow(&(sphere.center + Vec3::new(radius, 0.0, 0.0)));
        self.grow(&(sphere.center + Vec3::new(0.0, -radius, 0.0)));
        self.grow(&(sphere.center + Vec3::new(0.0, radius, 0.0)));
        self.grow(&(sphere.center + Vec3::new(0.0, 0.0, -radius)));
        self.grow(&(sphere.center + Vec3::new(0.0, 0.0, radius)));
    }

    fn grow_primitive(&mut self, primitive: &BvhPrimitive) {
        match &primitive.geometry {
            BvhGeometry::Triangle(triangle) => {
                self.grow_triangle(triangle);
            }
            BvhGeometry::Sphere(sphere) => {
                self.grow_sphere(sphere);
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

#[derive(Default)]
pub struct BvhNode {
    bounds: AABB,

    left: Handle<BvhNode>,
    right: Handle<BvhNode>,

    primitives: Vec<BvhPrimitive>,
}

impl BvhNode {
    pub fn new() -> Self {
        Self {
            bounds: Default::default(),
            left: Handle::NONE,
            right: Handle::NONE,
            primitives: vec![],
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }

    pub fn set_primitives(
        &mut self,
        primitives: Vec<BvhPrimitive>,
        max_depth: usize,
        nodes: &mut Pack<BvhNode>,
    ) {
        let mut timer = Timer::new();
        self.set_primitives_recursive(primitives, max_depth, 0, nodes);
        print_success!("BVH", "built in {:.2}ms", timer.get_delta().as_millis());
    }

    /// Surface Area Heuristics:
    /// The cost of a split is proportional to the summed cost of intersecting the two
    /// resulting boxes, including the triangles they store.
    fn evaluate_sah(&self, axis: Axis3, pos: f32) -> f32 {
        // determine triangle counts and bounds for this split candidate
        let mut left_box = AABB::default();
        let mut right_box = AABB::default();
        let mut left_count = 0;
        let mut right_count = 0;

        for pri in &self.primitives {
            let centroid = pri.centroid();
            if centroid[axis] < pos {
                left_count += 1;
                left_box.grow_primitive(pri);
            } else {
                right_count += 1;
                right_box.grow_primitive(pri);
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
    fn find_best_split_plane(&self) -> (Axis3, f32, f32) {
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
                let cost = self.evaluate_sah(axis, candidate_pos);
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
        primitives: Vec<BvhPrimitive>,
        max_depth: usize,
        level: usize,
        nodes: &mut Pack<BvhNode>,
    ) {
        assert!(!primitives.is_empty());
        self.primitives = primitives;

        self.bounds.a = Point3::new(f32::MAX, f32::MAX, f32::MAX);
        self.bounds.b = Point3::new(f32::MIN, f32::MIN, f32::MIN);

        // Visit each vertex of the primitives to find the lowest and highest x, y, and z
        for pri in self.primitives.iter() {
            self.bounds.a = self.bounds.a.min(&pri.min());
            self.bounds.b = self.bounds.b.max(&pri.max());
        }

        if level >= max_depth {
            return;
        }

        // Surface Area Heuristics
        let (split_axis, split_pos, split_cost) = self.find_best_split_plane();

        let no_split_cost = self.calculate_cost();
        if split_cost > no_split_cost {
            return;
        }

        // Partition-in-place to obtain two groups of triangles on both sides of the split plane
        let mut i = 0;
        let mut j = self.primitives.len();
        while i < j {
            let centroid = self.primitives[i].centroid();
            if centroid[split_axis] < split_pos {
                i += 1;
            } else {
                self.primitives.swap(i, j - 1);
                j -= 1;
            }
        }

        // Create child nodes for each half
        let left_count = i;
        let right_count = self.primitives.len() - left_count;
        if left_count > 0 && right_count > 0 {
            let right_triangles = self.primitives.split_off(left_count);
            let left_triangles = self.primitives.split_off(0);

            // Create two nodes
            let mut left_child = BvhNode::new();
            left_child.set_primitives_recursive(left_triangles, max_depth, level + 1, nodes);

            let mut right_child = BvhNode::new();
            right_child.set_primitives_recursive(right_triangles, max_depth, level + 1, nodes);

            self.left = nodes.push(left_child);
            self.right = nodes.push(right_child);
        }
    }

    fn intersects<'b>(
        &'b self,
        model: &Model,
        ray: &Ray,
        nodes: &'b Pack<BvhNode>,
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

            for tri in &self.primitives {
                if let Some(hit) = tri.intersects(model, ray) {
                    if hit.depth < depth {
                        depth = hit.depth;
                        ret = Some((hit, tri));
                    }
                }
            }
        } else {
            if let Some(left_node) = nodes.get(self.left) {
                if let Some((hit, tri)) = left_node.intersects(model, ray, nodes, triangle_count) {
                    if hit.depth < depth {
                        depth = hit.depth;
                        ret = Some((hit, tri));
                    }
                }
            }

            if let Some(right_node) = nodes.get(self.right) {
                if let Some((hit, tri)) = right_node.intersects(model, ray, nodes, triangle_count) {
                    if hit.depth < depth {
                        ret = Some((hit, tri));
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

    pub fn build(self) -> Bvh {
        Bvh::new(self.primitives, self.max_depth)
    }
}

pub struct Bvh {
    pub root: BvhNode,
    pub nodes: Pack<BvhNode>,
    pub triangle_count: usize,
}

impl Bvh {
    pub fn builder() -> BvhBuilder {
        BvhBuilder::new()
    }

    pub fn new(primitives: Vec<BvhPrimitive>, max_depth: usize) -> Self {
        let mut nodes = Pack::new();

        let mut root = BvhNode::new();
        root.bounds = AABB::new(
            Point3::new(f32::MAX, f32::MAX, f32::MAX),
            Point3::new(f32::MIN, f32::MIN, f32::MIN),
        );
        root.set_primitives(primitives, max_depth, &mut nodes);

        Self {
            root,
            nodes,
            triangle_count: 0,
        }
    }

    pub fn intersects_iter(&self, model: &Model, ray: &Ray) -> Option<(Hit, &BvhPrimitive)> {
        let mut node = &self.root;
        let mut stack = vec![];

        let mut ret_hit = None;
        let mut max_depth = f32::MAX;

        loop {
            if node.is_leaf() {
                for tri in &node.primitives {
                    if let Some(hit) = tri.intersects(model, ray) {
                        if hit.depth < max_depth {
                            max_depth = hit.depth;
                            ret_hit = Some((hit, tri));
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
        self.root
            .intersects(model, ray, &self.nodes, &mut triangle_count)
    }

    pub fn intersects_stats(
        &self,
        model: &Model,
        ray: &Ray,
        triangle_count: &mut usize,
    ) -> Option<(Hit, &BvhPrimitive)> {
        self.root
            .intersects(model, ray, &self.nodes, triangle_count)
    }
}

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    fn simple() {
        let mut model = Model::new();
        let triangle_prim = Primitive::unit_triangle();
        let node = model.nodes.push(Node::default());
        let triangles = triangle_prim.primitives(node, Handle::none(), &model);

        let bvh = Bvh::builder().primitives(triangles).build();
        assert!(bvh.nodes.is_empty());
        assert!(!bvh.root.left.valid());
        assert!(!bvh.root.right.valid());
        assert!(!bvh.root.primitives.is_empty());
    }

    #[test]
    fn two_children() {
        let mut model = Model::new();

        let left_triangle_prim = Primitive::builder()
            .vertices(vec![
                Vertex::new(-4.0, 0.0, 0.0),
                Vertex::new(-2.0, 0.0, 0.0),
                Vertex::new(-3.0, 0.3, 0.0),
            ])
            .indices(vec![0, 1, 2])
            .build();
        let right_triangle_prim = Primitive::unit_triangle();

        let node = model.nodes.push(Node::default());
        let mut left_primitives = left_triangle_prim.primitives(node, Handle::none(), &model);
        let mut right_primitives = right_triangle_prim.primitives(node, Handle::none(), &model);
        left_primitives.append(&mut right_primitives);
        let primitives = left_primitives;

        let bvh = Bvh::builder().primitives(primitives).build();
        assert!(!bvh.nodes.is_empty());
        assert!(bvh.root.left.valid());
        assert!(bvh.root.right.valid());
        assert!(bvh.root.primitives.is_empty());
    }
}
