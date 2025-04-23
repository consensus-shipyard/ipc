// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_blobs_shared::GetStatsReturn;
use fendermint_actor_recall_config_shared::get_config;
use fil_actors_runtime::{runtime::Runtime, ActorError};

use crate::{actor::BlobsActor, State};

impl BlobsActor {
    /// Returns credit and storage usage statistics.
    pub fn get_stats(rt: &impl Runtime) -> Result<GetStatsReturn, ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        let config = get_config(rt)?;
        let stats = rt
            .state::<State>()?
            .get_stats(&config, rt.current_balance());

        Ok(stats)
    }
}
