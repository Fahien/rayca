// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

#[cfg(feature = "web")]
pub mod context;
#[cfg(feature = "web")]
pub use context::*;
