// Copyright © 2022-2023
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::simd::{f32x4, SimdFloat};

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

    pub fn centroid(&self) -> Point3 {
        ((self.b - self.a) / 2.0).into()
    }

    fn area(&self) -> f32 {
        let e = self.b - self.a; // box extent
        e.simd[0] * e.simd[1] + e.simd[1] * e.simd[2] + e.simd[2] * e.simd[0]
    }

    fn grow_range(&mut self, bvh: &Bvh, range: BvhRange, primitives: &[impl Intersect]) {
        // Visit each primitive to find the lowest and highest x, y, and z
        let a = range.offset;
        let b = a + range.count;
        for i in a..b {
            let prim = &primitives[i as usize];
            self.a = self.a.min(&prim.min(bvh));
            self.b = self.b.max(&prim.max(bvh));
        }
    }

    fn grow(&mut self, p: &Point3) {
        self.a = self.a.min(p);
        self.b = self.b.max(p);
    }

    fn grow_triangle(&mut self, triangle: &Triangle) {
        self.grow(&triangle.vertices[0]);
        self.grow(&triangle.vertices[1]);
        self.grow(&triangle.vertices[2]);
    }

    fn grow_sphere(&mut self, sphere: &Sphere, bvh: &Bvh) {
        let radius = sphere.get_radius();
        let center = sphere.get_center(bvh);
        self.grow(&(center + Vec3::new(-radius, 0.0, 0.0)));
        self.grow(&(center + Vec3::new(radius, 0.0, 0.0)));
        self.grow(&(center + Vec3::new(0.0, -radius, 0.0)));
        self.grow(&(center + Vec3::new(0.0, radius, 0.0)));
        self.grow(&(center + Vec3::new(0.0, 0.0, -radius)));
        self.grow(&(center + Vec3::new(0.0, 0.0, radius)));
    }

    /// Slab test. We do not care where we hit the box; only info we need is a yes/no answer.
    fn intersects(&self, ray: &Ray) -> f32 {
        let origin_vec = Vec3::from(ray.origin);
        let t1 = (self.a - origin_vec) * ray.rdir;
        let t2 = (self.b - origin_vec) * ray.rdir;

        static WMAX: f32x4 = f32x4::from_array([1.0, 1.0, 1.0, f32::MAX]);
        static WMIN: f32x4 = f32x4::from_array([1.0, 1.0, 1.0, f32::MIN]);

        let vmax = t1.max(&t2).simd * WMAX;
        let vmin = t1.min(&t2).simd * WMIN;

        let tmax = vmax.reduce_min();
        let tmin = vmin.reduce_max();

        if tmax >= tmin && tmax > 0.0 {
            tmin
        } else {
            f32::MAX
        }
    }
}

/// I could use std::Range<u32>, but that is not #[repr(C)]
/// TODO: Whould it work aliasing the type and putting #[repr(C)] on it?
#[derive(Clone, Copy, Default)]
struct BvhRange {
    offset: u32,
    count: u32,
}

impl BvhRange {
    fn new(offset: u32, count: u32) -> Self {
        Self { offset, count }
    }

    fn is_empty(&self) -> bool {
        self.count == 0
    }
}

#[derive(Default)]
pub struct BvhNode {
    bounds: AABB,

    left: Handle<BvhNode>,
    right: Handle<BvhNode>,

    triangles: BvhRange,
    spheres: BvhRange,
}

impl BvhNode {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_leaf(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }

    fn set_primitives(
        &mut self,
        bvh: &mut Bvh,
        triangles_range: BvhRange,
        spheres_range: BvhRange,
    ) {
        let mut timer = Timer::new();
        self.set_primitives_recursive(bvh, triangles_range, spheres_range, 0);
        print_success!("Built", "BVH in {:.2}ms", timer.get_delta().as_millis());
    }

    /// Surface Area Heuristics:
    /// The cost of a split is proportional to the summed cost of intersecting the two
    /// resulting boxes, including the triangles they store.
    fn evaluate_sah(&self, bvh: &Bvh, axis: Axis3, pos: f32) -> f32 {
        // determine triangle counts and bounds for this split candidate
        let mut left_box = AABB::default();
        let mut right_box = AABB::default();
        let mut left_count = 0;
        let mut right_count = 0;

        let a = self.triangles.offset as usize;
        let b = a + self.triangles.count as usize;
        for tri in bvh.triangles.iter().take(b).skip(a) {
            if tri.get_centroid(bvh)[axis] < pos {
                left_count += 1;
                left_box.grow_triangle(tri);
            } else {
                right_count += 1;
                right_box.grow_triangle(tri);
            }
        }

        let a = self.spheres.offset as usize;
        let b = a + self.spheres.count as usize;
        for sphere in bvh.spheres.iter().take(b).skip(a) {
            if sphere.get_center(bvh)[axis] < pos {
                left_count += 1;
                left_box.grow_sphere(sphere, bvh);
            } else {
                right_count += 1;
                right_box.grow_sphere(sphere, bvh);
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
    fn find_best_split_plane(&self, bvh: &Bvh) -> (Axis3, f32, f32) {
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
                let cost = self.evaluate_sah(bvh, axis, candidate_pos);
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
        (self.triangles.count + self.spheres.count) as f32 * self.bounds.area()
    }

    /// Partition-in-place to obtain two groups of triangles on both sides of the split plane
    fn partition_triangles_in_place(
        bvh: &mut Bvh,
        split_axis: Axis3,
        split_pos: f32,
        range: BvhRange,
    ) -> u32 {
        let mut i = range.offset as usize;
        let mut j = (range.offset + range.count) as usize;

        while i < j {
            if bvh.triangles[i].get_centroid(bvh)[split_axis] < split_pos {
                i += 1;
            } else {
                bvh.triangles.swap(i, j - 1);
                bvh.triangles_ex.swap(i, j - 1);
                j -= 1;
            }
        }

        i as u32
    }

    /// Partition-in-place to obtain two groups of spheres on both sides of the split plane
    fn partition_spheres_in_place(
        bvh: &mut Bvh,
        split_axis: Axis3,
        split_pos: f32,
        range: BvhRange,
    ) -> u32 {
        let mut i = range.offset as usize;
        let mut j = (range.offset + range.count) as usize;

        while i < j {
            if bvh.spheres[i].get_centroid(bvh)[split_axis] < split_pos {
                i += 1;
            } else {
                bvh.spheres.swap(i, j - 1);
                bvh.spheres_ex.swap(i, j - 1);
                j -= 1;
            }
        }

        i as u32
    }

    fn set_primitives_recursive(
        &mut self,
        bvh: &mut Bvh,
        triangles_range: BvhRange,
        spheres_range: BvhRange,
        level: u8,
    ) {
        assert!(!triangles_range.is_empty() || !spheres_range.is_empty());
        self.triangles = triangles_range;
        self.spheres = spheres_range;

        self.bounds.a = Point3::new(f32::MAX, f32::MAX, f32::MAX);
        self.bounds.b = Point3::new(f32::MIN, f32::MIN, f32::MIN);

        self.bounds.grow_range(bvh, triangles_range, &bvh.triangles);
        self.bounds.grow_range(bvh, spheres_range, &bvh.spheres);

        if level >= bvh.max_depth {
            return;
        }

        // Surface Area Heuristics
        let (split_axis, split_pos, split_cost) = self.find_best_split_plane(bvh);

        let no_split_cost = self.calculate_cost();
        if split_cost > no_split_cost {
            return;
        }

        let i_tri = Self::partition_triangles_in_place(bvh, split_axis, split_pos, triangles_range);
        let i_sph = Self::partition_spheres_in_place(bvh, split_axis, split_pos, spheres_range);

        // Create child nodes for each half
        let tri_left_count = i_tri - triangles_range.offset;
        let tri_right_count = triangles_range.count - tri_left_count;

        let sph_left_count = i_sph - spheres_range.offset;
        let sph_right_count = spheres_range.count - sph_left_count;

        if (tri_left_count > 0 && tri_right_count > 0)
            || (sph_left_count > 0 && sph_right_count > 0)
        {
            let right_triangles_range =
                BvhRange::new(triangles_range.offset + tri_left_count, tri_right_count);
            let left_triangles_range = BvhRange::new(triangles_range.offset, tri_left_count);

            let right_spheres_range =
                BvhRange::new(spheres_range.offset + sph_left_count, sph_right_count);
            let left_spheres_range = BvhRange::new(spheres_range.offset, sph_left_count);

            // Create two nodes
            let mut left_child = BvhNode::new();
            left_child.set_primitives_recursive(
                bvh,
                left_triangles_range,
                left_spheres_range,
                level + 1,
            );

            let mut right_child = BvhNode::new();
            right_child.set_primitives_recursive(
                bvh,
                right_triangles_range,
                right_spheres_range,
                level + 1,
            );

            self.left = bvh.nodes.push(left_child);
            self.right = bvh.nodes.push(right_child);
            self.triangles.count = 0;
        }
    }

    fn intersects(
        &self,
        bvh: &Bvh,
        ray: &Ray,
        nodes: &Pack<BvhNode>,
        triangles: &[Triangle],
        spheres: &[Sphere],
        primitive_count: &mut usize,
    ) -> Option<Hit> {
        if self.bounds.intersects(ray) == f32::MAX {
            return None;
        }

        let mut ret = None;

        let mut depth = f32::INFINITY;

        if self.is_leaf() {
            *primitive_count += (self.triangles.count + self.spheres.count) as usize;

            let a = self.triangles.offset;
            let b = a + self.triangles.count;
            for i in a..b {
                let tri = &triangles[i as usize];
                if let Some(mut hit) = tri.intersects(bvh, ray) {
                    if hit.depth < depth {
                        depth = hit.depth;
                        hit.primitive = i;
                        ret = Some(hit);
                    }
                }
            }

            let a = self.spheres.offset;
            let b = a + self.spheres.count;
            for i in a..b {
                let prim = &spheres[i as usize];
                if let Some(mut hit) = prim.intersects(bvh, ray) {
                    if hit.depth < depth {
                        depth = hit.depth;
                        hit.primitive = i;
                        ret = Some(hit);
                    }
                }
            }
        } else {
            if let Some(left_node) = nodes.get(self.left) {
                if let Some(hit) =
                    left_node.intersects(bvh, ray, nodes, triangles, spheres, primitive_count)
                {
                    if hit.depth < depth {
                        depth = hit.depth;
                        ret = Some(hit);
                    }
                }
            }

            if let Some(right_node) = nodes.get(self.right) {
                if let Some(hit) =
                    right_node.intersects(bvh, ray, nodes, triangles, spheres, primitive_count)
                {
                    if hit.depth < depth {
                        ret = Some(hit);
                    }
                }
            }
        }

        ret
    }
}

pub struct BvhBuilder<'a> {
    max_depth: u8,
    model: Option<&'a GltfModel>,
}

impl<'a> Default for BvhBuilder<'a> {
    fn default() -> Self {
        Self {
            max_depth: u8::MAX,
            model: None,
        }
    }
}

impl<'a> BvhBuilder<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn max_depth(mut self, max_depth: u8) -> Self {
        self.max_depth = max_depth;
        self
    }

    pub fn model(mut self, model: &'a GltfModel) -> Self {
        self.model = Some(model);
        self
    }

    pub fn build(self) -> Bvh {
        Bvh::new(self.max_depth, self.model.unwrap())
    }
}

#[derive(Default)]
pub struct Bvh {
    pub max_depth: u8,
    pub root: BvhNode,
    pub nodes: Pack<BvhNode>,

    /// This is needed for sphere intersections, where we will need to transform
    /// the ray to model space and then transform the result back to world space.
    pub trss: Pack<SolvedTrs>,
    pub cameras: Vec<SolvedCamera>,

    pub triangles: Vec<Triangle>,
    pub triangles_ex: Vec<TriangleEx>,

    pub spheres: Vec<Sphere>,
    pub spheres_ex: Vec<SphereEx>,
}

impl Bvh {
    fn new(max_depth: u8, model: &GltfModel) -> Self {
        let mut timer = Timer::new();

        let mut ret = Self {
            max_depth,
            trss: model.collect_trss(),
            ..Default::default()
        };

        for i in 0..ret.trss.len() {
            let trs = &ret.trss[i];

            // Collect triangles
            let node = model.nodes.get(trs.node).unwrap();
            if let Some(mesh_handle) = node.mesh {
                let mesh = model.meshes.get(mesh_handle).unwrap();
                for prim_handle in mesh.primitives.iter() {
                    let prim = model.primitives.get(*prim_handle).unwrap();
                    let (prim_triangles, prim_triangles_ex, prim_spheres, prim_spheres_ex) =
                        prim.primitives(&ret, Handle::new(i));
                    ret.triangles.extend(prim_triangles);
                    ret.triangles_ex.extend(prim_triangles_ex);
                    ret.spheres.extend(prim_spheres);
                    ret.spheres_ex.extend(prim_spheres_ex);
                }
            }

            // Collect cameras
            if let Some(camera_handle) = node.camera {
                let camera = model.cameras.get(camera_handle).unwrap().clone();
                ret.cameras.push(SolvedCamera::new(camera, trs.trs.clone()));
            }
        }

        ret.set_primitives();

        print_success!(
            "Collected",
            "{} triangles in {:.2}s",
            ret.triangles.len(),
            timer.get_delta().as_secs_f32()
        );

        ret
    }

    pub fn builder<'a>() -> BvhBuilder<'a> {
        BvhBuilder::new()
    }

    pub fn from_primitives(
        max_depth: u8,
        triangles: Vec<Triangle>,
        triangles_ex: Vec<TriangleEx>,
        spheres: Vec<Sphere>,
        spheres_ex: Vec<SphereEx>,
    ) -> Self {
        let mut ret = Self {
            max_depth,
            triangles,
            triangles_ex,
            spheres,
            spheres_ex,
            ..Default::default()
        };
        ret.set_primitives();
        ret
    }

    // TODO: rename in reset primitives
    pub fn set_primitives(&mut self) {
        self.nodes.clear();

        let mut root = BvhNode::new();
        root.bounds = AABB::new(
            Point3::new(f32::MAX, f32::MAX, f32::MAX),
            Point3::new(f32::MIN, f32::MIN, f32::MIN),
        );
        let triangles_range = BvhRange::new(0, self.triangles.len() as u32);
        let spheres_range = BvhRange::new(0, self.spheres.len() as u32);
        root.set_primitives(self, triangles_range, spheres_range);

        self.root = root;
    }

    pub fn get_trs(&self, trs_handle: Handle<SolvedTrs>) -> &Trs {
        if let Some(solved_trs) = self.trss.get(trs_handle) {
            &solved_trs.trs
        } else {
            &Trs::IDENTITY
        }
    }

    pub fn get_shade(&self, primitive: u32) -> &dyn Shade {
        let index = primitive as usize;
        if index < self.triangles_ex.len() {
            &self.triangles_ex[index]
        } else {
            &self.spheres_ex[index - self.triangles_ex.len()]
        }
    }

    pub fn get_sphere(&self, primitive: u32) -> &Sphere {
        &self.spheres[primitive as usize - self.triangles.len()]
    }

    /// Iterative algorithm
    pub fn intersects(&self, ray: &Ray) -> Option<Hit> {
        let mut node = &self.root;
        let mut stack = vec![];

        let mut ret_hit = None;
        let mut max_depth = f32::MAX;

        loop {
            if node.is_leaf() {
                let a = node.triangles.offset;
                let b = a + node.triangles.count;
                for i in a..b {
                    let prim = &self.triangles[i as usize];
                    if let Some(mut hit) = prim.intersects(self, ray) {
                        if hit.depth < max_depth {
                            hit.primitive = i;
                            max_depth = hit.depth;
                            ret_hit = Some(hit);
                        }
                    }
                }

                let a = node.spheres.offset;
                let b = a + node.spheres.count;
                for i in a..b {
                    let prim = &self.spheres[i as usize];
                    if let Some(mut hit) = prim.intersects(self, ray) {
                        if hit.depth < max_depth {
                            hit.primitive = i;
                            max_depth = hit.depth;
                            ret_hit = Some(hit);
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

    pub fn intersects_recursive(&self, ray: &Ray) -> Option<Hit> {
        let mut primitive_count = 0;
        self.root.intersects(
            self,
            ray,
            &self.nodes,
            &self.triangles,
            &self.spheres,
            &mut primitive_count,
        )
    }

    pub fn intersects_stats(&self, ray: &Ray, primitive_count: &mut usize) -> Option<Hit> {
        self.root.intersects(
            self,
            ray,
            &self.nodes,
            &self.triangles,
            &self.spheres,
            primitive_count,
        )
    }
}

#[derive(Default)]
pub struct TlasNodeBuilder {
    bounds: AABB,
    left: Handle<TlasNode>,
    right: Handle<TlasNode>,
    // TODO: make range of handles
    blas: BvhRange,
}

impl TlasNodeBuilder {
    fn _bounds(mut self, bounds: AABB) -> Self {
        self.bounds = bounds;
        self
    }

    fn _blas(mut self, blas: BvhRange) -> Self {
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

    blas: BvhRange,
}

impl TlasNode {
    fn new(bounds: AABB, left: Handle<TlasNode>, right: Handle<TlasNode>, blas: BvhRange) -> Self {
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

    /// Surface Area Heuristics:
    /// The cost of a split is proportional to the summed cost of intersecting the two
    /// resulting boxes, including the triangles they store.
    /// TODO: Refactor this function so that it takes a slice of a generic type
    /// and then try to merge TlasNode and BvhNode into a single structure
    fn evaluate_sah(&self, tlas: &Tlas, axis: Axis3, pos: f32) -> f32 {
        // determine triangle counts and bounds for this split candidate
        let mut left_box = AABB::default();
        let mut right_box = AABB::default();
        let mut left_count = 0;
        let mut right_count = 0;

        let a = self.blas.offset as usize;
        let b = a + self.blas.count as usize;
        for blas_node in tlas.blas_nodes.iter().take(b).skip(a) {
            let bvh = tlas.bvhs.get(blas_node.bvh).unwrap();
            if bvh.root.bounds.centroid()[axis] < pos {
                left_count += 1;
                left_box.grow(&bvh.root.bounds.a);
                left_box.grow(&bvh.root.bounds.b);
            } else {
                right_count += 1;
                right_box.grow(&bvh.root.bounds.a);
                right_box.grow(&bvh.root.bounds.b);
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
    fn find_best_split_plane(&self, tlas: &Tlas) -> (Axis3, f32, f32) {
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
            const AREA_COUNT: i32 = 32;
            let scale = (bounds_max - bounds_min) / AREA_COUNT as f32;

            for i in 1..AREA_COUNT {
                let candidate_pos = bounds_min + i as f32 * scale;
                let cost = self.evaluate_sah(tlas, axis, candidate_pos);
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
        self.blas.count as f32 * self.bounds.area()
    }

    fn replace_models_recursive(&mut self, blas_range: BvhRange, tlas: &mut Tlas) {
        assert!(!blas_range.is_empty());
        self.blas = blas_range;

        self.bounds.a = Point3::new(f32::MAX, f32::MAX, f32::MAX);
        self.bounds.b = Point3::new(f32::MIN, f32::MIN, f32::MIN);

        // Visit each BLAS to find the lowest and highest x, y, and z
        let a = blas_range.offset;
        let b = a + blas_range.count;
        for i in a..b {
            let blas_node = &tlas.blas_nodes[i as usize];
            let blas = tlas.bvhs.get(blas_node.bvh).unwrap();
            self.bounds.a = self.bounds.a.min(&blas.root.bounds.a);
            self.bounds.b = self.bounds.b.max(&blas.root.bounds.b);
        }

        // Surface Area Heuristics
        let (split_axis, split_pos, split_cost) = self.find_best_split_plane(tlas);

        let no_split_cost = self.calculate_cost();
        if split_cost > no_split_cost {
            return;
        }

        // Partition-in-place to obtain two groups of models on both sides of the split plane
        let mut i = blas_range.offset as usize;
        let mut j = (blas_range.offset + blas_range.count) as usize;
        while i < j {
            let blas_node = &tlas.blas_nodes[i];
            let bvh = tlas.bvhs.get(blas_node.bvh).unwrap();
            if bvh.root.bounds.centroid()[split_axis] < split_pos {
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
}

/// The idea of a BLAS node is that we can reorder them in their vector container
/// while keeping the BVH and its model in their place.
pub struct BlasNode {
    pub bvh: Handle<Bvh>,
    pub model: Handle<GltfModel>,
}

impl BlasNode {
    pub fn new(bvh: Handle<Bvh>, model: Handle<GltfModel>) -> Self {
        Self { bvh, model }
    }
}

/// Top Level Acceleration Structure
#[derive(Default)]
pub struct Tlas {
    /// Max depth to use for BVHs
    pub max_depth: u8,

    pub root: TlasNode,

    /// TODO: Use index 0 as root, ignore 1, and children from 2 onwards
    pub tlas_nodes: Pack<TlasNode>,

    /// BLAS nodes are referenced by TLAS nodes
    pub blas_nodes: Vec<BlasNode>,

    /// Bounding Volume Hierarchies should have a 1-1 mapping with model geometries
    /// Bottom Level Acceleration Structures refer a BVH and a Model
    /// TODO: Change this reference from Model to Materials?
    pub bvhs: Pack<Bvh>,
}

impl Tlas {
    pub fn clear(&mut self) {
        self.root = TlasNode::default();
        self.tlas_nodes.clear();
        self.blas_nodes.clear();
        self.bvhs.clear();
    }

    pub fn replace_models(&mut self, models: &[GltfModel]) {
        self.clear();

        for (i, model) in models.iter().enumerate() {
            let bvh_handle = self.bvhs.push(Bvh::new(self.max_depth, model));
            self.blas_nodes
                .push(BlasNode::new(bvh_handle, Handle::new(i)))
        }

        // TODO: make a function which returns a range of handles from a pack
        let mut root = TlasNode::default();
        let range = BvhRange::new(0, models.len() as u32);
        root.replace_models_recursive(range, self);
        self.root = root;
    }

    pub fn intersects(&self, ray: &Ray) -> Option<Hit> {
        // Iterative algorithm
        assert!(self.root.blas.count > 0 || self.root.left.is_some() || self.root.right.is_some());

        let mut node = &self.root;
        let mut stack = vec![];

        let mut ret_hit = None;
        let mut max_depth = f32::MAX;

        loop {
            if node.is_leaf() {
                let a = node.blas.offset;
                let b = a + node.blas.count;
                for i in a..b {
                    let blas_node = &self.blas_nodes[i as usize];
                    let bvh = self.bvhs.get(blas_node.bvh).unwrap();
                    if let Some(mut hit) = bvh.intersects(ray) {
                        if hit.depth < max_depth {
                            hit.blas = i;
                            max_depth = hit.depth;
                            ret_hit = Some(hit);
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

            let mut child1 = self.tlas_nodes.get(node.left).unwrap();
            let mut child2 = self.tlas_nodes.get(node.right).unwrap();
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
}

#[cfg(test)]
mod test {

    use crate::*;

    #[test]
    fn simple() {
        let mut model = GltfModel::default();
        let primitive = model.primitives.push(GltfPrimitive::unit_triangle());
        let mesh = model.meshes.push(GltfMesh::new(vec![primitive]));
        let node = model.nodes.push(Node::builder().mesh(mesh).build());
        model.root.children.push(node);

        let bvh = Bvh::builder().model(&model).build();
        assert!(bvh.nodes.is_empty());
        assert!(bvh.root.left.is_none());
        assert!(bvh.root.right.is_none());
        assert_ne!(bvh.root.triangles.count, 0);
    }

    #[test]
    fn two_children() {
        let mut model = GltfModel::default();
        let left_triangle_prim = GltfPrimitive::builder()
            .vertices(vec![
                GltfVertex::from_position(Point3::new(-4.0, 0.0, 0.0)),
                GltfVertex::from_position(Point3::new(-2.0, 0.0, 0.0)),
                GltfVertex::from_position(Point3::new(-3.0, 0.3, 0.0)),
            ])
            .indices(vec![0, 1, 2])
            .build();
        let right_triangle_prim = GltfPrimitive::unit_triangle();

        let left_primitive = model.primitives.push(left_triangle_prim);
        let right_primitive = model.primitives.push(right_triangle_prim);
        let mesh = GltfMesh::new(vec![left_primitive, right_primitive]);
        let mesh = model.meshes.push(mesh);
        let node = model.nodes.push(Node::builder().mesh(mesh).build());
        model.root.children.push(node);

        let bvh = Bvh::builder().model(&model).build();
        assert!(!bvh.nodes.is_empty());
        assert!(bvh.root.left.is_some());
        assert!(bvh.root.right.is_some());
        assert_eq!(bvh.root.triangles.count, 0);
    }
}
