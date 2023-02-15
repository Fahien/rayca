// Copyright © 2022-2023
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

    pub fn centroid(&self) -> Point3 {
        ((self.b - self.a) / 2.0).into()
    }

    /// Slab test. We do not care where we hit the box; only info we need is a yes/no answer.
    fn intersects(&self, ray: &Ray) -> bool {
        let tx1 = (self.a.x - ray.origin.x) / ray.dir.x;
        let tx2 = (self.b.x - ray.origin.x) / ray.dir.x;
        let tmin = tx1.min(tx2);
        let tmax = tx1.max(tx2);

        let ty1 = (self.a.y - ray.origin.y) / ray.dir.y;
        let ty2 = (self.b.y - ray.origin.y) / ray.dir.y;
        let tmin = tmin.max(ty1.min(ty2));
        let tmax = tmax.min(ty1.max(ty2));

        let tz1 = (self.a.z - ray.origin.z) / ray.dir.z;
        let tz2 = (self.b.z - ray.origin.z) / ray.dir.z;
        let tmin = tmin.max(tz1.min(tz2));
        let tmax = tmax.min(tz1.max(tz2));

        tmax >= tmin && tmax > 0.0
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
}

impl BvhNode {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_leaf(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }

    fn set_triangles(
        &mut self,
        triangles_range: BvhRange,
        triangles: &mut [Triangle],
        triangles_ex: &mut [TriangleEx],
        nodes: &mut Pack<BvhNode>,
    ) {
        let mut timer = Timer::new();
        self.set_triangles_recursive(triangles_range, triangles, triangles_ex, nodes);
        print_success!("Built", "BVH in {:.2}ms", timer.get_delta().as_millis());
    }

    fn set_triangles_recursive(
        &mut self,
        triangles_range: BvhRange,
        triangles: &mut [Triangle],
        triangles_ex: &mut [TriangleEx],
        nodes: &mut Pack<BvhNode>,
    ) {
        assert!(!triangles_range.is_empty());
        self.triangles = triangles_range;

        self.bounds.a = Point3::new(f32::MAX, f32::MAX, f32::MAX);
        self.bounds.b = Point3::new(f32::MIN, f32::MIN, f32::MIN);

        // Visit each vertex of the triangles to find the lowest and highest x, y, and z
        let a = triangles_range.offset;
        let b = a + triangles_range.count;
        for i in a..b {
            let tri = &triangles[i as usize];
            self.bounds.a = self.bounds.a.min(&tri.min());
            self.bounds.b = self.bounds.b.max(&tri.max());
        }

        // Split AABB along its longest axis
        let extent = self.bounds.b - self.bounds.a;
        let mut axis = Axis3::X;
        if extent.y > extent.x {
            axis = Axis3::Y;
        }
        if extent.z > extent[axis] {
            axis = Axis3::Z
        }
        let split_pos = self.bounds.a[axis] + extent[axis] * 0.5;

        // Partition-in-place to obtain two groups of triangles on both sides of the split plane
        let mut i = triangles_range.offset as usize;
        let mut j = (triangles_range.offset + triangles_range.count) as usize;
        while i < j {
            if triangles[i].centroid[axis] < split_pos {
                i += 1;
            } else {
                triangles.swap(i, j - 1);
                triangles_ex.swap(i, j - 1);
                j -= 1;
            }
        }

        // Create child nodes for each half
        let left_count = i as u32 - triangles_range.offset;
        let right_count = triangles_range.count - left_count;
        if left_count > 0 && right_count > 0 {
            let right_triangles_range =
                BvhRange::new(triangles_range.offset + left_count, right_count);
            let left_triangles_range = BvhRange::new(triangles_range.offset, left_count);

            // Create two nodes
            let mut left_child = BvhNode::new();
            left_child.set_triangles_recursive(
                left_triangles_range,
                triangles,
                triangles_ex,
                nodes,
            );

            let mut right_child = BvhNode::new();
            right_child.set_triangles_recursive(
                right_triangles_range,
                triangles,
                triangles_ex,
                nodes,
            );

            self.left = nodes.push(left_child);
            self.right = nodes.push(right_child);
            self.triangles.count = 0;
        }
    }

    fn intersects(
        &self,
        ray: &Ray,
        nodes: &Pack<BvhNode>,
        triangles: &[Triangle],
        triangle_count: &mut usize,
    ) -> Option<Hit> {
        if !self.bounds.intersects(ray) {
            return None;
        }

        let mut ret = None;

        let mut depth = f32::INFINITY;

        if self.is_leaf() {
            *triangle_count += self.triangles.count as usize;

            let a = self.triangles.offset;
            let b = a + self.triangles.count;
            for i in a..b {
                let tri = &triangles[i as usize];
                if let Some(mut hit) = tri.intersects(ray) {
                    if hit.depth < depth {
                        depth = hit.depth;
                        hit.primitive = i;
                        ret = Some(hit);
                    }
                }
            }
        } else {
            if let Some(left_node) = nodes.get(self.left) {
                if let Some(hit) = left_node.intersects(ray, nodes, triangles, triangle_count) {
                    if hit.depth < depth {
                        depth = hit.depth;
                        ret = Some(hit);
                    }
                }
            }

            if let Some(right_node) = nodes.get(self.right) {
                if let Some(hit) = right_node.intersects(ray, nodes, triangles, triangle_count) {
                    if hit.depth < depth {
                        ret = Some(hit);
                    }
                }
            }
        }

        ret
    }
}

#[derive(Default)]
pub struct Bvh {
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

impl From<&GltfModel> for Bvh {
    fn from(model: &GltfModel) -> Self {
        let mut timer = Timer::new();

        let mut ret = Self {
            trss: model.collect_transforms(),
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
                    let (prim_triangles, prim_triangles_ex) = prim.triangles(&trs.trs);
                    ret.triangles.extend(prim_triangles);
                    ret.triangles_ex.extend(prim_triangles_ex);
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
}

impl Bvh {
    pub fn new(triangles: Vec<Triangle>, triangles_ex: Vec<TriangleEx>) -> Self {
        let mut ret = Self {
            triangles,
            triangles_ex,
            ..Default::default()
        };
        ret.set_primitives();
        ret
    }

    // TODO: rename in replace_primitives
    pub fn set_primitives(&mut self) {
        self.nodes.clear();

        let mut root = BvhNode::new();
        root.bounds = AABB::new(
            Point3::new(f32::MAX, f32::MAX, f32::MAX),
            Point3::new(f32::MIN, f32::MIN, f32::MIN),
        );
        let range = BvhRange::new(0, self.triangles.len() as u32);
        root.set_triangles(
            range,
            &mut self.triangles,
            &mut self.triangles_ex,
            &mut self.nodes,
        );

        self.root = root;
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

    pub fn intersects(&self, ray: &Ray) -> Option<Hit> {
        let mut triangle_count = 0;
        self.root
            .intersects(ray, &self.nodes, &self.triangles, &mut triangle_count)
    }

    pub fn intersects_stats(&self, ray: &Ray, triangle_count: &mut usize) -> Option<Hit> {
        self.root
            .intersects(ray, &self.nodes, &self.triangles, triangle_count)
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

        // Split AABB along its longest axis
        let extent = self.bounds.b - self.bounds.a;
        let mut axis = Axis3::X;
        if extent.y > extent.x {
            axis = Axis3::Y;
        }
        if extent.z > extent[axis] {
            axis = Axis3::Z
        }
        let split_pos = self.bounds.a[axis] + extent[axis] * 0.5;

        // Partition-in-place to obtain two groups of models on both sides of the split plane
        let mut i = blas_range.offset as usize;
        let mut j = (blas_range.offset + blas_range.count) as usize;
        while i < j {
            let blas_node = &tlas.blas_nodes[i];
            let bvh = tlas.bvhs.get(blas_node.bvh).unwrap();
            if bvh.root.bounds.centroid()[axis] < split_pos {
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

    fn intersects(&self, ray: &Ray, tlas: &Tlas) -> Option<Hit> {
        if !self.bounds.intersects(ray) {
            return None;
        }

        let mut ret = None;

        let mut depth = f32::INFINITY;

        if self.is_leaf() {
            let a = self.blas.offset;
            let b = a + self.blas.count;
            for i in a..b {
                let blas_node = &tlas.blas_nodes[i as usize];
                let bvh = tlas.bvhs.get(blas_node.bvh)?;
                if let Some(mut hit) = bvh.intersects(ray) {
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
                if let Some(hit) = left_node.intersects(ray, tlas) {
                    if hit.depth < depth {
                        depth = hit.depth;
                        ret = Some(hit);
                    }
                }
            }

            if let Some(right_node) = tlas.tlas_nodes.get(self.right) {
                if let Some(hit) = right_node.intersects(ray, tlas) {
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
            let bvh_handle = self.bvhs.push(Bvh::from(model));
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
        assert!(self.root.blas.count > 0 || self.root.left.is_some() || self.root.right.is_some());
        self.root.intersects(ray, self)
    }
}

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    fn simple() {
        let triangle_prim = GltfPrimitive::unit_triangle();
        let (triangles, triangles_ex) = triangle_prim.triangles(&Trs::default());

        let bvh = Bvh::new(triangles, triangles_ex);
        assert!(bvh.nodes.is_empty());
        assert!(bvh.root.left.is_none());
        assert!(bvh.root.right.is_none());
        assert_ne!(bvh.root.triangles.count, 0);
    }

    #[test]
    fn two_children() {
        let left_triangle_prim = GltfPrimitive::builder()
            .vertices(vec![
                GltfVertex::from_position(Point3::new(-4.0, 0.0, 0.0)),
                GltfVertex::from_position(Point3::new(-2.0, 0.0, 0.0)),
                GltfVertex::from_position(Point3::new(-3.0, 0.3, 0.0)),
            ])
            .indices(vec![0, 1, 2])
            .build();
        let right_triangle_prim = GltfPrimitive::unit_triangle();

        let (mut left_triangles, mut left_triangles_ex) =
            left_triangle_prim.triangles(&Trs::default());
        let (mut right_triangles, mut right_triangles_ex) =
            right_triangle_prim.triangles(&Trs::default());
        left_triangles.append(&mut right_triangles);
        left_triangles_ex.append(&mut right_triangles_ex);

        let bvh = Bvh::new(left_triangles, left_triangles_ex);
        assert!(!bvh.nodes.is_empty());
        assert!(bvh.root.left.is_some());
        assert!(bvh.root.right.is_some());
        assert_eq!(bvh.root.triangles.count, 0);
    }
}
