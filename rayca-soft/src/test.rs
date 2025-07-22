// Copyright © 2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use crate::*;

pub fn get_artifacts_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push("artifacts");
    if !path.exists() {
        log::info!("Creating artifacts directory: {}", path.display());
        std::fs::create_dir_all(&path).unwrap();
    }
    path
}
