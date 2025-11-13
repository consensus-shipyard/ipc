// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_blobs_shared::{
    accounts::SetAccountStatusParams, blobs::TrimBlobExpiriesParams, bytes::B256,
};
use fendermint_actor_recall_config_shared::{get_config, require_caller_is_admin};
use fil_actors_runtime::{runtime::Runtime, ActorError};
use recall_actor_sdk::caller::{Caller, CallerOption};

use crate::{
    actor::{delete_from_disc, BlobsActor},
    State,
};

impl BlobsActor {
    /// Sets the account status for an address.
    ///
    /// The `subscriber` address must be delegated (only delegated addresses can use credit).
    pub fn set_account_status(
        rt: &impl Runtime,
        params: SetAccountStatusParams,
    ) -> Result<(), ActorError> {
        require_caller_is_admin(rt)?;

        let caller = Caller::new_delegated(rt, params.subscriber, None, CallerOption::None)?;
        let config = get_config(rt)?;

        rt.transaction(|st: &mut State, rt| {
            st.set_account_status(
                rt.store(),
                &config,
                caller.state_address(),
                params.status,
                rt.curr_epoch(),
            )
        })
    }

    /// Trims the subscription expiries for an account based on its current maximum allowed blob TTL.
    ///
    /// This is used in conjunction with `set_account_status` when reducing an account's maximum
    /// allowed blob TTL.
    /// Returns the number of subscriptions processed and the next key to continue iteration.
    ///
    /// The `subscriber` address must be delegated (only delegated addresses can use credit).
    pub fn trim_blob_expiries(
        rt: &impl Runtime,
        params: TrimBlobExpiriesParams,
    ) -> Result<(u32, Option<B256>), ActorError> {
        require_caller_is_admin(rt)?;

        let caller = Caller::new_delegated(rt, params.subscriber, None, CallerOption::None)?;
        let config = get_config(rt)?;

        let (processed, next_key, deleted_blobs) = rt.transaction(|st: &mut State, rt| {
            st.trim_blob_expiries(
                &config,
                rt.store(),
                caller.state_address(),
                rt.curr_epoch(),
                params.starting_hash,
                params.limit,
            )
        })?;

        for hash in deleted_blobs {
            delete_from_disc(hash)?;
        }

        Ok((processed, next_key))
    }
}
