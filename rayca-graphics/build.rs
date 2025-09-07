// Copyright © 2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

//! This build script performs two tasks for every WGSL shader listed:
//! 1. Full Naga validation (type/layout/spec compliance).
//! 2. Custom lint to detect *unused global or local variables*.
//!
//! **Why the custom lint?**
//!
//! In this project we observed that leaving declared-but-unused global resources
//! (e.g. a @group/@binding variable never read) can lead, at runtime, to an
//! unexpected pink / magenta frame. That coloration is a classic signal that the
//! pipeline or bound resources did not match expectations (often because the
//! shader compiler / driver stripped the unused binding while the Rust side still
//! created & bound it). By failing the build early we surface the problem at
//! compile time instead of during rendering.
//!
//! Policy:
//!  - Any unused global variable is treated as a hard error (no underscore
//!    exemption). Even a single unreferenced binding has produced magenta frames
//!    in practice, so every declared binding must be read at least once.
//!  - Local variables inside functions / entry points are also flagged to keep
//!    shaders tidy and to catch forgotten temporaries that may hint at logic
//!    mistakes.
//!
//! If this script panics with "Shader lint failed", inspect the listed symbols
//! and either remove them or actually use them in shader logic. After fixing,
//! rerun `cargo build` and the magenta frame symptom should disappear.

use std::{
    collections::{HashMap, HashSet},
    fs,
};

/// Scan a validated `naga::Module` and return a list of human-readable messages
/// describing unused WGSL globals or locals. No name exemptions: underscore-
/// prefixed variables are still considered unused if not referenced.
///
/// Limitations:
///  - Currently does not attempt interprocedural reachability for functions; it
///    simply records variable usage across all expressions. This is sufficient
///    for detecting unused globals that could cause pipeline / layout mismatch
///    (manifesting as a magenta frame when wgpu binds resources the shader
///    optimized away).
fn lint_unused(module: &naga::Module) -> Vec<String> {
    let mut errors = Vec::new();

    // Helper to scan expressions for called functions & used globals/locals.
    #[derive(Default)]
    struct Usage {
        globals: HashSet<naga::Handle<naga::GlobalVariable>>,
        locals: HashMap<usize, HashSet<naga::Handle<naga::LocalVariable>>>, // function index -> locals used
    }

    fn scan_function(
        fun_index: Option<usize>,
        expressions: &naga::Arena<naga::Expression>,
        usage: &mut Usage,
    ) {
        for (_expr_handle, expr) in expressions.iter() {
            match *expr {
                naga::Expression::GlobalVariable(g) => {
                    usage.globals.insert(g);
                }
                naga::Expression::LocalVariable(l) => {
                    if let Some(idx) = fun_index {
                        usage.locals.entry(idx).or_default().insert(l);
                    }
                }
                _ => {}
            }
        }
    }

    let mut usage = Usage::default();

    // Scan entry points first (their functions are inline, not in module.functions arena)
    for ep in &module.entry_points {
        scan_function(None, &ep.function.expressions, &mut usage);
    }

    // Also scan all user-defined functions so globals referenced only there are not treated as unused.
    for (handle, function) in module.functions.iter() {
        let idx = handle.index();
        scan_function(Some(idx), &function.expressions, &mut usage);
    }

    // Used globals already collected; now find unused globals
    for (handle, gv) in module.global_variables.iter() {
        // Skip if it is a built-in (no name and has binding?)? we still report unless underscore.
        let used = usage.globals.contains(&handle);
        let name = gv.name.as_deref().unwrap_or("<unnamed_global>");
        if !used {
            errors.push(format!("Unused WGSL global variable: {name}"));
        }
    }

    // Local variables: need to scan each function (reachable ones + entry points)
    // For entry points
    for ep in &module.entry_points {
        let fun = &ep.function;
        // Build set of used locals in this function from expressions
        let mut used_locals: HashSet<naga::Handle<naga::LocalVariable>> = HashSet::new();
        for (_eh, expr) in fun.expressions.iter() {
            if let naga::Expression::LocalVariable(lh) = *expr {
                used_locals.insert(lh);
            }
        }
        for (lh, lv) in fun.local_variables.iter() {
            let name = lv.name.as_deref().unwrap_or("<unnamed_local>");
            if !used_locals.contains(&lh) {
                errors.push(format!("Unused WGSL local (entry point) variable: {name}"));
            }
        }
    }

    // For reachable internal functions
    for (_handle, function) in module.functions.iter() {
        let mut used_locals: HashSet<naga::Handle<naga::LocalVariable>> = HashSet::new();
        for (_eh, expr) in function.expressions.iter() {
            if let naga::Expression::LocalVariable(lh) = *expr {
                used_locals.insert(lh);
            }
        }
        for (lh, lv) in function.local_variables.iter() {
            let name = lv.name.as_deref().unwrap_or("<unnamed_local>");
            if !used_locals.contains(&lh) {
                errors.push(format!(
                    "Unused WGSL local variable in function {}: {}",
                    function.name.as_deref().unwrap_or("<unnamed_function>"),
                    name
                ));
            }
        }
    }

    errors
}

/// Entry point for the build script.
///
/// For each WGSL shader:
///  1. Reads source from disk.
///  2. Parses & validates with Naga (mirrors wgpu's pipeline validation).
///  3. Runs `lint_unused` to catch unused globals/locals.
///
/// Any validation or lint error aborts the build with a panic. This *fails fast*
/// instead of allowing a runtime where wgpu might present a magenta frame due to
/// resource layout divergence (common when an unused binding is stripped by the
/// shader compiler).
///
/// There is no suppression mechanism: if something is intentionally unused you
/// must delete it (or temporarily comment it out) until it's needed.
fn main() {
    env_logger::init();

    let shaders = ["shader/compute.wgsl", "shader/present.wgsl"];

    let mut any_errors = false;
    for path in shaders {
        match fs::read_to_string(path) {
            Ok(src) => match naga::front::wgsl::parse_str(&src) {
                Ok(module) => {
                    let mut validator = naga::valid::Validator::new(
                        naga::valid::ValidationFlags::all(),
                        naga::valid::Capabilities::all(),
                    );
                    if let Err(e) = validator.validate(&module) {
                        log::error!("WGSL validation error in {path}: {e:?}");
                        any_errors = true;
                        continue;
                    }
                    let lint_errors = lint_unused(&module);
                    if !lint_errors.is_empty() {
                        any_errors = true;
                        log::error!("WGSL lint errors in {path}:");
                        for err in lint_errors {
                            eprintln!("  - {err}");
                        }
                    }
                }
                Err(parse_err) => {
                    log::error!("Failed to parse WGSL {path}: {parse_err:?}");
                    any_errors = true;
                }
            },
            Err(io_err) => {
                log::error!("Failed to read shader {path}: {io_err}");
                any_errors = true;
            }
        }
    }

    if any_errors {
        panic!("Shader lint failed (treating unused symbols / validation errors as fatal)");
    }

    for path in shaders {
        println!("cargo:rerun-if-changed={path}");
    }
}
