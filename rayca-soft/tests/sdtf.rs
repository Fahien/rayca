// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use rayca_soft::*;

fn run_test(in_path: &str, out_path: &str, mut config: Config) {
    let mut scene = Scene::default();

    let (_, sdtf_config) = scene
        .push_sdtf_from_path(tests::get_model_path().join(in_path))
        .unwrap();
    assert!(!scene.root.children.is_empty());

    let mut image = Image::new(sdtf_config.width, sdtf_config.height, ColorType::RGBA8);
    config.max_depth = sdtf_config.max_depth as u32;
    let mut renderer = SoftRenderer::new_with_config(config);
    renderer.draw(&scene, &mut image);

    let out_path = tests::get_artifact_path().join(out_path);
    let out_dir = out_path.parent().unwrap();
    std::fs::create_dir_all(out_dir).unwrap();
    image.dump_png(out_path).unwrap();
}

#[test]
fn scene1() {
    run_test(
        "sdtf/1/scene1.sdtf",
        "sdtf/1/scene1.png",
        Config::builder()
            .bvh(false)
            .integrator(IntegratorType::Raytracer)
            .build(),
    );
}

#[test]
fn scene2() {
    run_test(
        "sdtf/1/scene2.sdtf",
        "sdtf/1/scene2.png",
        Config::builder()
            .bvh(false)
            .integrator(IntegratorType::Raytracer)
            .build(),
    );
}

#[test]
fn scene3() {
    run_test(
        "sdtf/1/scene3.sdtf",
        "sdtf/1/scene3.png",
        Config::builder()
            .bvh(false)
            .integrator(IntegratorType::Raytracer)
            .build(),
    );
}

#[test]
fn scene4() {
    run_test(
        "sdtf/1/scene4-ambient.sdtf",
        "sdtf/1/scene4-ambient.png",
        Config::builder()
            .bvh(false)
            .integrator(IntegratorType::Raytracer)
            .build(),
    );
    run_test(
        "sdtf/1/scene4-diffuse.sdtf",
        "sdtf/1/scene4-diffuse.png",
        Config::builder()
            .bvh(false)
            .integrator(IntegratorType::Raytracer)
            .build(),
    );
    run_test(
        "sdtf/1/scene4-emission.sdtf",
        "sdtf/1/scene4-emission.png",
        Config::builder()
            .bvh(false)
            .integrator(IntegratorType::Raytracer)
            .build(),
    );
    run_test(
        "sdtf/1/scene4-specular.sdtf",
        "sdtf/1/scene4-specular.png",
        Config::builder()
            .bvh(false)
            .integrator(IntegratorType::Raytracer)
            .build(),
    );
}

#[test]
fn scene5() {
    run_test(
        "sdtf/1/scene5.sdtf",
        "sdtf/1/scene5.png",
        Config::builder()
            .bvh(false)
            .integrator(IntegratorType::Raytracer)
            .build(),
    );
}

#[test]
fn scene6() {
    run_test(
        "sdtf/1/scene6.sdtf",
        "sdtf/1/scene6.png",
        Config::builder()
            .bvh(false)
            .integrator(IntegratorType::Raytracer)
            .build(),
    );
}

#[test]
#[ignore]
fn scene7() {
    run_test(
        "sdtf/1/scene7.sdtf",
        "sdtf/1/scene7.png",
        Config::builder()
            .bvh(false)
            .integrator(IntegratorType::Raytracer)
            .build(),
    );
}
