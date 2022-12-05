// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

pub use owo_colors::OwoColorize;

#[cfg(not(target_arch = "wasm32"))]
#[macro_export]
macro_rules! rlog {
    ( $( $t:tt )* ) => {
        println!($( $t )*)
    }
}

#[macro_export]
macro_rules! fail {
    ( $( $t:tt )* ) => {
        format!("{:>12} {}", "Failed".red().bold(), $( $t )*)
    }
}
