// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::{borrow::Cow, path::Path};

pub struct Asset {
    data: Vec<u8>,
}

impl Asset {
    #[cfg(target_os = "android")]
    pub fn load<P: AsRef<Path>>(
        android_app: &winit::platform::android::activity::AndroidApp,
        path: P,
    ) -> Self {
        use std::str::FromStr;
        let str_path = path
            .as_ref()
            .to_str()
            .expect("Failed to convert path to str");
        let c_path = std::ffi::CString::from_str(str_path).expect("Failed to create C path");

        let msg = format!("Failed to open asset: {}", str_path);
        let mut asset = android_app
            .asset_manager()
            .open(c_path.as_c_str())
            .expect(&msg);

        let data = asset.buffer().expect("Failed to read asset data").to_vec();

        Self { data }
    }

    #[cfg(not(target_os = "android"))]
    pub fn load<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();
        let msg = format!("Failed to read asset file: {}", path.display());
        let data = std::fs::read(path).expect(&msg);
        Self { data }
    }

    pub fn get_bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.data
    }

    pub fn to_string<'a>(&'a self) -> Cow<'a, str> {
        match std::str::from_utf8(self.get_bytes()) {
            Ok(s) => Cow::Borrowed(s),
            Err(_) => Cow::Owned(String::from_utf8_lossy(self.get_bytes()).into_owned()),
        }
    }
}

impl std::ops::Deref for Asset {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl std::io::Read for Asset {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.data.as_slice().read(buf)
    }
}

pub struct Assets {
    #[cfg(target_os = "android")]
    pub android_app: winit::platform::android::activity::AndroidApp,
}

impl Assets {
    #[cfg(target_os = "android")]
    pub fn new(android_app: winit::platform::android::activity::AndroidApp) -> Self {
        Self { android_app }
    }

    #[cfg(not(target_os = "android"))]
    pub fn new() -> Self {
        Self {}
    }

    pub fn load<P: AsRef<Path>>(&self, path: P) -> Asset {
        Asset::load(
            #[cfg(target_os = "android")]
            &self.android_app,
            path,
        )
    }
}
