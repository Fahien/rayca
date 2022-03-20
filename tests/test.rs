// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use rayca::{png::*, *};

#[test]
fn circle() {
    let mut image = Image::new(1024, 1024);
    let mut scene = Scene::new();
    let sphere = Sphere::new(Vec3::new(0.0, 0.0, -1.0), 0.1, RGBA8::from(0xFF0000FFu32));
    scene.objects.push(Box::new(sphere));
    scene.draw(&mut image);
    dump(&image, "target/sphere.png");
}

#[test]
fn triangle() {
    let mut image = Image::new(1024, 1024);
    let mut scene = Scene::new();
    let mut triangle = Triangle::new(
        Vertex::new(-0.1, -0.1, -1.0),
        Vertex::new(0.1, -0.1, -1.0),
        Vertex::new(0.0, 0.1, -1.0),
    );
    triangle.vertices[0].color = RGBA8::from(0xFF0000FF);
    triangle.vertices[1].color = RGBA8::from(0x00FF00FF);
    triangle.vertices[2].color = RGBA8::from(0x0000FFFF);

    triangle.vertices[0].normal = Vec3::new(0.0, 0.0, 1.0);
    triangle.vertices[1].normal = Vec3::new(1.0, 0.0, 0.0);
    triangle.vertices[2].normal = Vec3::new(0.0, 1.0, 0.0);

    scene.objects.push(Box::new(triangle));
    scene.draw(&mut image);
    dump(&image, "target/triangle.png");
}

mod gltf {
    use super::*;

    #[test]
    fn cube() {
        let mut image = Image::new(1024, 1024);
        let mut scene = Scene::new();
        scene.load("tests/model/box/box.gltf").unwrap();

        scene.draw(&mut image);
        dump(&image, "target/cube.png");
    }

    #[test]
    fn triangle() {
        let mut image = Image::new(256, 256);
        let mut scene = Scene::new();
        scene.load("tests/model/triangle/triangle.gltf").unwrap();

        scene.draw(&mut image);
        dump(&image, "target/gltf-triangle.png");
    }
}
