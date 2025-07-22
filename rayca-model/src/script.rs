// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

type ScriptFn = Box<dyn Fn(f32, &mut Model, Handle<Node>)>;

impl Default for Script {
    fn default() -> Self {
        Self { update: None }
    }
}

pub struct Script {
    /// Using an option for taking it out when calling
    pub update: Option<ScriptFn>,
}

impl std::fmt::Debug for Script {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Script")
            .field("update", &self.update.is_some())
            .finish()
    }
}

impl Script {
    pub fn new(update: ScriptFn) -> Self {
        Self {
            update: Some(update),
        }
    }

    pub fn update(delta: f32, model: &mut Model, node: Handle<Node>) {
        let script_handle = model.nodes.get(node).unwrap().script;

        if let Some(script_handle) = script_handle {
            // Take function out for managing borrow checker
            let func_opt = model.scripts.get_mut(script_handle).unwrap().update.take();
            let func = func_opt.as_ref().unwrap();
            func(delta, model, node);
            // Restore function
            model.scripts.get_mut(script_handle).unwrap().update = func_opt;
        }

        let children = model.nodes.get(node).unwrap().children.clone();
        for child in children {
            Self::update(delta, model, child);
        }
    }
}
