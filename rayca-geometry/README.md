# Rayca Geometry

A Rust crate providing geometric primitives and utilities for ray tracing and computer graphics. It is part of the Rayca project and is designed to be modular, efficient, and easy to use in rendering engines or scientific applications.

## Features

- **Geometric primitives:**
  - `Sphere` with intersection, normal, and bounding box methods
  - `Triangle` with intersection, centroid, and bounding box methods
  - `Vertex` and `VertexExt` for mesh representation
- **Builder patterns** for ergonomic construction of primitives

## Getting Started

Add to your `Cargo.toml`:

```toml
[dependencies]
rayca-geometry = { git = "https://github.com/Fahien/rayca" }
```

Enable the nightly Rust toolchain for SIMD support:

```sh
rustup override set nightly
```

## Example

### Creating and Intersecting a Sphere
```rust
use rayca_geometry::*;
use rayca_math::*;

let sphere = Sphere::new(Point3::new(0.0, 0.0, 0.0), 1.0);
let ray = Ray::new(Point3::new(0.0, 0.0, -3.0), Vec3::new(0.0, 0.0, 1.0));
let hit = sphere.intersects(&Trs::IDENTITY, &ray);
if let Some(hit) = hit {
    println!("Ray hit the sphere at point: {:?}", hit.point);
}
```

### Creating and Intersecting a Triangle
```rust
use rayca_geometry::*;
use rayca_math::*;

let triangle = Triangle::builder()
    .a(Point3::new(-1.0, 0.0, 0.0))
    .b(Point3::new(1.0, 0.0, 0.0))
    .c(Point3::new(0.0, 1.0, 0.0))
    .build();
let ray = Ray::new(Point3::new(0.0, 0.5, -1.0), Vec3::new(0.0, 0.0, 1.0));
if let Some(hit) = triangle.intersects(&Trs::IDENTITY, &ray) {
    println!("Ray hit the triangle at point: {:?}", hit.point);
}
```

### Using Vertex and VertexExt
```rust
use rayca_geometry::*;
use rayca_math::*;

let vertex = Vertex::builder()
    .position(Point3::new(1.0, 2.0, 3.0))
    .color(Color::WHITE)
    .normal(Vec3::Z_AXIS)
    .build();
println!("Vertex position: {:?}, color: {:?}", vertex.pos, vertex.ext.color);
```
