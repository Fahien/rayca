// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

struct Vertex {
    pos: vec4<f32>,
    
    uv: vec2<f32>,
    color: vec4<f32>,

    normal: vec4<f32>,
    tangent: vec4<f32>,
    bitangent: vec4<f32>,
}

struct Triangle {
    pos: array<vec3<f32>,3>,
    centroid: vec4<f32>,
}

struct Hit {
    depth: f32,
    uv: vec2<f32>,
    primitive: u32,
};

struct Ray {
    origin: vec4<f32>,
    dir: vec4<f32>,
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

fn intersect_triangle(ray: ptr<function, Ray>, tri_index: u32, primitive: u32) {
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
        (*ray).hit.primitive = primitive;
    }
}

fn trace(ray: ptr<function, Ray>) -> vec4<f32> {
    for (var i: u32 = 0u; i < tri_count; i++) {
        intersect_triangle(ray, i, 0u);
    }
    if (*ray).hit.depth < 1.0e30 {
        return vec4(1.0, 1.0, 1.0, 1.0);
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

    var xx = (2.0 * ((f32(x) + offset) * inv_width) - 1.0)
                        * camera.angle
                        * aspect_ratio;
    var yy = (1.0 - 2.0 * ((f32(y) + offset) * inv_height)) * camera.angle;
    // Vectors have w = 0, which effectively ignores translation
    var ray_dir = camera.transform * vec4(xx, yy, -1.0, 0.0);

    return Ray(ray_origin, ray_dir, Hit(1.0e30, vec2(0.0, 0.0), 0u));
}

@compute
@workgroup_size(1)
fn render(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var ray = create_ray(global_id);
    var color = trace(&ray);

    storage_b[global_id.x + global_id.y * size.y] = rgba32f_to_rgba8(color);
}
