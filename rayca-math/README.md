# Rayca Math

SIMD-accelerated math library for 2D and 3D graphics, geometry, and linear algebra, written in Rust. It provides types and operations for vectors, points, matrices, quaternions, colors, and geometric transformations, designed for use in rendering engines, simulations, and games.

## Features

- SIMD-accelerated `Vec3`, `Point3`, `Mat4`, `Quat`
- Quaternion (`Quat`) support for 3D rotations
- Color types: `Color`, `RGB8`, `RGBA8`, `RGBA32F`
- Transformations: translation, rotation, scaling, and composition (`Trs`)
- Ray structure for ray tracing and intersection tests
- Operator overloading for intuitive math expressions
- Conversion between types and color formats

## Getting Started

Add to your `Cargo.toml`:

```toml
[dependencies]
rayca-math = { git = "https://github.com/Fahien/rayca" }
```

Enable the nightly Rust toolchain for SIMD support:

```sh
rustup override set nightly
```

## Examples

### Vectors

```rust
use rayca_math::Vec3;

let a = Vec3::new(1.0, 2.0, 3.0);
let b = Vec3::new(4.0, 5.0, 6.0);

let sum = a + b;
let diff = a - b;
let scaled = a * 2.0;
let dot = a.dot(&b);
let cross = a.cross(&b);
let normalized = a.get_normalized();
```

### Points

```rust
use rayca_math::Point3;

let p = Point3::new(1.0, 2.0, 3.0);
let v = rayca_math::Vec3::new(0.5, 0.5, 0.5);

let moved = p + v;
let displacement = moved - p; // yields Vec3
```

### Quaternions

```rust
use rayca_math::{Quat, Vec3};
use std::f32::consts::PI;

let axis = Vec3::new(0.0, 1.0, 0.0);
let angle = PI / 2.0;
let q = Quat::axis_angle(axis, angle);

let v = Vec3::new(1.0, 0.0, 0.0);
let rotated = q * v;
```

### Matrices

```rust
use rayca_math::{Mat4, Vec3, Quat};

let scale = Vec3::new(2.0, 2.0, 2.0);
let rotation = Quat::axis_angle(Vec3::new(0.0, 1.0, 0.0), std::f32::consts::FRAC_PI_2);
let translation = Vec3::new(1.0, 0.0, 0.0);

let mut mat = Mat4::identity();
mat.scale(&scale);
mat.rotate(&rotation);
mat.translate(&translation);

let transformed = mat * Vec3::new(1.0, 0.0, 0.0);
```

### Transformations (`Trs`)

```rust
use rayca_math::{Trs, Vec3, Quat, Point3};

let trs = Trs::builder()
    .translation(Vec3::new(1.0, 2.0, 3.0))
    .rotation(Quat::axis_angle(Vec3::new(0.0, 1.0, 0.0), std::f32::consts::PI))
    .scale(Vec3::new(2.0, 2.0, 2.0))
    .build();

let point = Point3::new(1.0, 0.0, 0.0);
let transformed = &trs * point;
```

### Rays

```rust
use rayca_math::{Ray, Point3, Vec3};

let origin = Point3::new(0.0, 0.0, 0.0);
let dir = Vec3::new(0.0, 0.0, -1.0);
let mut ray = Ray::new(origin, dir);

let scale = Vec3::new(2.0, 2.0, 2.0);
ray.scale(&scale);
```

### Colors

```rust
use rayca_math::{Color, RGBA8, RGB8, Vec3};

let color = Color::new(0.5, 0.2, 0.8, 1.0);
let rgba8: RGBA8 = color.into();
let rgb8: RGB8 = Vec3::new(1.0, 0.0, 0.0).into();
```
