// Copyright © 2022-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT;

use std::sync::Once;

static INIT: Once = Once::new();

/// Initializes logging for both native and WASM targets.
pub fn init() {
    INIT.call_once(|| {
        #[cfg(target_arch = "wasm32")]
        {
            // The "color" feature is enabled in Cargo.toml for styled output
            console_log::init_with_level(log::Level::Debug)
                .expect("error initializing console_log");
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            use env_logger::Env;
            env_logger::Builder::from_env(
                Env::default()
                    .default_filter_or("info")
                    .write_style_or("CLICOLOR_FORCE", "always"),
            )
            .init();
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_info() {
        init();
        log::info!("Some details");
    }

    #[test]
    fn test_print_warn() {
        init();
        log::warn!("Be careful");
    }

    #[test]
    fn test_print_fail() {
        init();
        log::error!("Something went wrong");
    }
}
