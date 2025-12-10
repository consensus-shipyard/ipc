// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm::kernel::prelude::Cid;
use fvm::kernel::Result;

pub trait RecallOps {
    fn block_add(&mut self, cid: Cid, data: &[u8]) -> Result<()>;
}
