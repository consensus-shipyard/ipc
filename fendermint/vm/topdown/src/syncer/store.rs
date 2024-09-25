// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::syncer::error::Error;
use crate::syncer::payload::ParentView;
use crate::BlockHeight;

/// Stores the parent view observed of the current node
pub trait ParentViewStore {
    /// Store a newly observed parent view
    fn store(&mut self, view: ParentView) -> Result<(), Error>;

    /// Get the parent view at the specified height
    fn get(&self, height: BlockHeight) -> Result<Option<ParentView>, Error>;

    /// Purge the parent view at the target height
    fn purge(&mut self, height: BlockHeight) -> Result<(), Error>;

    fn minimal_parent_view_height(&self) -> Result<Option<BlockHeight>, Error>;

    fn max_parent_view_height(&self) -> Result<Option<BlockHeight>, Error>;
}
