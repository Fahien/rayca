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

pub struct Bvh {
    pub root: BvhNode,
    pub nodes: Pack<BvhNode>,
    pub triangle_count: usize,

    pub triangles: Vec<Triangle>,
    pub triangles_ex: Vec<TriangleEx>,

    pub spheres: Vec<Sphere>,
    pub spheres_ex: Vec<SphereEx>,
}

impl Default for Bvh {
    fn default() -> Self {
        Self::new(vec![], vec![])
    }
}

impl Bvh {
    pub fn new(mut triangles: Vec<Triangle>, mut triangles_ex: Vec<TriangleEx>) -> Self {
        let mut nodes = Pack::new();

        let mut root = BvhNode::new();
        root.bounds = AABB::new(
            Point3::new(f32::MAX, f32::MAX, f32::MAX),
            Point3::new(f32::MIN, f32::MIN, f32::MIN),
        );
        let range = BvhRange::new(0, triangles.len() as u32);
        root.set_triangles(range, &mut triangles, &mut triangles_ex, &mut nodes);

        Self {
            root,
            nodes,
            triangle_count: 0,
            triangles,
            triangles_ex,
            spheres: Default::default(),
            spheres_ex: Default::default(),
        }
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
