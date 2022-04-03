// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use rayca::*;

#[test]
fn circle() {
    let mut image = Image::new(128, 128);
    let mut scene = Scene::new();
    let sphere = Sphere::new(Point3::new(0.0, 0.0, -1.0), 1.0);
    let sphere_ex = SphereEx::new(RGBA8::from(0xFF0000FFu32));
    scene.spheres.push(sphere);
    scene.spheres_ex.push(sphere_ex);
    scene.draw(&mut image);
    image.dump_png("target/sphere.png");
}

#[test]
fn triangle() {
    let mut image = Image::new(128, 128);
    let mut scene = Scene::new();
    let triangle = Triangle::new(
        Point3::new(-1.0, -1.0, -1.0),
        Point3::new(1.0, -1.0, -1.0),
        Point3::new(0.0, 1.0, -1.0),
    );
    let mut triangle_ex = TriangleEx::default();
    triangle_ex.vertices[0].color = Color::from(0xFF0000FF);
    triangle_ex.vertices[1].color = Color::from(0x00FF00FF);
    triangle_ex.vertices[2].color = Color::from(0x0000FFFF);
    triangle_ex.vertices[0].normal = Vec3::new(0.0, 0.0, 1.0);
    triangle_ex.vertices[1].normal = Vec3::new(1.0, 0.0, 0.0);
    triangle_ex.vertices[2].normal = Vec3::new(0.0, 1.0, 0.0);
    scene.triangles.push(triangle);
    scene.triangles_ex.push(triangle_ex);
    scene.draw(&mut image);
    image.dump_png("target/triangle.png");
}

#[test]
fn gltf_box() {
    let mut image = Image::new(128, 128);
    let mut scene = Scene::new();
    scene.gltf_model = GltfModel::load("tests/model/box/box.gltf").unwrap();
    scene.draw(&mut image);
    image.dump_png("target/gltf-box.png");
}

#[test]
fn gltf_triangle() {
    let mut image = Image::new(128, 128);
    let mut scene = Scene::new();
    scene.gltf_model = GltfModel::load("tests/model/triangle/triangle.gltf").unwrap();
    scene.draw(&mut image);
    image.dump_png("target/gltf-triangle.png");
}

#[test]
fn gltf_suzanne() {
    let mut image = Image::new(128, 128);
    let mut scene = Scene::new();

    let mut timer = Timer::new();
    scene.gltf_model = GltfModel::load("tests/model/suzanne/suzanne.gltf").unwrap();
    println!("Scene loaded in {}ms", timer.get_delta().as_millis());

    scene.draw(&mut image);
    println!("Scene rendered in {}ms", timer.get_delta().as_millis());

    image.dump_png("target/gltf-suzanne.png");
}

#[test]
fn gltf_cameras() {
    let mut image = Image::new(256, 256);
    let mut scene = Scene::new();
    scene.gltf_model = GltfModel::load("tests/model/cameras/cameras.gltf").unwrap();
    scene.draw(&mut image);
    image.dump_png("target/gltf-cameras.png");
}
