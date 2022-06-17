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
macro_rules! print_success {
    ( $s:expr, $( $t:tt )* ) => {
        println!("{:>12} {}",
            owo_colors::OwoColorize::bold(
                &owo_colors::OwoColorize::green(&$s)), format!($( $t )*))
    }
}

#[macro_export]
macro_rules! print_info {
    ( $s:expr, $( $t:tt )* ) => {
        println!("{:>12} {}", owo_colors::OwoColorize::bold(&owo_colors::OwoColorize::blue($s)), format!($( $t )*))
    }
}

#[macro_export]
macro_rules! fail {
    ( $( $t:tt )* ) => {
        format!("{:>12} {}", owo_colors::OwoColorize::bold(&owo_colors::OwoColorize::red(&"Failed")), format!($( $t )*))
    }
}

#[macro_export]
macro_rules! warn {
    ( $s:expr, $( $t:tt )* ) => {
        format!("{:>12} {}", owo_colors::OwoColorize::bold(&owo_colors::OwoColorize::yellow($s)), format!($( $t )*))
    }
}

#[macro_export]
macro_rules! panic_fail {
    ( $( $t:tt )* ) => {
        panic!("{:>12} {}", "Failed".red().bold(), format!($( $t )*))
    }
}
