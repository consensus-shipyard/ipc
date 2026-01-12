// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Storage trait and implementations for the storage node
//!
//! This module provides:
//! - A trait for storing node state (e.g., last polled height)
//! - An in-memory implementation for development/testing

use anyhow::Result;
use std::sync::RwLock;

/// Storage trait for persisting node state
pub trait Store: Send + Sync {
    /// Get the last polled block height
    fn get_last_polled_height(&self) -> Result<Option<u64>>;

    /// Store the last polled block height
    fn set_last_polled_height(&self, height: u64) -> Result<()>;
}

/// In-memory implementation of the Store trait
///
/// This implementation stores state in memory and is suitable for
/// development and testing. State is lost when the node restarts.
pub struct InMemoryStore {
    last_polled_height: RwLock<Option<u64>>,
}

impl InMemoryStore {
    /// Create a new in-memory store
    pub fn new() -> Self {
        Self {
            last_polled_height: RwLock::new(None),
        }
    }

    /// Create a new in-memory store with an initial height
    pub fn with_initial_height(height: u64) -> Self {
        Self {
            last_polled_height: RwLock::new(Some(height)),
        }
    }
}

impl Default for InMemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl Store for InMemoryStore {
    fn get_last_polled_height(&self) -> Result<Option<u64>> {
        let guard = self
            .last_polled_height
            .read()
            .map_err(|e| anyhow::anyhow!("failed to acquire read lock: {}", e))?;
        Ok(*guard)
    }

    fn set_last_polled_height(&self, height: u64) -> Result<()> {
        let mut guard = self
            .last_polled_height
            .write()
            .map_err(|e| anyhow::anyhow!("failed to acquire write lock: {}", e))?;
        *guard = Some(height);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_store() {
        let store = InMemoryStore::new();

        // Initially None
        assert_eq!(store.get_last_polled_height().unwrap(), None);

        // Set and get
        store.set_last_polled_height(100).unwrap();
        assert_eq!(store.get_last_polled_height().unwrap(), Some(100));

        // Update
        store.set_last_polled_height(200).unwrap();
        assert_eq!(store.get_last_polled_height().unwrap(), Some(200));
    }

    #[test]
    fn test_in_memory_store_with_initial_height() {
        let store = InMemoryStore::with_initial_height(50);
        assert_eq!(store.get_last_polled_height().unwrap(), Some(50));
    }
}
