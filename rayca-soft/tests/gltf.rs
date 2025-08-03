// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::error::Error;

use rayca_soft::*;

#[test]
fn sphere() -> Result<(), Box<dyn Error>> {
    let mut image = Image::new(256, 256, ColorType::RGBA8);
    let mut renderer = SoftRenderer::new_with_config(
        Config::builder()
            .bvh(false)
            .integrator(IntegratorStrategy::Scratcher)
            .build(),
    );

    let mut model = Model::default();
    let geometry_handle = model.geometries.push(Geometry::Sphere(Sphere::unit()));
    let prim_handle = model
        .primitives
        .push(Primitive::builder().geometry(geometry_handle).build());
    let mesh_handle = model
        .meshes
        .push(Mesh::builder().primitive(prim_handle).build());
    let node = Node::builder()
        .mesh(mesh_handle)
        .trs(
            Trs::builder()
                .translation(Vec3::new(0.0, 0.0, -1.0))
                .scale(Vec3::new(1.0, 2.0, 1.0))
                .build(),
        )
        .build();
    let node_handle = model.nodes.push(node);
    model.root.children.push(node_handle);

    let mut scene = Scene::default();
    scene.push_model(model);
    scene.push_model(SoftRenderer::create_default_model());

    renderer.draw(&scene, &mut image);
    image.dump_png(tests::get_artifact_path().join("sphere.png"))?;
    Ok(())
}

#[test]
fn triangle() -> Result<(), Box<dyn Error>> {
    let mut image = Image::new(256, 256, ColorType::RGBA8);
    let mut renderer = SoftRenderer::default();

    let mut model = Model::default();
    let mut triangle = TriangleMesh::unit();
    triangle.vertices[0].ext.color = Color::from(0xFF0000FF);
    triangle.vertices[1].ext.color = Color::from(0x00FF00FF);
    triangle.vertices[2].ext.color = Color::from(0x0000FFFF);
    let geometry_handle = model.geometries.push(Geometry::TriangleMesh(triangle));
    let prim_handle = model
        .primitives
        .push(Primitive::builder().geometry(geometry_handle).build());
    let mesh_handle = model
        .meshes
        .push(Mesh::builder().primitive(prim_handle).build());
    let node = Node::builder()
        .mesh(mesh_handle)
        .trs(
            Trs::builder()
                .translation(Vec3::new(0.0, -1.0, 0.0))
                .scale(Vec3::new(1.0, 2.0, 1.0))
                .build(),
        )
        .build();
    let node_handle = model.nodes.push(node);
    model.root.children.push(node_handle);

    let mut scene = Scene::default();
    scene.push_model(model);
    scene.push_model(SoftRenderer::create_default_model());

    renderer.draw(&scene, &mut image);
    image.dump_png(tests::get_artifact_path().join("triangle.png"))?;
    Ok(())
}

#[test]
fn cube_over_plane() -> Result<(), Box<dyn Error>> {
    let mut image = Image::new(512, 512, ColorType::RGBA8);
    let mut renderer = SoftRenderer::default();

    let model_path = tests::get_model_path();
    let mut scene = Scene::default();
    let assets = Assets::new();

    // Blue floor
    {
        let node_handle = scene.push_model(Model::load_gltf_path(
            model_path.join("box/box.gltf"),
            &assets,
        )?);

        {
            let node = scene.nodes.get_mut(node_handle).unwrap();
            node.trs.scale = Vec3::new(16.0, 0.125, 16.0);
            node.trs.translation.translate(&Vec3::new(0.0, -1.0, 0.0));
        }

        let node = scene.get_node(node_handle).unwrap();
        let model = scene.get_model_mut(node.model.unwrap());
        let child = model.nodes.get_mut(1.into()).unwrap();
        child.trs.rotation = Quat::default();
        {
            let blue_mat_handle = model
                .materials
                .get(0.into())
                .unwrap()
                .get_pbr_material_handle();
            let pbr_material = model.get_pbr_material_mut(blue_mat_handle).unwrap();
            pbr_material.color = Color::new(0.1, 0.2, 0.7, 1.0);
            pbr_material.metallic_factor = 1.0;
        }
    }

    // Right cube
    {
        let model_node = scene.push_model(Model::load_gltf_path(
            model_path.join("box/box.gltf"),
            &assets,
        )?);
        let model_node = scene.get_node_mut(model_node);
        let shift = Vec3::new(1.0, 1.0, -2.0);
        model_node.trs.translation += shift;
    }

    // Middle cube
    {
        let model_node_handle = scene.push_model(Model::load_gltf_path(
            model_path.join("box/box.gltf"),
            &assets,
        )?);
        let model_node = scene.get_node_mut(model_node_handle);
        let shift = Vec3::new(0.0, 0.0, -1.0);
        model_node.trs.translation += shift;
    }

    // Left cube
    {
        let model_node_handle = scene.push_model(Model::load_gltf_path(
            model_path.join("box/box.gltf"),
            &assets,
        )?);
        let model_node = scene.get_node_mut(model_node_handle);
        let shift = Vec3::new(-1.5, 0.0, -4.0);
        model_node.trs.translation += shift;
    }

    // Sphere on top
    {
        let mut model = Model::default();
        let geometry_handle = model.geometries.push(Geometry::Sphere(Sphere::unit()));
        let primitive_handle = model
            .primitives
            .push(Primitive::builder().geometry(geometry_handle).build());
        let sphere_mesh = model
            .meshes
            .push(Mesh::builder().primitive(primitive_handle).build());
        let sphere_node = model.nodes.push(
            Node::builder()
                .trs(
                    Trs::builder()
                        .translation(Vec3::new(-0.5, 2.0, -3.0))
                        .build(),
                )
                .mesh(sphere_mesh)
                .build(),
        );
        model.root.children.push(sphere_node);
        scene.push_model(model);
    }

    scene.push_model(SoftRenderer::create_default_model());

    renderer.draw(&scene, &mut image);
    image.dump_png(tests::get_artifact_path().join("cube-over-plane.png"))?;
    Ok(())
}

mod gltf {
    use super::*;

    #[test]
    fn cube() -> Result<(), Box<dyn Error>> {
        let mut image = Image::new(256, 256, ColorType::RGBA8);
        let mut renderer = SoftRenderer::default();

        let mut scene = Scene::default();
        let assets = Assets::new();
        scene.push_gltf_from_path(tests::get_model_path().join("box/box.gltf"), &assets)?;
        scene.push_model(SoftRenderer::create_default_model());

        renderer.draw(&scene, &mut image);
        image.dump_png(tests::get_artifact_path().join("gltf-cube.png"))?;
        Ok(())
    }

    #[test]
    fn triangle() -> Result<(), Box<dyn Error>> {
        let mut scene = Scene::default();
        let assets = Assets::new();
        scene.push_gltf_from_path(
            tests::get_model_path().join("triangle/triangle.gltf"),
            &assets,
        )?;
        scene.push_model(SoftRenderer::create_default_model());

        let mut renderer = SoftRenderer::default();
        let mut image = Image::new(128, 128, ColorType::RGBA8);
        renderer.draw(&scene, &mut image);
        image.dump_png(tests::get_artifact_path().join("gltf-triangle.png"))?;
        Ok(())
    }

    #[test]
    fn suzanne() -> Result<(), Box<dyn Error>> {
        let mut image = Image::new(128, 128, ColorType::RGBA8);

        let mut scene = Scene::default();
        let assets = Assets::new();
        scene.push_gltf_from_path(
            tests::get_model_path().join("suzanne/suzanne.gltf"),
            &assets,
        )?;
        scene.push_model(SoftRenderer::create_default_model());

        let mut renderer = SoftRenderer::default();
        renderer.draw(&scene, &mut image);
        image.dump_png(tests::get_artifact_path().join("suzanne.png"))?;
        Ok(())
    }

    #[test]
    fn duck() -> Result<(), Box<dyn Error>> {
        let mut scene = Scene::default();
        let assets = Assets::new();
        let duck_node_handle =
            scene.push_gltf_from_path(tests::get_model_path().join("duck/duck.gltf"), &assets)?;
        let duck_node = scene.get_node_mut(duck_node_handle);
        duck_node.trs.scale(Vec3::from(0.125));

        scene.push_model(SoftRenderer::create_default_model());

        let mut renderer = SoftRenderer::default();
        let mut image = Image::new(512, 512, ColorType::RGBA8);
        renderer.draw(&scene, &mut image);
        image.dump_png(tests::get_artifact_path().join("duck.png"))?;
        Ok(())
    }

    #[test]
    fn cameras() -> Result<(), Box<dyn Error>> {
        let mut scene = Scene::default();
        let assets = Assets::new();
        scene.push_gltf_from_path(
            tests::get_model_path().join("cameras/cameras.gltf"),
            &assets,
        )?;
        let mut image = Image::new(256, 256, ColorType::RGBA8);
        let mut renderer = SoftRenderer::default();
        renderer.draw(&scene, &mut image);
        image.dump_png(tests::get_artifact_path().join("cameras.png"))?;
        Ok(())
    }

    /// Add a custom camera
    fn add_camera(model: &mut Model, camera_position: Vec3) {
        let camera_handle = model.cameras.push(Camera::default());
        let camera_node = Node::builder()
            .trs(Trs::builder().translation(camera_position).build())
            .camera(camera_handle)
            .build();
        let camera_node_handle = model.nodes.push(camera_node);
        model.root.children.push(camera_node_handle);
    }

    #[test]
    fn orientation() -> Result<(), Box<dyn Error>> {
        let mut scene = Scene::default();
        let assets = Assets::new();
        scene.push_gltf_from_path(
            tests::get_model_path().join("orientation/OrientationTest.gltf"),
            &assets,
        )?;
        add_camera(&mut scene.models[0], Vec3::new(0.0, 0.0, 20.0));

        let mut renderer = SoftRenderer::default();
        let mut image = Image::new(512, 512, ColorType::RGBA8);
        renderer.draw(&scene, &mut image);
        image.dump_png(tests::get_artifact_path().join("orientation.png"))?;
        Ok(())
    }

    #[test]
    fn flight() -> Result<(), Box<dyn Error>> {
        let mut scene = Scene::default();
        let assets = Assets::new();
        scene.push_gltf_from_path(
            tests::get_model_path().join("flight-helmet/FlightHelmet.gltf"),
            &assets,
        )?;

        add_camera(&mut scene.models[0], Vec3::new(0.0, 0.32, 1.0));

        let mut image = Image::new(32, 32, ColorType::RGBA8);
        let mut renderer = SoftRenderer::default();
        renderer.draw(&scene, &mut image);
        image.dump_png(tests::get_artifact_path().join("flight.png"))?;
        Ok(())
    }

    #[test]
    #[ignore]
    fn sponza() -> Result<(), Box<dyn Error>> {
        let mut scene = Scene::default();
        let assets = Assets::new();

        scene.push_gltf_from_path(tests::get_model_path().join("sponza/sponza.gltf"), &assets)?;

        // Custom camera
        let rotation = Quat::new(0.0, -0.707, 0.0, 0.707);
        scene.models[0]
            .nodes
            .get_mut(0.into())
            .unwrap()
            .trs
            .rotation = rotation;

        let camera_handle = scene.models[0].cameras.push(Camera::default());
        let camera_node = Node::builder()
            .trs(Trs::builder().translation(Vec3::new(0.2, 1.0, 0.0)).build())
            .camera(camera_handle)
            .build();
        let camera_node_handle = scene.models[0].nodes.push(camera_node);
        scene.models[0].root.children.push(camera_node_handle);

        let mut image = Image::new(32, 32, ColorType::RGBA8);
        let mut renderer = SoftRenderer::default();
        renderer.draw(&scene, &mut image);
        image.dump_png(tests::get_artifact_path().join("sponza.png"))?;
        Ok(())
    }
}
