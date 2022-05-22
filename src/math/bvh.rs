// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use super::*;
use crate::*;

#[derive(Default)]
pub struct AABB {
    pub a: Vec3,
    pub b: Vec3,
}

impl AABB {
    pub fn new(a: Vec3, b: Vec3) -> Self {
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

pub struct BvhNode<'m> {
    bounds: AABB,

    left: Option<Handle<BvhNode<'m>>>,
    right: Option<Handle<BvhNode<'m>>>,

    triangles: Vec<Triangle<'m>>,
}

impl<'m> BvhNode<'m> {
    pub fn new() -> Self {
        Self {
            bounds: Default::default(),
            left: None,
            right: None,
            triangles: vec![],
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }

    pub fn set_triangles(&mut self, triangles: Vec<Triangle<'m>>, nodes: &mut Pack<BvhNode<'m>>) {
        let mut timer = Timer::new();
        self.set_triangles_recursive(triangles, nodes);
        rlog!(
            "{:>12} in {:.2}ms",
            "BHV built".green().bold(),
            timer.get_delta().as_millis()
        );
    }

    fn set_triangles_recursive(
        &mut self,
        triangles: Vec<Triangle<'m>>,
        nodes: &mut Pack<BvhNode<'m>>,
    ) {
        assert!(!triangles.is_empty());
        self.triangles = triangles;

        self.bounds.a = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
        self.bounds.b = Vec3::new(f32::MIN, f32::MIN, f32::MIN);

        // Visit each vertex of the triangles to find the lowest and highest x, y, and z
        for tri in self.triangles.iter() {
            self.bounds.a = self.bounds.a.min(&tri.min());
            self.bounds.b = self.bounds.b.max(&tri.max());
        }

        // Split AABB along its longest axis
        let extent = self.bounds.b - self.bounds.a;
        let mut axis = Axis::X;
        if extent.y > extent.x {
            axis = Axis::Y;
        }
        if extent.z > extent[axis] {
            axis = Axis::Z
        }
        let split_pos = self.bounds.a[axis] + extent[axis] * 0.5;

        // Partition-in-place to obtain two groups of triangles on both sides of the split plane
        let mut i = 0;
        let mut j = self.triangles.len();
        while i < j {
            if self.triangles[i].centroid[axis] < split_pos {
                i += 1;
            } else {
                self.triangles.swap(i, j - 1);
                j -= 1;
            }
        }

        // Create child nodes for each half
        let left_count = i;
        let right_count = self.triangles.len() - left_count;
        if left_count > 0 && right_count > 0 {
            let right_triangles = self.triangles.split_off(left_count);
            let left_triangles = self.triangles.split_off(0);

            // Create two nodes
            let mut left_child = BvhNode::new();
            left_child.set_triangles_recursive(left_triangles, nodes);

            let mut right_child = BvhNode::new();
            right_child.set_triangles_recursive(right_triangles, nodes);

            self.left = Some(nodes.push(left_child));
            self.right = Some(nodes.push(right_child));
        }
    }

    fn intersects(
        &self,
        ray: &Ray,
        nodes: &'m Pack<BvhNode>,
        triangle_count: &mut usize,
    ) -> Option<(Hit, &'m Triangle)> {
        if !self.bounds.intersects(ray) {
            return None;
        }

        let mut ret = None;

        let mut depth = f32::INFINITY;

        if self.is_leaf() {
            *triangle_count += self.triangles.len();

            for tri in &self.triangles {
                if let Some(hit) = tri.intersects(&ray) {
                    if hit.depth < depth {
                        depth = hit.depth;
                        ret = Some((hit, tri));
                    }
                }
            }
        } else {
            if let Some(left_handle) = self.left {
                let left_node = nodes.get(left_handle).unwrap();
                if let Some((hit, tri)) = left_node.intersects(ray, nodes, triangle_count) {
                    if hit.depth < depth {
                        depth = hit.depth;
                        ret = Some((hit, tri));
                    }
                }
            }

            if let Some(right_handle) = self.right {
                let right_node = nodes.get(right_handle).unwrap();
                if let Some((hit, tri)) = right_node.intersects(ray, nodes, triangle_count) {
                    if hit.depth < depth {
                        ret = Some((hit, tri));
                    }
                }
            }
        }

        return ret;
    }
}

pub struct Bvh<'m> {
    pub root: BvhNode<'m>,
    pub nodes: Pack<BvhNode<'m>>,
    pub triangle_count: usize,
}

impl<'m> Bvh<'m> {
    // TODO: A slice instead of a vec?
    pub fn new(triangles: Vec<Triangle<'m>>) -> Self {
        let mut nodes = Pack::new();

        let mut root = BvhNode::new();
        root.bounds = AABB::new(
            Vec3::new(f32::MAX, f32::MAX, f32::MAX),
            Vec3::new(f32::MIN, f32::MIN, f32::MIN),
        );
        root.set_triangles(triangles, &mut nodes);

        Self {
            root,
            nodes,
            triangle_count: 0,
        }
    }

    pub fn intersects(&self, ray: &Ray) -> Option<(Hit, &Triangle)> {
        let mut triangle_count = 0;
        let ret = self.root.intersects(ray, &self.nodes, &mut triangle_count);
        ret
    }

    pub fn intersects_stats(
        &self,
        ray: &Ray,
        triangle_count: &mut usize,
    ) -> Option<(Hit, &Triangle)> {
        self.root.intersects(ray, &self.nodes, triangle_count)
    }
}

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    fn simple() {
        let model = Model::new();
        let triangle_prim = Primitive::unit_triangle();
        let triangles = triangle_prim.triangles(&Trs::default(), Handle::none(), &model);

        let bvh = Bvh::new(triangles);
        assert!(bvh.nodes.is_empty());
        assert!(bvh.root.left.is_none());
        assert!(bvh.root.right.is_none());
        assert!(!bvh.root.triangles.is_empty());
    }

    #[test]
    fn two_children() {
        let model = Model::new();

        let left_triangle_prim = Primitive::builder()
            .vertices(vec![
                Vertex::new(-4.0, 0.0, 0.0),
                Vertex::new(-2.0, 0.0, 0.0),
                Vertex::new(-3.0, 0.3, 0.0),
            ])
            .indices(vec![0, 1, 2])
            .build();
        let right_triangle_prim = Primitive::unit_triangle();

        let mut left_triangles =
            left_triangle_prim.triangles(&Trs::default(), Handle::none(), &model);
        let mut right_triangles =
            right_triangle_prim.triangles(&Trs::default(), Handle::none(), &model);
        left_triangles.append(&mut right_triangles);
        let triangles = left_triangles;

        let bvh = Bvh::new(triangles);
        assert!(!bvh.nodes.is_empty());
        assert!(bvh.root.left.is_some());
        assert!(bvh.root.right.is_some());
        assert!(bvh.root.triangles.is_empty());
    }
}