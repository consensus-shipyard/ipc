// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::syncer::error::Error;
use crate::syncer::payload::ParentBlockView;
use crate::{BlockHeight, SequentialKeyCache};

/// Stores the parent view observed of the current node
pub trait ParentViewStore {
    /// Store a newly observed parent view
    fn store(&mut self, view: ParentBlockView) -> Result<(), Error>;

    /// Get the parent view at the specified height
    fn get(&self, height: BlockHeight) -> Result<Option<ParentBlockView>, Error>;

    /// Purge the parent view at the target height
    fn purge(&mut self, height: BlockHeight) -> Result<(), Error>;

    fn min_parent_view_height(&self) -> Result<Option<BlockHeight>, Error>;

    fn max_parent_view_height(&self) -> Result<Option<BlockHeight>, Error>;
}

pub struct InMemoryParentViewStore {
    inner: SequentialKeyCache<BlockHeight, ParentBlockView>,
}

impl ParentViewStore for InMemoryParentViewStore {
    fn store(&mut self, view: ParentBlockView) -> Result<(), Error> {
        self.inner
            .append(view.parent_height, view)
            .map_err(|_| Error::NonSequentialParentViewInsert)
    }

    fn get(&self, height: BlockHeight) -> Result<Option<ParentBlockView>, Error> {
        Ok(self.inner.get_value(height).cloned())
    }

    fn purge(&mut self, height: BlockHeight) -> Result<(), Error> {
        self.inner.remove_key_below(height + 1);
        Ok(())
    }

    fn min_parent_view_height(&self) -> Result<Option<BlockHeight>, Error> {
        Ok(self.inner.lower_bound())
    }

    fn max_parent_view_height(&self) -> Result<Option<BlockHeight>, Error> {
        Ok(self.inner.upper_bound())
    }
}
