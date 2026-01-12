// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use cfg_aliases::cfg_aliases;

fn main() {
    // Setup cfg aliases
    cfg_aliases! {
        // Convenience aliases
        wasm_browser: { all(target_family = "wasm", target_os = "unknown") },
    }
}
