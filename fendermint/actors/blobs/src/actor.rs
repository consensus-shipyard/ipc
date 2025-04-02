// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_blobs_shared::params::{
    AddBlobParams, ApproveCreditParams, BuyCreditParams, DeleteBlobParams, FinalizeBlobParams,
    GetAccountParams, GetAddedBlobsParams, GetBlobParams, GetBlobStatusParams,
    GetCreditApprovalParams, GetGasAllowanceParams, GetPendingBlobsParams, GetStatsReturn,
    OverwriteBlobParams, RevokeCreditParams, SetAccountStatusParams, SetBlobPendingParams,
    SetSponsorParams, TrimBlobExpiriesParams, UpdateGasAllowanceParams,
};
use fendermint_actor_blobs_shared::state::{
    BlobInfo, BlobRequest, BlobStatus, Credit, CreditApproval, GasAllowance, Hash, Subscription,
};
use fendermint_actor_blobs_shared::{state::AccountInfo, Method};
use fendermint_actor_recall_config_shared::{get_config, require_caller_is_admin};
use fil_actors_runtime::{
    actor_dispatch, actor_error, extract_send_result,
    runtime::{ActorCode, Runtime},
    ActorError, FIRST_EXPORTED_METHOD_NUMBER, SYSTEM_ACTOR_ADDR,
};
use fvm_ipld_encoding::ipld_block::IpldBlock;
use fvm_shared::{econ::TokenAmount, error::ExitCode, MethodNum, METHOD_SEND};
use num_traits::Zero;
use recall_actor_sdk::{
    emit_evm_event, require_addr_is_origin_or_caller, to_delegated_address, to_id_address,
    to_id_and_delegated_address, InputData, InvokeContractParams, InvokeContractReturn,
};

use crate::sol_facade::credit::{CreditApproved, CreditDebited, CreditPurchased, CreditRevoked};
use crate::sol_facade::gas::{GasSponsorSet, GasSponsorUnset};
use crate::sol_facade::{blobs as sol_blobs, credit as sol_credit, AbiCall, AbiCallRuntime};
use crate::{State, BLOBS_ACTOR_NAME};

#[cfg(feature = "fil-actor")]
fil_actors_runtime::wasm_trampoline!(BlobsActor);

/// Singleton actor for managing blob storage.
///
/// The [`Address`]es stored in this actor's state _must_ be ID-based addresses for
/// efficient comparison with message origin and caller addresses, which are always ID-based.
/// [`Address`]es in the method params can be of any type.
/// They will be resolved to ID-based addresses.
///
/// For simplicity, this actor currently manages both blobs and credit.
/// A future version of the protocol will likely separate them in some way.
pub struct BlobsActor;

impl BlobsActor {
    /// Creates a new `[BlobsActor]` state.
    ///
    /// This is only used in tests. This actor is created manually at genesis.
    fn constructor(rt: &impl Runtime) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;
        let state = State::new(rt.store())?;
        rt.create(&state)
    }

    /// Returns credit and storage usage statistics.
    fn get_stats(rt: &impl Runtime) -> Result<GetStatsReturn, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let config = get_config(rt)?;
        let stats = rt
            .state::<State>()?
            .get_stats(&config, rt.current_balance());
        Ok(stats)
    }

    /// Buy credit with token.
    ///
    /// The recipient address must be delegated (only delegated addresses can own credit).
    fn buy_credit(rt: &impl Runtime, params: BuyCreditParams) -> Result<AccountInfo, ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        let (id_addr, delegated_addr) = to_id_and_delegated_address(rt, params.0)?;

        let config = get_config(rt)?;

        let mut credit_amount = Credit::zero();
        let account = rt.transaction(|st: &mut State, rt| {
            let pre_buy = st.credit_sold.clone();
            let account = st.buy_credit(
                &config,
                rt.store(),
                id_addr,
                rt.message().value_received(),
                rt.curr_epoch(),
            )?;
            credit_amount = &st.credit_sold - &pre_buy;
            Ok(account)
        })?;

        emit_evm_event(rt, CreditPurchased::new(delegated_addr, credit_amount))?;

        AccountInfo::from(rt, account)
    }

    /// Updates gas allowance for the `from` address.
    ///
    /// The allowance update is applied to `sponsor` if it exists.
    /// The `from` address must have an approval from `sponsor`.
    /// This method is called by the recall executor, and as such, cannot fail.
    fn update_gas_allowance(
        rt: &impl Runtime,
        params: UpdateGasAllowanceParams,
    ) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        let from = to_id_address(rt, params.from, false)?;

        let sponsor = if let Some(sponsor) = params.sponsor {
            Some(to_id_address(rt, sponsor, false)?)
        } else {
            None
        };

        rt.transaction(|st: &mut State, rt| {
            st.update_gas_allowance(
                rt.store(),
                from,
                sponsor,
                params.add_amount,
                rt.curr_epoch(),
            )
        })
    }

    /// Approve credit and gas usage from one account to another.
    ///
    /// The `from` address must be delegated (only delegated addresses can own credit).
    /// The `from` address must be the message origin or caller.
    /// The `to` address must be delegated (only delegated addresses can use credit).
    /// TODO: Remove the `caller_allowlist` parameter.
    fn approve_credit(
        rt: &impl Runtime,
        params: ApproveCreditParams,
    ) -> Result<CreditApproval, ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        let (from_id_addr, from_delegated_addr) = to_id_and_delegated_address(rt, params.from)?;
        require_addr_is_origin_or_caller(rt, from_id_addr)?;

        let config = get_config(rt)?;

        let (approval, to_delegated_addr) = match to_id_and_delegated_address(rt, params.to) {
            Ok((to_id_addr, to_delegated_addr)) => rt.transaction(|st: &mut State, rt| {
                let approval = st.approve_credit(
                    &config,
                    rt.store(),
                    from_id_addr,
                    to_id_addr,
                    rt.curr_epoch(),
                    params.credit_limit,
                    params.gas_fee_limit,
                    params.ttl,
                )?;
                Ok((approval, to_delegated_addr))
            }),
            Err(e) if e.exit_code() == ExitCode::USR_NOT_FOUND => {
                // We send zero tokens to create the account in the FVM
                extract_send_result(rt.send_simple(
                    &params.to,
                    METHOD_SEND,
                    None,
                    TokenAmount::zero(),
                ))?;
                let (to_id_addr, to_delegated_addr) = to_id_and_delegated_address(rt, params.to)?;
                let approval = rt.transaction(|st: &mut State, rt| {
                    let approval = st.approve_credit(
                        &config,
                        rt.store(),
                        from_id_addr,
                        to_id_addr,
                        rt.curr_epoch(),
                        params.credit_limit,
                        params.gas_fee_limit,
                        params.ttl,
                    );
                    st.set_account_sponsor(
                        &config,
                        rt.store(),
                        to_id_addr,
                        Some(from_id_addr),
                        rt.curr_epoch(),
                    )?;
                    approval
                })?;
                Ok((approval, to_delegated_addr))
            }
            Err(e) => Err(e),
        }?;

        emit_evm_event(
            rt,
            CreditApproved {
                from: from_delegated_addr,
                to: to_delegated_addr,
                credit_limit: approval.credit_limit.clone(),
                gas_fee_limit: approval.gas_fee_limit.clone(),
                expiry: approval.expiry,
            },
        )?;

        Ok(approval)
    }

    /// Revoke credit and gas usage from one account to another.
    ///
    /// The `from` address must be delegated (only delegated addresses can own credit).
    /// The `from` address must be the message origin or caller.
    /// The `to` address must be delegated (only delegated addresses can use credit).
    fn revoke_credit(rt: &impl Runtime, params: RevokeCreditParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        let (from_id_addr, from_delegated_addr) = to_id_and_delegated_address(rt, params.from)?;
        require_addr_is_origin_or_caller(rt, from_id_addr)?;
        let (to_id_addr, to_delegated_addr) = to_id_and_delegated_address(rt, params.to)?;

        rt.transaction(|st: &mut State, rt| {
            st.revoke_credit(rt.store(), from_id_addr, to_id_addr)
        })?;

        emit_evm_event(
            rt,
            CreditRevoked::new(from_delegated_addr, to_delegated_addr),
        )?;

        Ok(())
    }

    /// Sets or unsets a default credit and gas sponsor from one account to another.
    ///
    /// If `sponsor` does not exist, the default sponsor is unset.
    /// The `from` address must be delegated (only delegated addresses can use credit).
    /// The `from` address must be the message origin or caller.
    /// The `sponsor` address must be delegated (only delegated addresses can own credit).
    fn set_account_sponsor(rt: &impl Runtime, params: SetSponsorParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        let from = to_id_address(rt, params.from, true)?;
        require_addr_is_origin_or_caller(rt, from)?;
        let (sponsor_id_addr, sponsor_delegated_addr) = if let Some(sponsor) = params.sponsor {
            let addrs = to_id_and_delegated_address(rt, sponsor)?;
            (Some(addrs.0), Some(addrs.1))
        } else {
            (None, None)
        };

        let config = get_config(rt)?;

        rt.transaction(|st: &mut State, rt| {
            st.set_account_sponsor(&config, rt.store(), from, sponsor_id_addr, rt.curr_epoch())
        })?;

        if let Some(sponsor) = sponsor_delegated_addr {
            emit_evm_event(rt, GasSponsorSet::mew(sponsor))?;
        } else {
            emit_evm_event(rt, GasSponsorUnset::new())?;
        }

        Ok(())
    }

    /// Sets the account status for an address.
    fn set_account_status(
        rt: &impl Runtime,
        params: SetAccountStatusParams,
    ) -> Result<(), ActorError> {
        require_caller_is_admin(rt)?;

        let subscriber = to_id_address(rt, params.subscriber, true)?;

        let config = get_config(rt)?;

        rt.transaction(|st: &mut State, rt| {
            st.set_account_status(
                &config,
                rt.store(),
                subscriber,
                params.status,
                rt.curr_epoch(),
            )
        })
    }

    /// Returns the account for an address.
    ///
    /// Only delegated addresses can own or use credit, but we don't need to waste gas enforcing
    /// that condition here.
    fn get_account(
        rt: &impl Runtime,
        params: GetAccountParams,
    ) -> Result<Option<AccountInfo>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        let from = to_id_address(rt, params.0, false)?;

        let account = rt
            .state::<State>()?
            .get_account(rt.store(), from)?
            .map(|mut account| {
                // Resolve the credit sponsor
                account.credit_sponsor = account
                    .credit_sponsor
                    .map(|sponsor| to_delegated_address(rt, sponsor))
                    .transpose()?;

                AccountInfo::from(rt, account)
            });

        account.transpose()
    }

    /// Returns the credit approval from one account to another if it exists.
    ///
    /// Only delegated addresses can own or use credit, but we don't need to waste gas enforcing
    /// that condition here.
    fn get_credit_approval(
        rt: &impl Runtime,
        params: GetCreditApprovalParams,
    ) -> Result<Option<CreditApproval>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        let from = to_id_address(rt, params.from, false)?;
        let to = to_id_address(rt, params.to, false)?;

        let approval = rt
            .state::<State>()?
            .get_credit_approval(rt.store(), from, to)?;

        Ok(approval)
    }

    /// Returns the gas allowance from a credit purchase for an address.
    ///
    /// Only delegated addresses can own or use credit, but we don't need to waste gas enforcing
    /// that condition here.
    /// TODO: Gas allowance methods need unit tests.
    fn get_gas_allowance(
        rt: &impl Runtime,
        params: GetGasAllowanceParams,
    ) -> Result<GasAllowance, ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        let from = match to_id_address(rt, params.0, false) {
            Ok(from) => from,
            Err(e) => {
                return if e.exit_code() == ExitCode::USR_FORBIDDEN {
                    // Disallowed actor type (this is called by all txns so we can't error)
                    Ok(GasAllowance::default())
                } else {
                    Err(e)
                };
            }
        };

        let allowance =
            rt.state::<State>()?
                .get_gas_allowance(rt.store(), from, rt.curr_epoch())?;

        Ok(allowance)
    }

    /// Debits all accounts for current blob usage.
    ///
    /// This is called by the system actor every X blocks, where X is set in the recall config actor.
    /// TODO: Take a start key and page limit to avoid out-of-gas errors.
    fn debit_accounts(rt: &impl Runtime) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;
        let config = get_config(rt)?;
        let mut credit_debited = Credit::zero();
        let (deletes, num_accounts) = rt.transaction(|st: &mut State, rt| {
            let initial_credit_debited = st.credit_debited.clone();
            let deletes = st.debit_accounts(
                rt.store(),
                rt.curr_epoch(),
                config.blob_delete_batch_size,
                config.account_debit_batch_size,
            )?;
            credit_debited = &st.credit_debited - initial_credit_debited;
            let num_accounts = st.accounts.len();
            Ok((deletes, num_accounts))
        })?;

        for hash in deletes {
            delete_from_disc(hash)?;
        }

        // TODO: Wire more_accounts param when pagination work is done.
        emit_evm_event(
            rt,
            CreditDebited {
                amount: credit_debited,
                num_accounts,
                more_accounts: false,
            },
        )?;

        Ok(())
    }

    /// Adds or updates a blob subscription.
    ///
    /// The subscriber will only need credits for blobs that are not already covered by one of
    /// their existing subscriptions.
    ///
    /// The `sponsor` will be the subscriber (the account responsible for payment), if it exists
    /// and there is an approval from `sponsor` to the message `origin` or `caller`.    
    ///
    /// Only delegated addresses can own or use credit.
    fn add_blob(rt: &impl Runtime, params: AddBlobParams) -> Result<Subscription, ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        let (from_id_addr, from_delegated_addr) = to_id_and_delegated_address(rt, params.from)?;
        require_addr_is_origin_or_caller(rt, from_id_addr)?;
        let (subscriber_id_addr, subscriber_delegated_addr) = if let Some(sponsor) = params.sponsor
        {
            to_id_and_delegated_address(rt, sponsor)?
        } else {
            (from_id_addr, from_delegated_addr)
        };

        let tokens_received = rt.message().value_received();

        let config = get_config(rt)?;

        let mut capacity_used = 0;
        let (sub, tokens_unspent) = rt.transaction(|st: &mut State, rt| {
            let initial_capacity_used = st.capacity_used;
            let res = st.add_blob(
                &config,
                rt.store(),
                from_id_addr,
                subscriber_id_addr,
                rt.curr_epoch(),
                params.hash,
                params.metadata_hash,
                params.id,
                params.size,
                params.ttl,
                params.source,
                tokens_received,
            )?;
            capacity_used = st.capacity_used - initial_capacity_used;
            Ok(res)
        })?;

        // Send back unspent tokens
        if !tokens_unspent.is_zero() {
            extract_send_result(rt.send_simple(&from_id_addr, METHOD_SEND, None, tokens_unspent))?;
        }

        emit_evm_event(
            rt,
            sol_blobs::BlobAdded {
                subscriber: subscriber_delegated_addr,
                hash: &params.hash,
                size: params.size,
                expiry: sub.expiry,
                bytes_used: capacity_used,
            },
        )?;

        Ok(sub)
    }

    /// Returns a blob by [`Hash`] if it exists.
    fn get_blob(rt: &impl Runtime, params: GetBlobParams) -> Result<Option<BlobInfo>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        match rt.state::<State>()?.get_blob(rt.store(), params.0)? {
            Some(blob) => Ok(Some(BlobInfo::from(rt, blob)?)),
            None => Ok(None),
        }
    }

    /// Returns the current [`BlobStatus`] for a blob by [`Hash`].
    fn get_blob_status(
        rt: &impl Runtime,
        params: GetBlobStatusParams,
    ) -> Result<Option<BlobStatus>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let subscriber = to_id_address(rt, params.subscriber, false)?;
        rt.state::<State>()?
            .get_blob_status(rt.store(), subscriber, params.hash, params.id)
    }

    /// Returns a list of [`BlobRequest`]s that are currenlty in the [`BlobStatus::Added`] state.
    ///
    /// All blobs that have been added but have not yet been picked up by validators for download
    /// are in the [`BlobStatus::Added`] state.
    fn get_added_blobs(
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
    fn get_pending_blobs(
        rt: &impl Runtime,
        params: GetPendingBlobsParams,
    ) -> Result<Vec<BlobRequest>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        rt.state::<State>()?.get_pending_blobs(rt.store(), params.0)
    }

    /// Sets a blob to the [`BlobStatus::Pending`] state.
    fn set_blob_pending(rt: &impl Runtime, params: SetBlobPendingParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        let (subscriber_id_addr, subscriber_delegated_addr) =
            to_id_and_delegated_address(rt, params.subscriber)?;

        rt.transaction(|st: &mut State, rt| {
            st.set_blob_pending(
                rt.store(),
                subscriber_id_addr,
                params.hash,
                params.size,
                params.id,
                params.source,
            )
        })?;

        emit_evm_event(
            rt,
            sol_blobs::BlobPending {
                subscriber: subscriber_delegated_addr,
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
    fn finalize_blob(rt: &impl Runtime, params: FinalizeBlobParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        let (subscriber_id_addr, subscriber_delegated_addr) =
            to_id_and_delegated_address(rt, params.subscriber)?;
        let event_resolved = matches!(params.status, BlobStatus::Resolved);

        let config = get_config(rt)?;

        rt.transaction(|st: &mut State, rt| {
            st.finalize_blob(
                &config,
                rt.store(),
                subscriber_id_addr,
                rt.curr_epoch(),
                params.hash,
                params.id,
                params.status,
            )
        })?;

        emit_evm_event(
            rt,
            sol_blobs::BlobFinalized {
                subscriber: subscriber_delegated_addr,
                hash: &params.hash,
                resolved: event_resolved,
            },
        )
    }

    /// Deletes a blob subscription.
    ///
    /// The `sponsor` will be the subscriber (the account responsible for payment), if it exists
    /// and there is an approval from `sponsor` to the message `origin` or `caller`.    
    ///
    /// Only delegated addresses can own or use credit.
    fn delete_blob(rt: &impl Runtime, params: DeleteBlobParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        let (from_id_addr, from_delegated_addr) = to_id_and_delegated_address(rt, params.from)?;
        require_addr_is_origin_or_caller(rt, from_id_addr)?;
        let (subscriber_id_addr, subscriber_delegated_addr) = if let Some(sponsor) = params.sponsor
        {
            to_id_and_delegated_address(rt, sponsor)?
        } else {
            (from_id_addr, from_delegated_addr)
        };

        let mut capacity_released = 0;
        let (delete, size) = rt.transaction(|st: &mut State, rt| {
            let initial_capacity_used = st.capacity_used;
            let res = st.delete_blob(
                rt.store(),
                from_id_addr,
                subscriber_id_addr,
                rt.curr_epoch(),
                params.hash,
                params.id,
            )?;
            capacity_released = initial_capacity_used - st.capacity_used;
            Ok(res)
        })?;

        if delete {
            delete_from_disc(params.hash)?;
        }

        emit_evm_event(
            rt,
            sol_blobs::BlobDeleted {
                subscriber: subscriber_delegated_addr,
                hash: &params.hash,
                size,
                bytes_released: capacity_released,
            },
        )?;

        Ok(())
    }

    /// Deletes a blob subscription and adds another in a sinlge call.
    ///
    /// This method is more efficient than two separate calls to `delete_blob` and `add_blob`,
    /// and is useful for some blob workflows like a replacing a key in a bucket actor.
    ///
    /// The `sponsor` will be the subscriber (the account responsible for payment), if it exists
    /// and there is an approval from `sponsor` to the message `origin` or `caller`.    
    ///
    /// Only delegated addresses can own or use credit.
    fn overwrite_blob(
        rt: &impl Runtime,
        params: OverwriteBlobParams,
    ) -> Result<Subscription, ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        let (from_id_addr, from_delegated_addr) = to_id_and_delegated_address(rt, params.add.from)?;
        require_addr_is_origin_or_caller(rt, from_id_addr)?;
        let (subscriber_id_addr, subscriber_delegated_addr) =
            if let Some(sponsor) = params.add.sponsor {
                to_id_and_delegated_address(rt, sponsor)?
            } else {
                (from_id_addr, from_delegated_addr)
            };

        let config = get_config(rt)?;

        // Determine if we need to delete an existing blob before adding the new one
        let overwrite = params.old_hash != params.add.hash;

        let add_hash = params.add.hash;
        let add_size = params.add.size;
        let mut capacity_released = 0;
        let mut capacity_used = 0;

        // To ensure atomicity, we combine the two independent calls into a single transaction.
        let (delete, delete_size, sub) = rt.transaction(|st: &mut State, rt| {
            let add_params = params.add;

            let initial_capacity_used = st.capacity_used;
            let (delete, delete_size) = if overwrite {
                st.delete_blob(
                    rt.store(),
                    from_id_addr,
                    subscriber_id_addr,
                    rt.curr_epoch(),
                    params.old_hash,
                    add_params.id.clone(),
                )?
            } else {
                (false, 0)
            };
            capacity_released = initial_capacity_used - st.capacity_used;

            let initial_capacity_used = st.capacity_used;
            let (subscription, _) = st.add_blob(
                &config,
                rt.store(),
                from_id_addr,
                subscriber_id_addr,
                rt.curr_epoch(),
                add_params.hash,
                add_params.metadata_hash,
                add_params.id,
                add_params.size,
                add_params.ttl,
                add_params.source,
                TokenAmount::zero(),
            )?;
            capacity_used = st.capacity_used - initial_capacity_used;

            Ok((delete, delete_size, subscription))
        })?;

        if delete {
            delete_from_disc(params.old_hash)?;
        }

        if overwrite {
            emit_evm_event(
                rt,
                sol_blobs::BlobDeleted {
                    subscriber: subscriber_delegated_addr,
                    hash: &params.old_hash,
                    size: delete_size,
                    bytes_released: capacity_released,
                },
            )?;
        }
        emit_evm_event(
            rt,
            sol_blobs::BlobAdded {
                subscriber: subscriber_delegated_addr,
                hash: &add_hash,
                size: add_size,
                expiry: sub.expiry,
                bytes_used: capacity_used,
            },
        )?;

        Ok(sub)
    }

    /// Trims the subscription expiries for an account based on its current maximum allowed blob TTL.
    ///
    /// This is used in conjunction with `set_account_status` when reducing an account's maximum
    /// allowed blob TTL.
    /// Returns the number of subscriptions processed and the next key to continue iteration.
    fn trim_blob_expiries(
        rt: &impl Runtime,
        params: TrimBlobExpiriesParams,
    ) -> Result<(u32, Option<Hash>), ActorError> {
        require_caller_is_admin(rt)?;

        let subscriber = to_id_address(rt, params.subscriber, true)?;

        let config = get_config(rt)?;

        let (processed, next_key, deleted_blobs) = rt.transaction(|st: &mut State, rt| {
            st.trim_blob_expiries(
                &config,
                rt.store(),
                subscriber,
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

    fn invoke_contract(
        rt: &impl Runtime,
        params: InvokeContractParams,
    ) -> Result<InvokeContractReturn, ActorError> {
        let input_data: InputData = params.try_into()?;
        if sol_blobs::can_handle(&input_data) {
            let output_data = match sol_blobs::parse_input(&input_data)? {
                sol_blobs::Calls::addBlob(call) => {
                    let params = call.params(rt)?;
                    Self::add_blob(rt, params)?;
                    call.returns(())
                }
                sol_blobs::Calls::deleteBlob(call) => {
                    let params = call.params(rt)?;
                    Self::delete_blob(rt, params)?;
                    call.returns(())
                }
                sol_blobs::Calls::getBlob(call) => {
                    let params: GetBlobParams = call.params()?;
                    let blob = Self::get_blob(rt, params)?;
                    call.returns(blob)?
                }
                sol_blobs::Calls::getStats(call) => {
                    let stats = Self::get_stats(rt)?;
                    call.returns(stats)
                }
                sol_blobs::Calls::overwriteBlob(call) => {
                    let params = call.params(rt)?;
                    Self::overwrite_blob(rt, params)?;
                    call.returns(())
                }
                sol_blobs::Calls::trimBlobExpiries(call) => {
                    let params = call.params();
                    let cursor = Self::trim_blob_expiries(rt, params)?;
                    call.returns(cursor)
                }
            };
            Ok(InvokeContractReturn { output_data })
        } else if sol_credit::can_handle(&input_data) {
            let output_data = match sol_credit::parse_input(&input_data)? {
                sol_credit::Calls::buyCredit_0(call) => {
                    // function buyCredit() external payable;
                    let params = call.params(rt);
                    Self::buy_credit(rt, params)?;
                    call.returns(())
                }
                sol_credit::Calls::buyCredit_1(call) => {
                    // function buyCredit(address recipient) external payable;
                    let params = call.params();
                    Self::buy_credit(rt, params)?;
                    call.returns(())
                }
                sol_credit::Calls::approveCredit_0(call) => {
                    let params = call.params(rt);
                    Self::approve_credit(rt, params)?;
                    call.returns(())
                }
                sol_credit::Calls::approveCredit_1(call) => {
                    let params = call.params(rt);
                    Self::approve_credit(rt, params)?;
                    call.returns(())
                }
                sol_credit::Calls::approveCredit_2(call) => {
                    let params = call.params(rt);
                    Self::approve_credit(rt, params)?;
                    call.returns(())
                }
                sol_credit::Calls::revokeCredit_0(call) => {
                    let params = call.params(rt);
                    Self::revoke_credit(rt, params)?;
                    call.returns(())
                }
                sol_credit::Calls::revokeCredit_1(call) => {
                    let params = call.params(rt);
                    Self::revoke_credit(rt, params)?;
                    call.returns(())
                }
                sol_credit::Calls::setAccountSponsor(call) => {
                    let params = call.params(rt);
                    Self::set_account_sponsor(rt, params)?;
                    call.returns(())
                }
                sol_credit::Calls::getAccount(call) => {
                    let params = call.params();
                    let account_info = Self::get_account(rt, params)?;
                    call.returns(account_info)?
                }
                sol_credit::Calls::getCreditApproval(call) => {
                    let params = call.params();
                    let credit_approval = Self::get_credit_approval(rt, params)?;
                    call.returns(credit_approval)
                }
                sol_credit::Calls::setAccountStatus(call) => {
                    let params = call.params()?;
                    Self::set_account_status(rt, params)?;
                    call.returns(())
                }
            };
            Ok(InvokeContractReturn { output_data })
        } else {
            Err(actor_error!(illegal_argument, "invalid call".to_string()))
        }
    }

    /// Fallback method for unimplemented method numbers.
    pub fn fallback(
        rt: &impl Runtime,
        method: MethodNum,
        _: Option<IpldBlock>,
    ) -> Result<Option<IpldBlock>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        if method >= FIRST_EXPORTED_METHOD_NUMBER {
            Ok(None)
        } else {
            Err(actor_error!(unhandled_message; "invalid method: {}", method))
        }
    }
}

/// Makes a syscall that will delete a blob from the underlying Iroh-based data store.
fn delete_from_disc(hash: Hash) -> Result<(), ActorError> {
    #[cfg(feature = "fil-actor")]
    {
        recall_actor_sdk::hash_rm(hash.0).map_err(|en| {
            ActorError::unspecified(format!("failed to delete blob from disc: {:?}", en))
        })?;
        log::debug!("deleted blob {} from disc", hash);
        Ok(())
    }
    #[cfg(not(feature = "fil-actor"))]
    {
        log::debug!("mock deletion from disc (hash={})", hash);
        Ok(())
    }
}

impl ActorCode for BlobsActor {
    type Methods = Method;

    fn name() -> &'static str {
        BLOBS_ACTOR_NAME
    }

    actor_dispatch! {
        Constructor => constructor,

        // User methods
        BuyCredit => buy_credit,
        ApproveCredit => approve_credit,
        RevokeCredit => revoke_credit,
        SetAccountSponsor => set_account_sponsor,
        GetAccount => get_account,
        GetCreditApproval => get_credit_approval,
        AddBlob => add_blob,
        GetBlob => get_blob,
        DeleteBlob => delete_blob,
        OverwriteBlob => overwrite_blob,

        // System methods
        GetGasAllowance => get_gas_allowance,
        UpdateGasAllowance => update_gas_allowance,
        GetBlobStatus => get_blob_status,
        GetAddedBlobs => get_added_blobs,
        GetPendingBlobs => get_pending_blobs,
        SetBlobPending => set_blob_pending,
        FinalizeBlob => finalize_blob,
        DebitAccounts => debit_accounts,

        // Admin methods
        SetAccountStatus => set_account_status,
        TrimBlobExpiries => trim_blob_expiries,

        // Metrics methods
        GetStats => get_stats,
        // EVM interop
        InvokeContract => invoke_contract,
        _ => fallback,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fendermint_actor_blobs_shared::state::SubscriptionId;
    use fendermint_actor_blobs_testing::{new_hash, new_pk};
    use fendermint_actor_recall_config_shared::{RecallConfig, RECALL_CONFIG_ACTOR_ADDR};
    use fil_actors_evm_shared::address::EthAddress;
    use fil_actors_runtime::test_utils::{
        expect_empty, MockRuntime, ETHACCOUNT_ACTOR_CODE_ID, EVM_ACTOR_CODE_ID,
        SYSTEM_ACTOR_CODE_ID,
    };
    use fvm_shared::address::Address;
    use fvm_shared::{bigint::BigInt, clock::ChainEpoch, sys::SendFlags};
    use recall_actor_sdk::to_actor_event;

    pub fn construct_and_verify() -> MockRuntime {
        let rt = MockRuntime {
            receiver: Address::new_id(10),
            ..Default::default()
        };
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);
        let result = rt
            .call::<BlobsActor>(Method::Constructor as u64, None)
            .unwrap();
        expect_empty(result);
        rt.verify();
        rt.reset();
        rt
    }

    fn expect_get_config(rt: &MockRuntime) {
        rt.expect_send(
            RECALL_CONFIG_ACTOR_ADDR,
            fendermint_actor_recall_config_shared::Method::GetConfig as MethodNum,
            None,
            TokenAmount::zero(),
            None,
            SendFlags::READ_ONLY,
            IpldBlock::serialize_cbor(&RecallConfig::default()).unwrap(),
            ExitCode::OK,
            None,
        );
    }

    fn expect_emitted_purchase_event(
        rt: &MockRuntime,
        params: &BuyCreditParams,
        amount: TokenAmount,
    ) {
        let event = to_actor_event(CreditPurchased::new(params.0, amount)).unwrap();
        rt.expect_emitted_event(event);
    }

    fn expect_emitted_approve_event(
        rt: &MockRuntime,
        from: Address,
        to: Address,
        credit_limit: Option<TokenAmount>,
        gas_fee_limit: Option<TokenAmount>,
        expiry: ChainEpoch,
    ) {
        let event = to_actor_event(CreditApproved {
            from,
            to,
            credit_limit,
            gas_fee_limit,
            expiry: Some(expiry),
        })
        .unwrap();
        rt.expect_emitted_event(event);
    }

    fn expect_emitted_revoke_event(rt: &MockRuntime, from: Address, to: Address) {
        let event = to_actor_event(CreditRevoked::new(from, to)).unwrap();
        rt.expect_emitted_event(event);
    }

    fn expect_emitted_add_event(
        rt: &MockRuntime,
        current_epoch: ChainEpoch,
        params: &AddBlobParams,
        subscriber: Address,
        used: u64,
    ) {
        let event = to_actor_event(sol_blobs::BlobAdded {
            subscriber,
            hash: &params.hash,
            size: params.size,
            expiry: params.ttl.unwrap_or(86400) + current_epoch,
            bytes_used: used,
        })
        .unwrap();
        rt.expect_emitted_event(event);
    }

    #[test]
    fn test_buy_credit() {
        let rt = construct_and_verify();

        // TODO(bcalza): Choose a rate different than default
        let token_credit_rate = BigInt::from(1000000000000000000u64);

        let id_addr = Address::new_id(110);
        let eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000000"
        ));
        let f4_eth_addr = Address::new_delegated(10, &eth_addr.0).unwrap();

        rt.set_delegated_address(id_addr.id().unwrap(), f4_eth_addr);
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, id_addr);
        rt.set_origin(id_addr);

        let tokens = 1;
        let mut expected_credits =
            Credit::from_atto(1000000000000000000u64 * tokens * &token_credit_rate);
        let mut expected_gas_allowance = TokenAmount::from_whole(tokens);
        rt.set_received(TokenAmount::from_whole(tokens));
        rt.expect_validate_caller_any();
        let fund_params = BuyCreditParams(f4_eth_addr);
        expect_get_config(&rt);
        expect_emitted_purchase_event(&rt, &fund_params, expected_credits.clone());
        let result = rt
            .call::<BlobsActor>(
                Method::BuyCredit as u64,
                IpldBlock::serialize_cbor(&fund_params).unwrap(),
            )
            .unwrap()
            .unwrap()
            .deserialize::<AccountInfo>()
            .unwrap();
        assert_eq!(result.credit_free, expected_credits);
        assert_eq!(result.gas_allowance, expected_gas_allowance);
        rt.verify();

        let additional_credits = Credit::from_atto(1000000000u64 * tokens * &token_credit_rate);
        expected_credits += &additional_credits;
        expected_gas_allowance += TokenAmount::from_nano(tokens);
        rt.set_received(TokenAmount::from_nano(tokens));
        rt.expect_validate_caller_any();
        let fund_params = BuyCreditParams(f4_eth_addr);
        expect_get_config(&rt);
        expect_emitted_purchase_event(&rt, &fund_params, additional_credits);
        let result = rt
            .call::<BlobsActor>(
                Method::BuyCredit as u64,
                IpldBlock::serialize_cbor(&fund_params).unwrap(),
            )
            .unwrap()
            .unwrap()
            .deserialize::<AccountInfo>()
            .unwrap();
        assert_eq!(result.credit_free, expected_credits);
        assert_eq!(result.gas_allowance, expected_gas_allowance);
        rt.verify();

        let additional_credits = Credit::from_atto(tokens * &token_credit_rate);
        expected_credits += &additional_credits;
        expected_gas_allowance += TokenAmount::from_atto(tokens);
        rt.set_received(TokenAmount::from_atto(tokens));
        rt.expect_validate_caller_any();
        let fund_params = BuyCreditParams(f4_eth_addr);
        expect_get_config(&rt);
        expect_emitted_purchase_event(&rt, &fund_params, additional_credits);
        let result = rt
            .call::<BlobsActor>(
                Method::BuyCredit as u64,
                IpldBlock::serialize_cbor(&fund_params).unwrap(),
            )
            .unwrap()
            .unwrap()
            .deserialize::<AccountInfo>()
            .unwrap();
        assert_eq!(result.credit_free, expected_credits);
        assert_eq!(result.gas_allowance, expected_gas_allowance);
        rt.verify();
    }

    #[test]
    fn test_approve_credit() {
        let rt = construct_and_verify();

        // Credit owner
        let owner_id_addr = Address::new_id(110);
        let owner_eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000000"
        ));
        let owner_f4_eth_addr = Address::new_delegated(10, &owner_eth_addr.0).unwrap();
        rt.set_delegated_address(owner_id_addr.id().unwrap(), owner_f4_eth_addr);

        // Credit receiver
        let to_id_addr = Address::new_id(111);
        let to_eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000001"
        ));
        let to_f4_eth_addr = Address::new_delegated(10, &to_eth_addr.0).unwrap();
        rt.set_delegated_address(to_id_addr.id().unwrap(), to_f4_eth_addr);
        rt.set_address_actor_type(to_id_addr, *ETHACCOUNT_ACTOR_CODE_ID);

        // Proxy EVM contract on behalf of the credit owner
        let proxy_id_addr = Address::new_id(112);
        let proxy_eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000002"
        ));
        let proxy_f4_eth_addr = Address::new_delegated(10, &proxy_eth_addr.0).unwrap();
        rt.set_delegated_address(proxy_id_addr.id().unwrap(), proxy_f4_eth_addr);
        rt.set_address_actor_type(proxy_id_addr, *EVM_ACTOR_CODE_ID);

        // Caller/origin is the same as from (i.e., the standard case)
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, owner_id_addr);
        rt.set_origin(owner_id_addr);
        rt.expect_validate_caller_any();
        expect_get_config(&rt);
        let approve_params = ApproveCreditParams {
            from: owner_id_addr,
            to: to_id_addr,
            caller_allowlist: None,
            credit_limit: None,
            gas_fee_limit: None,
            ttl: None,
        };
        expect_emitted_approve_event(
            &rt,
            owner_f4_eth_addr,
            to_f4_eth_addr,
            approve_params.credit_limit.clone(),
            approve_params.gas_fee_limit.clone(),
            0,
        );
        let result = rt.call::<BlobsActor>(
            Method::ApproveCredit as u64,
            IpldBlock::serialize_cbor(&approve_params).unwrap(),
        );
        assert!(result.is_ok());
        rt.verify();

        // Proxy caller (caller mismatch with from, but is correct origin)
        rt.set_caller(*EVM_ACTOR_CODE_ID, proxy_id_addr);
        rt.set_origin(owner_id_addr);
        rt.expect_validate_caller_any();
        expect_get_config(&rt);
        let approve_params = ApproveCreditParams {
            from: owner_id_addr,
            to: to_id_addr,
            caller_allowlist: None,
            credit_limit: None,
            gas_fee_limit: None,
            ttl: None,
        };
        expect_emitted_approve_event(
            &rt,
            owner_f4_eth_addr,
            to_f4_eth_addr,
            approve_params.credit_limit.clone(),
            approve_params.gas_fee_limit.clone(),
            0,
        );
        let result = rt.call::<BlobsActor>(
            Method::ApproveCredit as u64,
            IpldBlock::serialize_cbor(&approve_params).unwrap(),
        );
        assert!(result.is_ok());
        rt.verify();

        // Caller/origin mismatch with from
        rt.set_caller(*EVM_ACTOR_CODE_ID, proxy_id_addr);
        rt.set_origin(owner_id_addr);
        rt.expect_validate_caller_any();
        let approve_params = ApproveCreditParams {
            from: to_id_addr, // mismatch
            to: to_id_addr,
            caller_allowlist: None,
            credit_limit: None,
            gas_fee_limit: None,
            ttl: None,
        };
        let result = rt.call::<BlobsActor>(
            Method::ApproveCredit as u64,
            IpldBlock::serialize_cbor(&approve_params).unwrap(),
        );
        let expected_return = Err(ActorError::illegal_argument(format!(
            "address {} does not match origin or caller",
            to_id_addr
        )));
        assert_eq!(result, expected_return);
        rt.verify();
    }

    #[test]
    fn test_approve_credit_to_new_account() {
        let rt = construct_and_verify();

        // Credit owner
        let owner_id_addr = Address::new_id(110);
        let owner_eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000000"
        ));
        let owner_f4_eth_addr = Address::new_delegated(10, &owner_eth_addr.0).unwrap();
        rt.set_delegated_address(owner_id_addr.id().unwrap(), owner_f4_eth_addr);
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, owner_id_addr);
        rt.set_origin(owner_id_addr);

        // Use a new receiver that doesn't exist in the FVM
        let receiver_eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000001"
        ));
        let receiver_f4_eth_addr = Address::new_delegated(10, &receiver_eth_addr.0).unwrap();

        rt.expect_validate_caller_any();
        expect_get_config(&rt);
        rt.expect_send_simple(
            receiver_f4_eth_addr,
            METHOD_SEND,
            None,
            TokenAmount::zero(),
            None,
            ExitCode::OK,
        );
        let approve_params = ApproveCreditParams {
            from: owner_f4_eth_addr,
            to: receiver_f4_eth_addr, // Use the external address to force the ID lookup to fail
            caller_allowlist: None,
            credit_limit: None,
            gas_fee_limit: None,
            ttl: None,
        };
        let result = rt.call::<BlobsActor>(
            Method::ApproveCredit as u64,
            IpldBlock::serialize_cbor(&approve_params).unwrap(),
        );
        // This test should pass, but in the mock runtime, sending token to an address does not
        // create the actor, like it does in the real FVM runtime.
        // The result is that the second call to to_id_address in the approve_credit method still
        // fails after the call to send with a "not found" error.
        // However, we are able to test that the call to send did happen using
        // rt.expect_send_simple above.
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().exit_code(), ExitCode::USR_NOT_FOUND);
        rt.verify();
    }

    #[test]
    fn test_revoke_credit() {
        let rt = construct_and_verify();

        // Credit owner
        let owner_id_addr = Address::new_id(110);
        let owner_eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000000"
        ));
        let owner_f4_eth_addr = Address::new_delegated(10, &owner_eth_addr.0).unwrap();
        rt.set_delegated_address(owner_id_addr.id().unwrap(), owner_f4_eth_addr);

        // Credit receiver
        let to_id_addr = Address::new_id(111);
        let to_eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000001"
        ));
        let to_f4_eth_addr = Address::new_delegated(10, &to_eth_addr.0).unwrap();
        rt.set_delegated_address(to_id_addr.id().unwrap(), to_f4_eth_addr);
        rt.set_address_actor_type(to_id_addr, *ETHACCOUNT_ACTOR_CODE_ID);

        // Proxy EVM contract on behalf of the credit owner
        let proxy_id_addr = Address::new_id(112);
        let proxy_eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000002"
        ));
        let proxy_f4_eth_addr = Address::new_delegated(10, &proxy_eth_addr.0).unwrap();
        rt.set_delegated_address(proxy_id_addr.id().unwrap(), proxy_f4_eth_addr);
        rt.set_address_actor_type(proxy_id_addr, *EVM_ACTOR_CODE_ID);

        // Set up the approval to revoke
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, owner_id_addr);
        rt.set_origin(owner_id_addr);
        rt.expect_validate_caller_any();
        expect_get_config(&rt);
        let approve_params = ApproveCreditParams {
            from: owner_id_addr,
            to: to_id_addr,
            caller_allowlist: None,
            credit_limit: None,
            gas_fee_limit: None,
            ttl: None,
        };
        expect_emitted_approve_event(
            &rt,
            owner_f4_eth_addr,
            to_f4_eth_addr,
            approve_params.credit_limit.clone(),
            approve_params.gas_fee_limit.clone(),
            0,
        );
        let result = rt.call::<BlobsActor>(
            Method::ApproveCredit as u64,
            IpldBlock::serialize_cbor(&approve_params).unwrap(),
        );
        assert!(result.is_ok());
        rt.verify();

        // Caller/origin is the same as from (i.e., the standard case)
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, owner_id_addr);
        rt.set_origin(owner_id_addr);
        rt.expect_validate_caller_any();
        let revoke_params = RevokeCreditParams {
            from: owner_id_addr,
            to: to_id_addr,
            for_caller: None,
        };
        expect_emitted_revoke_event(&rt, owner_f4_eth_addr, to_f4_eth_addr);
        let result = rt.call::<BlobsActor>(
            Method::RevokeCredit as u64,
            IpldBlock::serialize_cbor(&revoke_params).unwrap(),
        );
        assert!(result.is_ok());
        rt.verify();

        // Proxy caller (caller mismatch with from, but is correct origin)
        rt.set_caller(*EVM_ACTOR_CODE_ID, proxy_id_addr);
        rt.set_origin(owner_id_addr);
        rt.expect_validate_caller_any();
        let revoke_params = RevokeCreditParams {
            from: owner_id_addr,
            to: to_id_addr,
            for_caller: None,
        };
        let result = rt.call::<BlobsActor>(
            Method::RevokeCredit as u64,
            IpldBlock::serialize_cbor(&revoke_params).unwrap(),
        );
        // This should be a state error, not from the actor API
        assert!(result.is_err());
        assert!(result.err().unwrap().msg().contains("not found"),);
        rt.verify();

        // Caller/origin mismatch with from
        rt.set_caller(*EVM_ACTOR_CODE_ID, proxy_id_addr);
        rt.set_origin(owner_id_addr);
        rt.expect_validate_caller_any();
        let revoke_params = RevokeCreditParams {
            from: to_id_addr, // mismatch
            to: to_id_addr,
            for_caller: None,
        };
        let result = rt.call::<BlobsActor>(
            Method::RevokeCredit as u64,
            IpldBlock::serialize_cbor(&revoke_params).unwrap(),
        );
        let expected_return = Err(ActorError::illegal_argument(format!(
            "address {} does not match origin or caller",
            to_id_addr
        )));
        assert_eq!(result, expected_return);
        rt.verify();
    }

    #[test]
    fn test_add_blob() {
        let rt = construct_and_verify();

        let token_credit_rate = BigInt::from(1000000000000000000u64);

        let id_addr = Address::new_id(110);
        let eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000000"
        ));
        let f4_eth_addr = Address::new_delegated(10, &eth_addr.0).unwrap();

        rt.set_delegated_address(id_addr.id().unwrap(), f4_eth_addr);
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, id_addr);
        rt.set_origin(id_addr);
        rt.set_epoch(ChainEpoch::from(0));

        // Try without first funding
        rt.expect_validate_caller_any();
        let hash = new_hash(1024);
        let add_params = AddBlobParams {
            sponsor: None,
            source: new_pk(),
            hash: hash.0,
            metadata_hash: new_hash(1024).0,
            id: SubscriptionId::default(),
            size: hash.1,
            ttl: Some(3600),
            from: id_addr,
        };
        expect_get_config(&rt);
        let result = rt.call::<BlobsActor>(
            Method::AddBlob as u64,
            IpldBlock::serialize_cbor(&add_params).unwrap(),
        );
        assert!(result.is_err());
        rt.verify();

        // Fund an account
        let tokens = 1;
        let received = TokenAmount::from_whole(tokens);
        let expected_credits =
            Credit::from_atto(1000000000000000000u64 * tokens * &token_credit_rate);
        rt.set_received(received.clone());
        rt.expect_validate_caller_any();
        let fund_params = BuyCreditParams(f4_eth_addr);
        expect_get_config(&rt);
        expect_emitted_purchase_event(&rt, &fund_params, expected_credits);
        let result = rt.call::<BlobsActor>(
            Method::BuyCredit as u64,
            IpldBlock::serialize_cbor(&fund_params).unwrap(),
        );
        assert!(result.is_ok());
        rt.verify();

        // Try with sufficient balance
        rt.set_epoch(ChainEpoch::from(5));
        rt.expect_validate_caller_any();
        expect_get_config(&rt);
        expect_emitted_add_event(&rt, 5, &add_params, f4_eth_addr, add_params.size);
        let subscription = rt
            .call::<BlobsActor>(
                Method::AddBlob as u64,
                IpldBlock::serialize_cbor(&add_params).unwrap(),
            )
            .unwrap()
            .unwrap()
            .deserialize::<Subscription>()
            .unwrap();
        assert_eq!(subscription.added, 5);
        assert_eq!(subscription.expiry, 3605);
        assert_eq!(subscription.delegate, None);
        rt.verify();

        // Get it back
        rt.expect_validate_caller_any();
        let get_params = GetBlobParams(hash.0);
        let blob = rt
            .call::<BlobsActor>(
                Method::GetBlob as u64,
                IpldBlock::serialize_cbor(&get_params).unwrap(),
            )
            .unwrap()
            .unwrap()
            .deserialize::<Option<BlobInfo>>()
            .unwrap();
        assert!(blob.is_some());
        let blob = blob.unwrap();
        assert_eq!(blob.size, add_params.size);
        assert_eq!(blob.metadata_hash, add_params.metadata_hash);
        assert_eq!(blob.subscribers.len(), 1);
        assert_eq!(blob.status, BlobStatus::Added);
    }

    #[test]
    fn test_add_blob_inline_buy() {
        let rt = construct_and_verify();

        let id_addr = Address::new_id(110);
        let eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000000"
        ));
        let f4_eth_addr = Address::new_delegated(10, &eth_addr.0).unwrap();

        rt.set_delegated_address(id_addr.id().unwrap(), f4_eth_addr);
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, id_addr);
        rt.set_origin(id_addr);
        rt.set_epoch(ChainEpoch::from(0));

        // Try sending a lot
        rt.expect_validate_caller_any();
        let hash = new_hash(1024);
        let add_params = AddBlobParams {
            sponsor: None,
            source: new_pk(),
            hash: hash.0,
            metadata_hash: new_hash(1024).0,
            id: SubscriptionId::default(),
            size: hash.1,
            ttl: Some(3600),
            from: id_addr,
        };
        let tokens_sent = TokenAmount::from_whole(1);
        rt.set_received(tokens_sent.clone());
        rt.set_balance(tokens_sent.clone());
        let tokens_required_atto = add_params.size * add_params.ttl.unwrap() as u64;
        let expected_tokens_unspent = tokens_sent.atto() - tokens_required_atto;
        expect_get_config(&rt);
        expect_emitted_add_event(&rt, 0, &add_params, f4_eth_addr, add_params.size);
        rt.expect_send_simple(
            id_addr,
            METHOD_SEND,
            None,
            TokenAmount::from_atto(expected_tokens_unspent),
            None,
            ExitCode::OK,
        );
        let result = rt.call::<BlobsActor>(
            Method::AddBlob as u64,
            IpldBlock::serialize_cbor(&add_params).unwrap(),
        );
        assert!(result.is_ok());
        rt.verify();

        // Try sending zero
        rt.expect_validate_caller_any();
        rt.set_received(TokenAmount::zero());
        let hash = new_hash(1024);
        let add_params = AddBlobParams {
            sponsor: None,
            hash: hash.0,
            metadata_hash: new_hash(1024).0,
            source: new_pk(),
            id: SubscriptionId::default(),
            size: hash.1,
            ttl: Some(3600),
            from: id_addr,
        };
        expect_get_config(&rt);
        let response = rt.call::<BlobsActor>(
            Method::AddBlob as u64,
            IpldBlock::serialize_cbor(&add_params).unwrap(),
        );
        assert!(response.is_err());
        rt.verify();

        // Try sending exact amount
        let tokens_required_atto = add_params.size * add_params.ttl.unwrap() as u64;
        let tokens_sent = TokenAmount::from_atto(tokens_required_atto);
        rt.set_received(tokens_sent.clone());
        rt.expect_validate_caller_any();
        let hash = new_hash(1024);
        let add_params = AddBlobParams {
            sponsor: None,
            hash: hash.0,
            metadata_hash: new_hash(1024).0,
            source: new_pk(),
            id: SubscriptionId::default(),
            size: hash.1,
            ttl: Some(3600),
            from: id_addr,
        };
        expect_get_config(&rt);
        expect_emitted_add_event(&rt, 0, &add_params, f4_eth_addr, add_params.size);
        let result = rt.call::<BlobsActor>(
            Method::AddBlob as u64,
            IpldBlock::serialize_cbor(&add_params).unwrap(),
        );
        assert!(result.is_ok());
        rt.verify();
    }

    #[test]
    fn test_add_blob_with_sponsor() {
        let rt = construct_and_verify();

        let token_credit_rate = BigInt::from(1000000000000000000u64);

        // Credit sponsor
        let sponsor_id_addr = Address::new_id(110);
        let sponsor_eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000000"
        ));
        let sponsor_f4_eth_addr = Address::new_delegated(10, &sponsor_eth_addr.0).unwrap();
        rt.set_delegated_address(sponsor_id_addr.id().unwrap(), sponsor_f4_eth_addr);

        // Credit spender
        let spender_id_addr = Address::new_id(111);
        let spender_eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000001"
        ));
        let spender_f4_eth_addr = Address::new_delegated(10, &spender_eth_addr.0).unwrap();
        rt.set_delegated_address(spender_id_addr.id().unwrap(), spender_f4_eth_addr);
        rt.set_address_actor_type(spender_id_addr, *ETHACCOUNT_ACTOR_CODE_ID);

        // Proxy EVM contract on behalf of the credit owner
        let proxy_id_addr = Address::new_id(112);
        let proxy_eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000002"
        ));
        let proxy_f4_eth_addr = Address::new_delegated(10, &proxy_eth_addr.0).unwrap();
        rt.set_delegated_address(proxy_id_addr.id().unwrap(), proxy_f4_eth_addr);
        rt.set_address_actor_type(proxy_id_addr, *EVM_ACTOR_CODE_ID);

        // Sponsor buys credit
        let tokens = 1;
        let received = TokenAmount::from_whole(tokens);
        let expected_credits =
            Credit::from_atto(1000000000000000000u64 * tokens * &token_credit_rate);
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, sponsor_id_addr);
        rt.set_received(received);
        rt.expect_validate_caller_any();
        let fund_params = BuyCreditParams(sponsor_f4_eth_addr);
        expect_get_config(&rt);
        expect_emitted_purchase_event(&rt, &fund_params, expected_credits);
        let response = rt.call::<BlobsActor>(
            Method::BuyCredit as u64,
            IpldBlock::serialize_cbor(&fund_params).unwrap(),
        );
        assert!(response.is_ok());
        rt.verify();

        // Sponsors approves credit
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, sponsor_id_addr);
        rt.set_origin(sponsor_id_addr);
        rt.expect_validate_caller_any();
        expect_get_config(&rt);
        let approve_params = ApproveCreditParams {
            from: sponsor_id_addr,
            to: spender_id_addr,
            caller_allowlist: None,
            credit_limit: None,
            gas_fee_limit: None,
            ttl: None,
        };
        expect_emitted_approve_event(
            &rt,
            sponsor_f4_eth_addr,
            spender_f4_eth_addr,
            approve_params.credit_limit.clone(),
            approve_params.gas_fee_limit.clone(),
            0,
        );
        let response = rt.call::<BlobsActor>(
            Method::ApproveCredit as u64,
            IpldBlock::serialize_cbor(&approve_params).unwrap(),
        );
        assert!(response.is_ok());
        rt.verify();

        // Try sending zero
        rt.set_origin(spender_id_addr);
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, spender_id_addr);
        rt.expect_validate_caller_any();
        rt.set_received(TokenAmount::zero());
        let hash = new_hash(1024);
        let add_params = AddBlobParams {
            sponsor: Some(sponsor_id_addr),
            hash: hash.0,
            metadata_hash: new_hash(1024).0,
            source: new_pk(),
            id: SubscriptionId::default(),
            size: hash.1,
            ttl: Some(3600),
            from: spender_id_addr,
        };
        expect_get_config(&rt);
        expect_emitted_add_event(&rt, 0, &add_params, sponsor_f4_eth_addr, add_params.size);
        let response = rt.call::<BlobsActor>(
            Method::AddBlob as u64,
            IpldBlock::serialize_cbor(&add_params).unwrap(),
        );
        assert!(response.is_ok());
        rt.verify();

        // Try sending non-zero -> cannot buy for a sponsor
        rt.set_origin(spender_id_addr);
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, spender_id_addr);
        rt.expect_validate_caller_any();
        rt.set_received(TokenAmount::from_whole(1));
        let hash = new_hash(1024);
        let add_params = AddBlobParams {
            sponsor: Some(sponsor_id_addr),
            hash: hash.0,
            metadata_hash: new_hash(1024).0,
            source: new_pk(),
            id: SubscriptionId::default(),
            size: hash.1,
            ttl: Some(3600),
            from: spender_id_addr,
        };
        expect_get_config(&rt);
        let response = rt.call::<BlobsActor>(
            Method::AddBlob as u64,
            IpldBlock::serialize_cbor(&add_params).unwrap(),
        );
        assert!(response.is_err());
        rt.verify();
    }
}
