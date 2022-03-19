// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use rayca::{png::*, *};

#[test]
fn circle() {
    let mut image = Image::new(1024, 1024);
    let mut scene = Scene::new();
    let sphere = Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.1);
    let sphere_ex = SphereEx::new(RGBA8::from(0xFF0000FFu32));
    scene.spheres.push(sphere);
    scene.spheres_ex.push(sphere_ex);
    scene.draw(&mut image);
    dump(&image, "target/sphere.png");
}

#[test]
fn triangle() {
    let mut image = Image::new(1024, 1024);
    let mut scene = Scene::new();
    let triangle = Triangle::new(
        Point3::new(-0.1, -0.1, -1.0),
        Point3::new(0.1, -0.1, -1.0),
        Point3::new(0.0, 0.1, -1.0),
    );
    let mut triangle_ex = TriangleEx::default();
    triangle_ex.vertices[0].color = Color::from(0xFF0000FF);
    triangle_ex.vertices[1].color = Color::from(0x00FF00FF);
    triangle_ex.vertices[2].color = Color::from(0x0000FFFF);
    scene.triangles.push(triangle);
    scene.triangles_ex.push(triangle_ex);
    scene.draw(&mut image);
    dump(&image, "target/triangle.png");
}

#[test]
fn gltf_box() {
    let mut image = Image::new(256, 256);
    let mut scene = Scene::new();
    scene.gltf_model = GltfModel::load("tests/model/box/box.gltf").unwrap();

    scene.draw(&mut image);
    dump(&image, "target/gltf-box.png");
}
