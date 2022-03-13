// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use rayca::{png::*, *};

#[test]
fn circle() {
    let mut image = Image::new(32, 32);
    let mut scene = Scene::new();
    let sphere = Sphere::new(Vec3::new(0.0, 0.0, -1.0), 0.1, 0xFF0000FFu32);
    scene.objects.push(Box::new(sphere));
    scene.draw(&mut image);
    dump(&image, "target/sphere.png");
}

#[test]
fn triangle() {
    let mut image = Image::new(32, 32);
    let mut scene = Scene::new();
    let triangle = Triangle::new(
        Vec3::new(-0.1, -0.1, -1.0),
        Vec3::new(0.1, -0.1, -1.0),
        Vec3::new(0.0, 0.1, -1.0),
    );
    scene.objects.push(Box::new(triangle));
    scene.draw(&mut image);
    dump(&image, "target/triangle.png");
}
