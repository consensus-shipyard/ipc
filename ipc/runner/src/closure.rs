// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::thread::JoinHandle;

pub struct ClosureRunner {
    handle: Option<JoinHandle<()>>,
    name: String,
}

impl ClosureRunner {
    pub fn new<F>(name: &str, task: F) -> Self
    where
        F: FnOnce() + Send + 'static,
    {
        let name = name.to_string();
        let handle = Some(std::thread::spawn(task));
        Self { handle, name }
    }

    pub fn join(self) {
        if let Some(handle) = self.handle {
            let _ = handle.join();
        }
    }
}
