// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

struct AABB {
    a: vec4<f32>,
    b: vec4<f32>,
}

struct BvhRange {
    offset: u32,
    count: u32,
}

/// If the node has primitives it means it is a leaf.
/// Left node index is `triangles.offset`, when the node has no primitives.
/// Right node index is just `left_node_index + 1`.
struct BvhNode {
    bounds: AABB,
    primitives: BvhRange,
}

struct Material {
    color: vec4<f32>,
    albedo_texture: u32,
    normal_texture: u32,
    metallic_factor: f32,
    roughness_factor: f32,
    metallic_roughness_texture: u32,
}

struct VertexExt {
    color: vec4<f32>,
    normal: vec4<f32>,
    tangent: vec4<f32>,
    bitangent: vec4<f32>,
    uv: vec2<f32>,
}

struct Triangle {
    pos: array<vec3<f32>,3>,
    centroid: vec4<f32>,
}

struct TriangleExt {
    vertices: array<VertexExt,3>,
    material: u32,
}

struct Hit {
    depth: f32,
    uv: vec2<f32>,
    primitive: u32,
};

struct Ray {
    origin: vec4<f32>,
    dir: vec3<f32>,
    rdir: vec3<f32>,
    hit: Hit,
};

struct Camera {
    transform: mat4x4<f32>,
    angle: f32,
}

// The values of this group do not change often, ideally only when resizing the window
@group(0)
@binding(0)
var<storage, read_write> storage_b: array<u32>;
@group(0)
@binding(1)
var<uniform> size: vec2<u32>;

// This group values can change every frame
@group(1)
@binding(0)
var<uniform> camera: Camera;
@group(1)
@binding(1)
var<uniform> tri_count: u32;
@group(1)
@binding(2)
var<storage, read> tri: array<Triangle>;
@group(1)
@binding(3)
var<storage, read> tri_ext: array<TriangleExt>;
@group(1)
@binding(4)
var<storage, read> materials: array<Material>;
@group(1)
@binding(5)
var<uniform> node_count: u32;
@group(1)
@binding(6)
var<storage, read> nodes: array<BvhNode>;


fn intersect_aabb( ray: ptr<function, Ray>, bounds: AABB) -> f32 {
    let ray_ori = (*ray).origin.xyz;
    let ray_rdir = (*ray).rdir.xyz;

    let tx1 = (bounds.a.x - ray_ori.x) * ray_rdir.x;
    let tx2 = (bounds.b.x - ray_ori.x) * ray_rdir.x;
    var tmin = min(tx1, tx2);
    var tmax = max(tx1, tx2);
    let ty1 = (bounds.a.y - ray_ori.y) * ray_rdir.y;
    let ty2 = (bounds.b.y - ray_ori.y) * ray_rdir.y;
    tmin = max(tmin, min(ty1, ty2));
    tmax = min(tmax, max(ty1, ty2));
    let tz1 = (bounds.a.z - ray_ori.z) * ray_rdir.z;
    let tz2 = (bounds.b.z - ray_ori.z) * ray_rdir.z;
    tmin = max(tmin, min(tz1, tz2));
    tmax = min(tmax, max(tz1, tz2));
    if tmax >= tmin && tmin < (*ray).hit.depth && tmax > 0.0 {
        return tmin;
    } else {
        return 1.0e30;
    }
}

fn intersect_triangle(ray: ptr<function, Ray>, tri_index: u32) {
    let ray_ori = (*ray).origin.xyz;
    let ray_dir = (*ray).dir.xyz;

    let v0 = tri[tri_index].pos[0].xyz;
    let v1 = tri[tri_index].pos[1].xyz;
    let v2 = tri[tri_index].pos[2].xyz;

    var edge1 = v1 - v0;
    var edge2 = v2 - v0;

    var h = cross(ray_dir, edge2);
    var a = dot(edge1, h);
    if a > -0.00001 && a < 0.00001 {
        return;
    }
    var f = 1.0 / a;
    var s = ray_ori - v0;
    var u = f * dot(s, h);
    if u < 0.0 || u > 1.0 {
        return;
    }
    var q = cross(s, edge1);
    var v = f * dot(ray_dir, q);
    if v < 0.0 || (u + v > 1.0) {
        return;
    }
    var t = f * dot(edge2, q);
    if t > 0.0001 && t < (*ray).hit.depth {
        (*ray).hit.depth = t;
        (*ray).hit.uv.x = u;
        (*ray).hit.uv.y = v;
        (*ray).hit.primitive = tri_index;
    }
}

fn intersect_bvh(ray: ptr<function, Ray>) {
    var node_index: u32 = 0u;
    var stack: array<u32, 32u>;
    var stack_ptr: u32 = 0u;
    while true {
        if node_index >= node_count {
            return;
        }

        let node = &nodes[node_index];
        if (*node).primitives.count > 0u { // is_leaf()
            for (var i: u32 = 0u; i < (*node).primitives.count; i++) {
                let tri_index = i + (*node).primitives.offset;
                intersect_triangle(ray, tri_index);
            }

            if stack_ptr == 0u {
                break;
            } else {
                stack_ptr--;
                node_index = stack[stack_ptr];
            }

            continue;
        }

        var child1_index: u32 = (*node).primitives.offset;
        let child1 = &nodes[child1_index];

        var child2_index: u32 = child1_index + 1u;
        let child2 = &nodes[child2_index];

        var dist1: f32 = intersect_aabb(ray, (*child1).bounds);
        var dist2: f32 = intersect_aabb(ray, (*child2).bounds);
        if dist1 > dist2 {
            var d: f32 = dist1;
            dist1 = dist2;
            dist2 = d;
            var c_index: u32 = child1_index;
            child1_index = child2_index;
            child2_index = c_index;
        }
        if dist1 == 1.0e30 {
            if stack_ptr == 0u {
                break;
            } else {
                stack_ptr--;
                node_index = stack[stack_ptr];
            }
        } else {
            node_index = child1_index;
            if dist2 != 1.0e30 {
                stack[stack_ptr] = child2_index;
                stack_ptr++;
            }
        }
    }
}

fn get_color(primitive_index: u32) -> vec4<f32> {
    let material_index = tri_ext[primitive_index].material;
    let material = materials[material_index];
    return material.color;
}

fn trace(ray: ptr<function, Ray>) -> vec4<f32> {
    intersect_bvh(ray);
    let d = (*ray).hit.depth;
    if d < 1.0e30 {
        return get_color((*ray).hit.primitive) / (d / 8.0);
    } else {
        return vec4(0.0, 0.0, 0.0, 1.0);
    }
}

fn rgba32f_to_rgba8(c: vec4<f32>) -> u32 {
    let a = u32(min(c.x, 1.0) * 255.0);
    let b = u32(min(c.y, 1.0) * 255.0);
    let g = u32(min(c.z, 1.0) * 255.0);
    let r = u32(min(c.w, 1.0) * 255.0);
    return (r << 24u) + (g << 16u) + (b << 8u) + a;
}

fn create_ray(global_id: vec3<u32>) -> Ray {
    var x: u32 = global_id.x;
    var y: u32 = global_id.y;

    // Ray origin is a point
    // In WGLS, matrices are column-major, meaning that matrix[0][1..4] is a column
    // And translation should be matrix[3].xyz
    var ray_origin = camera.transform * vec4(0.0, 0.0, 0.0, 1.0);

    // This is a vector
    var offset = 0.5;
    var width = f32(size.x);
    var height = f32(size.y);
    var inv_width = 1.0 / width;
    var inv_height = 1.0 / height;
    var aspect_ratio = width / height;

    var xx = (2.0 * ((f32(x) + offset) * inv_width) - 1.0) * camera.angle * aspect_ratio;
    var yy = (1.0 - 2.0 * ((f32(y) + offset) * inv_height)) * camera.angle;
    // Vectors have w = 0, which effectively ignores translation
    var ray_dir = (camera.transform * vec4(xx, yy, -1.0, 0.0)).xyz;
    let ray_rdir = vec3(1.0) / ray_dir;

    return Ray(ray_origin, ray_dir, ray_rdir, Hit(1.0e30, vec2(0.0, 0.0), 0u));
}

@compute
@workgroup_size(1)
fn render(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var ray = create_ray(global_id);
    var color = trace(&ray);

    storage_b[global_id.x + global_id.y * size.y] = rgba32f_to_rgba8(color);
}
