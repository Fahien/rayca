// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use rayca::*;

#[test]
fn sphere() {
    let mut image = Image::new(256, 256, ColorType::RGBA8);
    let mut scene = Scene::new_with_config(Config::new(false, Box::new(Scratcher::new())));

    let mut model = Model::new();
    let sphere = Sphere::new(Point3::default(), 1.0);
    let prim = Primitive::sphere(sphere);
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
    scene.models.push(model);

    scene.draw(&mut image);
    image.dump_png("target/sphere.png");
}

#[test]
fn triangle() {
    let mut image = Image::new(256, 256, ColorType::RGBA8);
    let mut scene = Scene::new();

    let mut model = Model::new();
    let mut prim = Primitive::unit_triangle();
    let triangles = prim.triangles.as_mut().unwrap();
    triangles.vertices[0].color = Color::from(0xFF0000FF);
    triangles.vertices[1].color = Color::from(0x00FF00FF);
    triangles.vertices[2].color = Color::from(0x0000FFFF);

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
    let mut image = Image::new(512, 512, ColorType::RGBA8);
    let mut scene = Scene::new();

    scene.load("tests/model/box/box.gltf").unwrap();
    scene.load("tests/model/box/box.gltf").unwrap();
    scene.load("tests/model/box/box.gltf").unwrap();
    scene.load("tests/model/box/box.gltf").unwrap();

    scene.models[0].root.trs.scale = Vec3::new(16.0, 0.125, 16.0);
    scene.models[0].root.trs.translation += Vec3::new(0.0, -1.0, 0.0);
    scene.models[0]
        .nodes
        .get_mut(1.into())
        .unwrap()
        .trs
        .rotation = Quat::default();
    {
        let blue_mat = scene.models[0].materials.get_mut(0.into()).unwrap();
        blue_mat.color = Color::new(0.1, 0.2, 0.7, 1.0);
        blue_mat.metallic_factor = 1.0;
    }

    let shift = Vec3::new(1.0, 1.0, -2.0);
    scene.models[1].root.trs.translation += shift;

    let shift = Vec3::new(0.0, 0.0, -1.0);
    scene.models[2].root.trs.translation += shift;

    let shift = Vec3::new(-1.5, 0.0, -4.0);
    scene.models[3].root.trs.translation += shift;

    let mut sphere_model = Model::new();
    let sphere_prim = Primitive::sphere(Sphere::default());
    let prim_handle = sphere_model.primitives.push(sphere_prim);
    let sphere_mesh = sphere_model.meshes.push(Mesh::new(vec![prim_handle]));
    let sphere_node = sphere_model
        .nodes
        .push(Node::builder().translation(Vec3::new(-0.5, 2.0, -3.0)).mesh(sphere_mesh).build());
    sphere_model.root.children.push(sphere_node);
    scene.push_model(sphere_model);

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

        scene.load("tests/model/suzanne/suzanne.gltf").unwrap();

        scene.draw(&mut image);
        image.dump_png("target/suzanne.png");
    }

    #[test]
    fn duck() {
        let mut image = Image::new(512, 512, ColorType::RGBA8);
        let mut scene = Scene::new();

        scene.load("tests/model/duck/duck.gltf").unwrap();

        // Custom camera
        add_camera(&mut scene.models[0], Vec3::new(0.1, 0.8, 2.2));
        scene.models[0].root.trs.scale *= 0.125;

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
        camera_node.camera = Some(model.cameras.push(Camera::default()));
        let camera_node_handle = model.nodes.push(camera_node);
        model.root.children.push(camera_node_handle);
    }

    #[test]
    fn orientation() {
        let mut image = Image::new(128, 128, ColorType::RGBA8);
        let mut scene = Scene::new();
        scene
            .load("tests/model/orientation/OrientationTest.gltf")
            .unwrap();

        scene.models[0].root.trs.scale = Vec3::new(1.0 / 6.0, 1.0 / 6.0, 1.0 / 6.0);
        scene.models[0].root.trs.translation.set_z(-1.0);

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

        add_camera(&mut scene.models[0], Vec3::new(0.0, 0.32, 1.0));

        scene.draw(&mut image);
        image.dump_png("target/flight.png");
    }

    #[test]
    fn sponza() {
        let mut image = Image::new(8, 8, ColorType::RGBA8);
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

        scene.draw(&mut image);
        image.dump_png("target/sponza.png");
    }
}
