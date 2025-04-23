// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_blobs_shared::blobs::BlobRequest;
use fendermint_actor_blobs_shared::{
    blobs::{
        BlobStatus, FinalizeBlobParams, GetAddedBlobsParams, GetBlobStatusParams,
        GetPendingBlobsParams, SetBlobPendingParams,
    },
    credit::{Credit, GasAllowance, GetGasAllowanceParams, UpdateGasAllowanceParams},
};
use fendermint_actor_recall_config_shared::get_config;
use fil_actors_runtime::{runtime::Runtime, ActorError, SYSTEM_ACTOR_ADDR};
use fvm_shared::error::ExitCode;
use num_traits::Zero;
use recall_actor_sdk::{
    caller::{Caller, CallerOption},
    evm::emit_evm_event,
};

use crate::{
    actor::{delete_from_disc, BlobsActor},
    sol_facade::{blobs as sol_blobs, credit::CreditDebited},
    state::blobs::FinalizeBlobStateParams,
    State,
};

impl BlobsActor {
    /// Returns the gas allowance from a credit purchase for an address.
    ///
    /// This method is called by the recall executor, and as such, cannot fail.
    pub fn get_gas_allowance(
        rt: &impl Runtime,
        params: GetGasAllowanceParams,
    ) -> Result<GasAllowance, ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        let from_caller = match Caller::new(rt, params.0, None, CallerOption::None) {
            Ok(caller) => caller,
            Err(e) => {
                return if e.exit_code() == ExitCode::USR_FORBIDDEN {
                    // Disallowed actor type (this is called by all txns so we can't error)
                    Ok(GasAllowance::default())
                } else {
                    Err(e)
                };
            }
        };

        let allowance = rt.state::<State>()?.get_gas_allowance(
            rt.store(),
            from_caller.state_address(),
            rt.curr_epoch(),
        )?;

        Ok(allowance)
    }

    /// Updates gas allowance for the `from` address.
    ///
    /// The allowance update is applied to `sponsor` if it exists.
    /// The `from` address must have an approval from `sponsor`.
    /// The `from` address can be any actor, including those without delegated addresses.
    /// This method is called by the recall executor, and as such, cannot fail.
    pub fn update_gas_allowance(
        rt: &impl Runtime,
        params: UpdateGasAllowanceParams,
    ) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        let caller = Caller::new(rt, params.from, params.sponsor, CallerOption::None)?;

        rt.transaction(|st: &mut State, rt| {
            st.update_gas_allowance(
                rt.store(),
                caller.state_address(),
                caller.sponsor_state_address(),
                params.add_amount,
                rt.curr_epoch(),
            )
        })
    }

    /// Returns the current [`BlobStatus`] for a blob by hash.
    pub fn get_blob_status(
        rt: &impl Runtime,
        params: GetBlobStatusParams,
    ) -> Result<Option<BlobStatus>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        let caller = Caller::new(rt, params.subscriber, None, CallerOption::None)?;

        rt.state::<State>()?.get_blob_status(
            rt.store(),
            caller.state_address(),
            params.hash,
            params.id,
        )
    }

    /// Returns a list of [`BlobRequest`]s that are currenlty in the [`BlobStatus::Added`] state.
    ///
    /// All blobs that have been added but have not yet been picked up by validators for download
    /// are in the [`BlobStatus::Added`] state.
    pub fn get_added_blobs(
        rt: &impl Runtime,
        params: GetAddedBlobsParams,
    ) -> Result<Vec<BlobRequest>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        rt.state::<State>()?.get_added_blobs(rt.store(), params.0)
    }

    /// Returns a list of [`BlobRequest`]s that are currenlty in the [`BlobStatus::Pending`] state.
    ///
    /// All blobs that have been added and picked up by validators for download are in the
    /// [`BlobStatus::Pending`] state.
    /// These are the blobs that validators are currently coordinating to download. They will
    /// vote on the final status ([`BlobStatus::Resolved`] or [`BlobStatus::Failed`]), which is
    /// recorded on-chain with the `finalize_blob` method.
    pub fn get_pending_blobs(
        rt: &impl Runtime,
        params: GetPendingBlobsParams,
    ) -> Result<Vec<BlobRequest>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        rt.state::<State>()?.get_pending_blobs(rt.store(), params.0)
    }

    /// Sets a blob to the [`BlobStatus::Pending`] state.
    ///
    /// The `subscriber` address must be delegated (only delegated addresses can use credit).
    pub fn set_blob_pending(
        rt: &impl Runtime,
        params: SetBlobPendingParams,
    ) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        let caller = Caller::new_delegated(rt, params.subscriber, None, CallerOption::None)?;

        rt.transaction(|st: &mut State, rt| {
            st.set_blob_pending(
                rt.store(),
                caller.state_address(),
                params.hash,
                params.size,
                params.id,
                params.source,
            )
        })?;

        emit_evm_event(
            rt,
            sol_blobs::BlobPending {
                subscriber: caller.event_address(),
                hash: &params.hash,
                source: &params.source,
            },
        )
    }

    /// Finalizes a blob to the [`BlobStatus::Resolved`] or [`BlobStatus::Failed`] state.
    ///
    /// This is the final protocol step to add a blob, which is controlled by validator consensus.
    /// The [`BlobStatus::Resolved`] state means that a quorum of validators was able to download the blob.
    /// The [`BlobStatus::Failed`] state means that a quorum of validators was not able to download the blob.
    ///
    /// The `subscriber` address must be delegated (only delegated addresses can use credit).
    pub fn finalize_blob(rt: &impl Runtime, params: FinalizeBlobParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        let caller = Caller::new_delegated(rt, params.subscriber, None, CallerOption::None)?;
        let event_resolved = matches!(params.status, BlobStatus::Resolved);

        rt.transaction(|st: &mut State, rt| {
            st.finalize_blob(
                rt.store(),
                caller.state_address(),
                FinalizeBlobStateParams::from_actor_params(params.clone(), rt.curr_epoch()),
            )
        })?;

        emit_evm_event(
            rt,
            sol_blobs::BlobFinalized {
                subscriber: caller.event_address(),
                hash: &params.hash,
                resolved: event_resolved,
            },
        )
    }

    /// Debits accounts for current blob usage.
    ///
    /// This is called by the system actor every X blocks, where X is set in the recall config actor.
    pub fn debit_accounts(rt: &impl Runtime) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        let config = get_config(rt)?;

        let mut credit_debited = Credit::zero();
        let (deletes, num_accounts, more_accounts) = rt.transaction(|st: &mut State, rt| {
            let initial_credit_debited = st.credits.credit_debited.clone();
            let (deletes, more_accounts) =
                st.debit_accounts(rt.store(), &config, rt.curr_epoch())?;
            credit_debited = &st.credits.credit_debited - initial_credit_debited;
            let num_accounts = st.accounts.len();
            Ok((deletes, num_accounts, more_accounts))
        })?;

        for hash in deletes {
            delete_from_disc(hash)?;
        }

        emit_evm_event(
            rt,
            CreditDebited {
                amount: credit_debited,
                num_accounts,
                more_accounts,
            },
        )?;

        Ok(())
    }
}
