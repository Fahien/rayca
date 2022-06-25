// Copyright © 2022-2024
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use rayca::*;

#[test]
fn sphere() {
    let mut image = Image::new(256, 256, ColorType::RGBA8);
    let mut scene = Scene::new_with_config(Config::new(false, Box::new(Scratcher::new())));

    let mut model = Model::new();
    let prim = Primitive::unit_sphere();
    let prim_handle = model.primitives.push(prim);
    let mesh = Mesh::new(vec![prim_handle]);
    let mesh_handle = model.meshes.push(mesh);
    let node = Node::builder()
        .mesh(mesh_handle)
        .translation(Vec3::new(0.0, 0.0, -1.0))
        .scale(Vec3::new(1.0, 2.0, 1.0))
        .build();
    let node_handle = model.nodes.push(node);
    model.root.children.push(node_handle);
    scene.push(model);
    scene.push_default_model();

    scene.draw(&mut image);
    image.dump_png("target/sphere.png");
}

#[test]
fn triangle() {
    let mut image = Image::new(256, 256, ColorType::RGBA8);
    let mut scene = Scene::new();

    let mut model = Model::new();
    let mut prim = Primitive::unit_triangle();
    if let Geometry::Triangles(triangle) = &mut prim.geometry {
        triangle.vertices[0].color = Color::from(0xFF0000FF);
        triangle.vertices[1].color = Color::from(0x00FF00FF);
        triangle.vertices[2].color = Color::from(0x0000FFFF);
    }
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
    scene.push_default_model();

    scene.draw(&mut image);
    image.dump_png("target/triangle.png");
}

#[test]
fn cube_over_plane() {
    let mut image = Image::new(1024, 1024, ColorType::RGBA8);
    let mut scene = Scene::new();

    scene.load("tests/model/box/box.gltf").unwrap();
    scene.load("tests/model/box/box.gltf").unwrap();
    scene.load("tests/model/box/box.gltf").unwrap();
    scene.load("tests/model/box/box.gltf").unwrap();
    scene.push_default_model();

    let root0 = scene.model.nodes.get_mut(1.into()).unwrap();
    root0.trs.scale = Vec3::new(16.0, 16.0, 0.125);
    root0.trs.translation.translate(&Vec3::new(0.0, -1.0, 0.0));
    let root0_child = scene.model.nodes.get_mut(2.into()).unwrap();
    root0_child.trs.rotation = Quat::default();
    {
        let blue_mat = scene.model.materials.get_mut(0.into()).unwrap();
        blue_mat.color = Color::new(0.1, 0.2, 0.7, 1.0);
        blue_mat.metallic_factor = 1.0;
    }

    let root1 = scene
        .model
        .nodes
        .get_mut(scene.model.root.children[1])
        .unwrap();
    let shift = Vec3::new(1.0, 1.0, -2.0);
    root1.trs.translation += shift;

    let root2 = scene
        .model
        .nodes
        .get_mut(scene.model.root.children[2])
        .unwrap();
    let shift = Vec3::new(0.0, 0.0, -1.0);
    root2.trs.translation += shift;

    let root3 = scene
        .model
        .nodes
        .get_mut(scene.model.root.children[3])
        .unwrap();
    let shift = Vec3::new(-1.5, 0.0, -4.0);
    root3.trs.translation += shift;

    scene.draw(&mut image);
    image.dump_png("target/cube-over-plane.png");
}

mod gltf {
    use super::*;

    #[test]
    fn cube() {
        let mut image = Image::new(256, 256, ColorType::RGBA8);
        let mut scene = Scene::new();

        scene.load("tests/model/box/box.gltf").unwrap();
        scene.push_default_model();

        scene.draw(&mut image);
        image.dump_png("target/gltf-cube.png");
    }

    #[test]
    fn triangle() {
        let mut image = Image::new(128, 128, ColorType::RGBA8);
        let mut scene = Scene::new();
        scene.load("tests/model/triangle/triangle.gltf").unwrap();
        scene.push_default_model();

        scene.draw(&mut image);
        image.dump_png("target/gltf-triangle.png");
    }

    #[test]
    fn suzanne() {
        let mut image = Image::new(128, 128, ColorType::RGBA8);
        let mut scene = Scene::new();

        scene.load("tests/model/suzanne/suzanne.gltf").unwrap();
        scene.push_default_model();

        scene.draw(&mut image);
        image.dump_png("target/suzanne.png");
    }

    #[test]
    fn duck() {
        let mut image = Image::new(512, 512, ColorType::RGBA8);
        let mut scene = Scene::new();

        scene.load("tests/model/duck/duck.gltf").unwrap();

        // Custom camera
        add_camera(&mut scene.model, Vec3::new(0.1, 0.8, 2.2));
        scene.model.root.trs.scale *= 0.125;

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

    /// Add a custom camera
    fn add_camera(model: &mut Model, camera_position: Vec3) {
        let mut camera_node = Node::builder()
            .id(model.nodes.len())
            .translation(camera_position)
            .build();
        camera_node.camera = model.cameras.push(Camera::default());
        let camera_node_handle = model.nodes.push(camera_node);
        model.root.children.push(camera_node_handle);
    }

    #[test]
    fn orientation() {
        let mut image = Image::new(512, 512, ColorType::RGBA8);
        let mut scene = Scene::new();
        scene
            .load("tests/model/orientation/OrientationTest.gltf")
            .unwrap();
        add_camera(&mut scene.model, Vec3::new(0.0, 0.0, 20.0));

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

        add_camera(&mut scene.model, Vec3::new(0.0, 0.32, 1.0));

        scene.draw(&mut image);
        image.dump_png("target/flight.png");
    }

    #[test]
    fn sponza() {
        let mut image = Image::new(32, 32, ColorType::RGBA8);
        let mut scene = Scene::new();

        scene.load("tests/model/sponza/sponza.gltf").unwrap();

        // Custom camera
        let rotation = Quat::new(0.0, -0.707, 0.0, 0.707);
        scene.model.nodes.get_mut(0.into()).unwrap().trs.rotation = rotation;

        let mut camera_node = Node::builder()
            .id(scene.model.nodes.len())
            .translation(Vec3::new(0.2, 1.0, 0.0))
            .build();
        camera_node.camera = scene.model.cameras.push(Camera::default());
        let camera_node_handle = scene.model.nodes.push(camera_node);
        scene.model.root.children.push(camera_node_handle);

        scene.draw(&mut image);
        image.dump_png("target/sponza.png");
    }
}
