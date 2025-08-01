// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use rayca_soft::*;

fn run_test(in_path: &str, out_path: &str, config: Config) {
    let mut renderer = SoftRenderer::new_with_config(config);
    let mut scene = Scene::default();

    let (_, config) = scene
        .push_sdtf_from_path(tests::get_model_path().join(in_path))
        .unwrap();
    assert!(!scene.root.children.is_empty());

    let mut image = Image::new(config.width, config.height, ColorType::RGBA8);
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
        Config::new(false, IntegratorType::Flat),
    );
}

#[test]
fn scene2() {
    run_test(
        "sdtf/1/scene2.sdtf",
        "sdtf/1/scene2.png",
        Config::new(false, IntegratorType::Flat),
    );
}

#[test]
fn scene3() {
    run_test(
        "sdtf/1/scene3.sdtf",
        "sdtf/1/scene3.png",
        Config::new(false, IntegratorType::Flat),
    );
}

#[test]
fn scene4() {
    run_test(
        "sdtf/1/scene4-ambient.sdtf",
        "sdtf/1/scene4-ambient.png",
        Config::new(false, IntegratorType::Flat),
    );
}

#[test]
fn scene5() {
    run_test(
        "sdtf/1/scene5.sdtf",
        "sdtf/1/scene5.png",
        Config::new(false, IntegratorType::Flat),
    );
}

#[test]
fn scene6() {
    run_test(
        "sdtf/1/scene6.sdtf",
        "sdtf/1/scene6.png",
        Config::new(false, IntegratorType::Flat),
    );
}

// TODO: enable this test when the scene is fixed
fn _scene7() {
    run_test(
        "sdtf/1/scene7.sdtf",
        "sdtf/1/scene7.png",
        Config::new(false, IntegratorType::Flat),
    );
}
