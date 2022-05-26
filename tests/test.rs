// Copyright © 2022
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
    scene.models.push(model);

    scene.draw(&mut image);
    image.dump_png("target/triangle.png");
}

#[test]
fn cube_over_plane() {
    let mut image = Image::new(1024, 1024, ColorType::RGBA8);
    let mut scene = Scene::new();

    let mut timer = Timer::new();
    scene.load("tests/model/box/box.gltf").unwrap();
    scene.load("tests/model/box/box.gltf").unwrap();
    rlog!("Scene loaded in {}ms", timer.get_delta().as_millis());

    scene.models[0].root.trs.scale = Vec3::new(16.0, 16.0, 0.125);
    scene.models[0].root.trs.translation.y -= 1.0;
    scene.models[0]
        .nodes
        .get_mut(1.into())
        .unwrap()
        .trs
        .rotation = Quat::default();
    scene.models[0].materials.get_mut(0.into()).unwrap().color = Color::new(0.1, 0.2, 0.7, 1.0);

    scene.draw(&mut image);
    image.dump_png("target/cube-over-plane.png");
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
        image.dump_png("target/cube.png");
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
        image.dump_png("target/suzanne.png");
    }

    #[test]
    fn duck() {
        let mut image = Image::new(128, 128, ColorType::RGBA8);
        let mut scene = Scene::new();

        let mut timer = Timer::new();
        scene.load("tests/model/duck/duck.gltf").unwrap();
        rlog!("Scene loaded in {}ms", timer.get_delta().as_millis());

        // Custom camera
        let mut camera_node = Node::builder()
            .id(scene.models[0].nodes.len())
            .translation(Vec3::new(0.1, 0.8, 2.2))
            .build();
        camera_node.camera = Some(scene.models[0].cameras.push(Camera::default()));
        let camera_node_handle = scene.models[0].nodes.push(camera_node);
        scene.models[0].root.children.push(camera_node_handle);

        scene.draw(&mut image);
        image.dump_png("target/duck.png");
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
        let mut image = Image::new(256, 256, ColorType::RGBA8);
        let mut scene = Scene::new();
        scene
            .load("tests/model/orientation/OrientationTest.gltf")
            .unwrap();

        // Custom camera
        let mut camera_node = Node::builder()
            .id(scene.models[0].nodes.len())
            .translation(Vec3::new(0.0, 0.0, 1.0))
            .build();
        camera_node.camera = Some(scene.models[0].cameras.push(Camera::default()));
        let camera_node_handle = scene.models[0].nodes.push(camera_node);
        scene.models[0].root.children.push(camera_node_handle);
        scene.models[0].root.trs.scale = Vec3::new(1.0 / 32.0, 1.0 / 32.0, 1.0 / 32.0);

        scene.draw(&mut image);
        image.dump_png("target/orientation.png");
    }

    #[test]
    fn flight() {
        let mut image = Image::new(64, 64, ColorType::RGBA8);
        let mut scene = Scene::new();

        scene
            .load("tests/model/flight-helmet/FlightHelmet.gltf")
            .unwrap();

        // Custom camera
        let mut camera_node = Node::builder()
            .id(scene.models[0].nodes.len())
            .translation(Vec3::new(0.0, 0.32, 1.0))
            .build();
        camera_node.camera = Some(scene.models[0].cameras.push(Camera::default()));
        let camera_node_handle = scene.models[0].nodes.push(camera_node);
        scene.models[0].root.children.push(camera_node_handle);

        scene.draw(&mut image);
        image.dump_png("target/flight.png");
    }

    #[test]
    fn sponza() {
        let mut image = Image::new(64, 64, ColorType::RGBA8);
        let mut scene = Scene::new();

        scene.load("tests/model/sponza/sponza.gltf").unwrap();

        // Custom camera
        let rotation = Quat::new(0.0, -0.707, 0.0, 0.707);
        scene.models[0]
            .nodes
            .get_mut(0.into())
            .unwrap()
            .trs
            .rotation = rotation;

        let mut camera_node = Node::builder()
            .id(scene.models[0].nodes.len())
            .translation(Vec3::new(0.2, 1.0, 0.0))
            .build();
        camera_node.camera = Some(scene.models[0].cameras.push(Camera::default()));
        let camera_node_handle = scene.models[0].nodes.push(camera_node);
        scene.models[0].root.children.push(camera_node_handle);
        scene.models[0].root.trs.scale = Vec3::new(0.5, 0.5, 0.5);

        scene.draw(&mut image);
        image.dump_png("target/sponza.png");
    }
}
