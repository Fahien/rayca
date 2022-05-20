// Copyright © 2022-2024
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use rayca::*;

#[test]
fn triangle() {
    let mut image = Image::new(256, 256, ColorType::RGBA8);
    let mut scene = Scene::new();

    let mut model = Model::new();
    let mut prim = Primitive::unit_triangle();
    prim.vertices[0].color = Color::from(0xFF0000FF);
    prim.vertices[1].color = Color::from(0x00FF00FF);
    prim.vertices[2].color = Color::from(0x0000FFFF);
    let prim_handle = model.primitives.push(prim);
    let mesh = Mesh::new(vec![prim_handle]);
    let mesh_handle = model.meshes.push(mesh);
    let node = Node::builder()
        .mesh(mesh_handle)
        .translation(Vec3::new(0.0, -1.0, 0.0))
        .scale(Vec3::new(1.0, 2.0, 1.0))
        .build();
    let node_handle = model.nodes.push(node);
    model.root.children.push(node_handle);
    scene.push(model);

    scene.draw(&mut image);
    image.dump_png("target/triangle.png");
}

mod gltf {
    use super::*;

    #[test]
    fn cube() {
        let mut image = Image::new(256, 256, ColorType::RGBA8);
        let mut scene = Scene::new();

        let mut timer = Timer::new();
        scene.load("tests/model/box/box.gltf").unwrap();
        rlog!("Scene loaded in {}ms", timer.get_delta().as_millis());

        scene.draw(&mut image);
        rlog!("Scene rendered in {}ms", timer.get_delta().as_millis());

        image.dump_png("target/gltf-cube.png");
    }

    #[test]
    fn triangle() {
        let mut image = Image::new(128, 128, ColorType::RGBA8);
        let mut scene = Scene::new();
        scene.load("tests/model/triangle/triangle.gltf").unwrap();

        scene.draw(&mut image);
        image.dump_png("target/gltf-triangle.png");
    }

    #[test]
    fn suzanne() {
        let mut image = Image::new(128, 128, ColorType::RGBA8);
        let mut scene = Scene::new();

        let mut timer = Timer::new();
        scene.load("tests/model/suzanne/suzanne.gltf").unwrap();
        rlog!("Scene loaded in {}ms", timer.get_delta().as_millis());

        scene.draw(&mut image);
        rlog!("Scene rendered in {}ms", timer.get_delta().as_millis());

        image.dump_png("target/suzanne.png");
    }

    #[test]
    fn cameras() {
        let mut image = Image::new(256, 256, ColorType::RGBA8);
        let mut scene = Scene::new();
        scene.load("tests/model/cameras/cameras.gltf").unwrap();
        scene.draw(&mut image);
        image.dump_png("target/cameras.png");
    }

    #[test]
    fn orientation() {
        let mut image = Image::new(1024, 1024, ColorType::RGBA8);
        let mut scene = Scene::new();
        scene
            .load("tests/model/orientation/OrientationTest.gltf")
            .unwrap();

        // Custom camera
        let mut camera_node = Node::builder()
            .id(scene.model.nodes.len())
            .translation(Vec3::new(0.0, 0.32, 24.0))
            .build();
        camera_node.camera = scene.model.cameras.push(Camera::default());
        let camera_node_handle = scene.model.nodes.push(camera_node);
        scene.model.root.children.push(camera_node_handle);

        scene.draw(&mut image);
        image.dump_png("target/orientation.png");
    }

    #[test]
    fn flight() {
        let mut image = Image::new(32, 32, ColorType::RGBA8);
        let mut scene = Scene::new();

        scene
            .load("tests/model/flight-helmet/FlightHelmet.gltf")
            .unwrap();

        // Custom camera
        let mut camera_node = Node::builder()
            .id(scene.model.nodes.len())
            .translation(Vec3::new(0.0, 0.32, 1.0))
            .build();
        camera_node.camera = scene.model.cameras.push(Camera::default());
        let camera_node_handle = scene.model.nodes.push(camera_node);
        scene.model.root.children.push(camera_node_handle);

        scene.draw(&mut image);
        image.dump_png("target/flight.png");
    }
}
