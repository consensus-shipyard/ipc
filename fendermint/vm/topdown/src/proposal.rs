// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::{BlockHeight, Error};
use crate::syncer::payload::ParentView;

pub struct Proposal {
    parent_height: BlockHeight,

}

pub struct ProposalMaker {}

impl ProposalMaker {
    /// Append a new parent view to the proposal maker. If there is a new proposal that can be
    /// made, returns it. Else returns None.
    pub fn append_new_view(&mut self, view: ParentView) -> Result<Option<Proposal>, Error> {
        todo!()
    }

    pub fn get_proposal_at_height(&self, height: BlockHeight) -> Result<Option<Proposal>, Error> {
        todo!()
    }

    /// Purge the proposals before the target height, inclusive
    pub fn finalize(&mut self, height: BlockHeight) -> Result<(), Error> {
        todo!()
    }

}