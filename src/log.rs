// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

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
        format!("{:>12} {}", owo_colors::OwoColorize::red(&owo_colors::OwoColorize::bold(&"Failed")), $( $t )*)
    }
}
