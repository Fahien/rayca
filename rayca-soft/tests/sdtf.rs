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
    image.fill(RGBA8::BLACK);
    config.apply(sdtf_config);
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
        Config::builder().bvh(false).build(),
    );
}

#[test]
fn scene2() {
    run_test(
        "sdtf/1/scene2.sdtf",
        "sdtf/1/scene2.png",
        Config::builder().bvh(false).build(),
    );
}

#[test]
fn scene3() {
    run_test(
        "sdtf/1/scene3.sdtf",
        "sdtf/1/scene3.png",
        Config::builder().bvh(false).build(),
    );
}

#[test]
fn scene4() {
    run_test(
        "sdtf/1/scene4-ambient.sdtf",
        "sdtf/1/scene4-ambient.png",
        Config::builder().bvh(false).build(),
    );
    run_test(
        "sdtf/1/scene4-diffuse.sdtf",
        "sdtf/1/scene4-diffuse.png",
        Config::builder().bvh(false).build(),
    );
    run_test(
        "sdtf/1/scene4-emission.sdtf",
        "sdtf/1/scene4-emission.png",
        Config::builder().bvh(false).build(),
    );
    run_test(
        "sdtf/1/scene4-specular.sdtf",
        "sdtf/1/scene4-specular.png",
        Config::builder().bvh(false).build(),
    );
}

#[test]
fn scene5() {
    run_test(
        "sdtf/1/scene5.sdtf",
        "sdtf/1/scene5.png",
        Config::builder().bvh(false).build(),
    );
}

#[test]
fn scene6() {
    run_test(
        "sdtf/1/scene6.sdtf",
        "sdtf/1/scene6.png",
        Config::builder().bvh(false).build(),
    );
}

#[test]
#[ignore]
fn scene7() {
    run_test(
        "sdtf/1/scene7.sdtf",
        "sdtf/1/scene7.png",
        Config::builder().bvh(false).build(),
    );
}

#[test]

fn analytic() {
    run_test(
        "sdtf/2/analytic.sdtf",
        "sdtf/2/analytic.png",
        Config::builder().bvh(false).build(),
    );
}

#[test]
fn direct3x3() {
    run_test(
        "sdtf/2/direct3x3.sdtf",
        "sdtf/2/direct3x3.png",
        Config::builder().bvh(false).build(),
    );
}

#[test]
fn direct9() {
    run_test(
        "sdtf/2/direct9.sdtf",
        "sdtf/2/direct9.png",
        Config::builder().bvh(false).build(),
    );
}

#[test]
fn sphere() {
    run_test(
        "sdtf/2/sphere.sdtf",
        "sdtf/2/sphere.png",
        Config::builder().bvh(false).build(),
    );
}

#[test]
fn cornell() {
    run_test(
        "sdtf/2/cornell.sdtf",
        "sdtf/2/cornell.png",
        Config::builder().bvh(false).build(),
    );
}

#[test]
#[ignore]
fn dragon() {
    run_test(
        "sdtf/2/dragon.sdtf",
        "sdtf/2/dragon.png",
        Config::builder().bvh(true).build(),
    );
}

#[test]
fn cornell2() {
    run_test(
        "sdtf/3/cornellSimple.sdtf",
        "sdtf/3/cornellSimple.png",
        Config::builder().bvh(false).build(),
    );
}

#[test]
fn cornell3() {
    run_test(
        "sdtf/3/cornellNEE.sdtf",
        "sdtf/3/cornellNEE.png",
        Config::builder().bvh(false).build(),
    );
}

#[test]
fn cornell4() {
    run_test(
        "sdtf/3/cornellRR.sdtf",
        "sdtf/3/cornellRR.png",
        Config::builder().bvh(false).build(),
    );
}

#[test]
#[ignore]
fn dragon2() {
    run_test(
        "sdtf/3/dragon.sdtf",
        "sdtf/3/dragon2.png",
        Config::builder().bvh(true).build(),
    );
}

#[test]
fn cornell5() {
    run_test(
        "sdtf/4/cornellCosine.sdtf",
        "sdtf/4/cornellCosine.png",
        Config::builder().bvh(false).build(),
    );
}
