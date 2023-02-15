// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use rayca::*;

fn run(mut scene: Scene, out_path: &str, width: u32, height: u32) {
    let mut image = Image::new(width, height, ColorType::RGBA8);

    scene.update();
    scene.draw(&mut image);

    image.dump_png(out_path);
}

fn _circle() {
    let scene = Scene::new();
    //let sphere = Sphere::new(Point3::new(0.0, 0.0, -1.0), 1.0);
    //let sphere_ex = SphereEx::new(RGBA8::from(0xFF0000FFu32));
    //scene.spheres.push(sphere);
    //scene.spheres_ex.push(sphere_ex);
    run(scene, "target/sphere.png", 128, 128);
}

fn _triangle() {
    let scene = Scene::new();
    //let triangle = Triangle::new(
    //    Point3::new(-1.0, -1.0, -1.0),
    //    Point3::new(1.0, -1.0, -1.0),
    //    Point3::new(0.0, 1.0, -1.0),
    //);
    //let mut triangle_ex = TriangleEx::default();
    //triangle_ex.vertices[0].color = Color::from(0xFF0000FF);
    //triangle_ex.vertices[1].color = Color::from(0x00FF00FF);
    //triangle_ex.vertices[2].color = Color::from(0x0000FFFF);
    //triangle_ex.vertices[0].normal = Vec3::new(0.0, 0.0, 1.0);
    //triangle_ex.vertices[1].normal = Vec3::new(1.0, 0.0, 0.0);
    //triangle_ex.vertices[2].normal = Vec3::new(0.0, 1.0, 0.0);
    //scene.triangles.push(triangle);
    //scene.triangles_ex.push(triangle_ex);
    run(scene, "target/triangle.png", 256, 256);
}

#[test]
fn boxes_over_plane() {
    let mut scene = Scene::new();

    let mut timer = Timer::new();
    let mut model = GltfModel::load_path("tests/model/box/box.gltf").unwrap();
    rlog!("Scene loaded in {}ms", timer.get_delta().as_millis());

    model.root.trs.scale = Vec3::new(4.0, 0.125, 8.0);
    model.root.trs.translation.y -= 1.0;
    model.root.trs.translation.z -= 2.0;

    model.nodes.get_mut(1.into()).unwrap().trs.rotation = Quat::default();
    model.materials.get_mut(0.into()).unwrap().color = Color::new(0.1, 0.2, 0.7, 1.0);

    scene.gltf_models.push(model);

    let model = GltfModel::load_path("tests/model/box/box.gltf").unwrap();
    scene.gltf_models.push(model);

    run(scene, "target/boxes-over-plane.png", 1024, 1024);
}

#[test]
fn gltf_box() {
    let mut scene = Scene::new();
    let model = GltfModel::load_path("tests/model/box/box.gltf").unwrap();
    scene.gltf_models.push(model);
    run(scene, "target/gltf-box.png", 128, 128);
}

#[test]
fn gltf_triangle() {
    let mut scene = Scene::new();
    let model = GltfModel::load_path("tests/model/triangle/triangle.gltf").unwrap();

    scene.gltf_models.push(model);
    run(scene, "target/gltf-triangle.png", 128, 128);
}

#[test]
fn gltf_suzanne() {
    let mut scene = Scene::new();

    let mut timer = Timer::new();
    let model = GltfModel::load_path("tests/model/suzanne/suzanne.gltf").unwrap();
    rlog!("Scene loaded in {}ms", timer.get_delta().as_millis());

    scene.gltf_models.push(model);
    run(scene, "target/gltf-suzanne.png", 128, 128);
}

#[test]
fn gtlf_duck() {
    let mut scene = Scene::new();

    let mut timer = Timer::new();
    let mut model = GltfModel::load_path("tests/model/duck/duck.gltf").unwrap();
    rlog!("Scene loaded in {}ms", timer.get_delta().as_millis());

    // Custom camera
    let mut camera_node = Node::builder()
        .id(model.nodes.len())
        .translation(Vec3::new(0.1, 0.8, 2.2))
        .build();
    camera_node.camera = Some(model.cameras.push(Camera::default()));
    let camera_node_handle = model.nodes.push(camera_node);
    model.root.children.push(camera_node_handle);

    scene.gltf_models.push(model);
    run(scene, "target/gltf-duck.png", 128, 128);
}

#[test]
fn gltf_cameras() {
    let mut scene = Scene::new();
    let model = GltfModel::load_path("tests/model/cameras/cameras.gltf").unwrap();
    scene.gltf_models.push(model);
    run(scene, "target/gltf-cameras.png", 256, 256);
}

#[test]
fn gltf_orientation() {
    let mut scene = Scene::new();
    let mut model = GltfModel::load_path("tests/model/orientation/orientation.gltf").unwrap();

    // Custom camera
    let mut camera_node = Node::builder()
        .id(model.nodes.len())
        .translation(Vec3::new(0.0, 0.1, 24.0))
        .build();
    camera_node.camera = Some(model.cameras.push(Camera::default()));
    let camera_node_handle = model.nodes.push(camera_node);
    model.root.children.push(camera_node_handle);
    scene.gltf_models.push(model);

    run(scene, "target/gltf-orientation.png", 256, 256);
}

#[test]
fn gltf_flight() {
    let mut scene = Scene::new();

    let mut model = GltfModel::load_path("tests/model/flight-helmet/flight-helmet.gltf").unwrap();

    // Custom camera
    let mut camera_node = Node::builder()
        .id(model.nodes.len())
        .translation(Vec3::new(0.0, 0.32, 1.0))
        .build();
    camera_node.camera = Some(model.cameras.push(Camera::default()));
    let camera_node_handle = model.nodes.push(camera_node);
    model.root.children.push(camera_node_handle);

    scene.gltf_models.push(model);
    run(scene, "target/gltf-flight.png", 32, 32);
}

#[test]
fn gltf_sponza() {
    let mut scene = Scene::new();

    let mut model = GltfModel::load_path("tests/model/sponza/sponza.gltf").unwrap();

    // Custom camera
    let rotation = Quat::new(0.0, -0.707, 0.0, 0.707);
    model.nodes.get_mut(0.into()).unwrap().trs.rotation = rotation;
    let mut camera_node = Node::builder()
        .id(model.nodes.len())
        .translation(Vec3::new(0.2, 1.0, 0.0))
        .build();
    camera_node.camera = Some(model.cameras.push(Camera::default()));
    let camera_node_handle = model.nodes.push(camera_node);
    model.root.children.push(camera_node_handle);
    scene.gltf_models.push(model);

    run(scene, "target/gltf-sponza.png", 32, 32);
}
