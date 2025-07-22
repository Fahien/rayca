// Copyright © 2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use crate::*;

pub fn get_workspace_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // Remove the current crate directory
    path
}

pub fn get_tests_path() -> PathBuf {
    let mut path = get_workspace_path();
    path.push("tests");
    if !path.exists() {
        log::error!("Failed to find tests directory: {}", path.display());
        panic!();
    }
    path
}

pub fn get_model_path() -> PathBuf {
    let mut path = get_tests_path();
    path.push("model");
    if !path.exists() {
        log::error!("Failed to find model directory: {}", path.display());
        panic!();
    }
    path
}

pub fn get_artifact_path() -> PathBuf {
    let mut path = get_tests_path();
    path.push("artifact");
    if !path.exists() {
        log::info!("Creating artifacts directory: {}", path.display());
        std::fs::create_dir_all(&path).unwrap();
    }
    path
}
