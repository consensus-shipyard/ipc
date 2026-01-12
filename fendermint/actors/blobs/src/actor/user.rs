// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_blobs_shared::{
    accounts::{Account, GetAccountParams},
    blobs::{
        AddBlobParams, Blob, DeleteBlobParams, GetBlobParams, OverwriteBlobParams, Subscription,
    },
    credit::{
        ApproveCreditParams, BuyCreditParams, Credit, CreditApproval, GetCreditApprovalParams,
        RevokeCreditParams, SetSponsorParams,
    },
};
use fendermint_actor_ipc_storage_config_shared::get_config;
use fil_actors_runtime::{extract_send_result, runtime::Runtime, ActorError};
use fvm_shared::{econ::TokenAmount, METHOD_SEND};
use ipc_storage_actor_sdk::{
    caller::{Caller, CallerOption},
    evm::emit_evm_event,
    util::is_bucket_address,
    util::to_delegated_address,
};
use num_traits::Zero;

use crate::{
    actor::{delete_from_disc, BlobsActor},
    caller::DelegationOptions,
    sol_facade::{
        blobs as sol_blobs,
        credit::{CreditApproved, CreditPurchased, CreditRevoked},
        gas::{GasSponsorSet, GasSponsorUnset},
    },
    state::blobs::{AddBlobStateParams, DeleteBlobStateParams},
    State,
};

impl BlobsActor {
    /// Buy credit with token.
    ///
    /// The `to` address must be delegated (only delegated addresses can own credit).
    pub fn buy_credit(rt: &impl Runtime, params: BuyCreditParams) -> Result<Account, ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        let caller = Caller::new_delegated(rt, params.0, None, CallerOption::Auth)?;
        let config = get_config(rt)?;

        let mut credit_amount = Credit::zero();
        let account = rt.transaction(|st: &mut State, rt| {
            let pre_buy = st.credits.credit_sold.clone();
            let account = st.buy_credit(
                rt.store(),
                &config,
                caller.state_address(),
                rt.message().value_received(),
                rt.curr_epoch(),
            )?;
            credit_amount = &st.credits.credit_sold - &pre_buy;
            Ok(account)
        })?;

        emit_evm_event(
            rt,
            CreditPurchased::new(caller.event_address(), credit_amount),
        )?;

        account.to_shared(rt)
    }

    /// Approve credit and gas usage from one account to another.
    ///
    /// The `from` address must be delegated (only delegated addresses can own credit).
    /// The `from` address must be the message origin or caller.
    /// The `to` address must be delegated (only delegated addresses can use credit).
    /// The `to` address will be created if it does not exist.
    /// TODO: Remove the `caller_allowlist` parameter.
    pub fn approve_credit(
        rt: &impl Runtime,
        params: ApproveCreditParams,
    ) -> Result<CreditApproval, ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        let from_caller =
            Caller::new_delegated(rt, rt.message().caller(), None, CallerOption::Auth)?;
        let to_caller = Caller::new_delegated(rt, params.to, None, CallerOption::Create)?;
        let config = get_config(rt)?;

        let approval = rt.transaction(|st: &mut State, rt| {
            let approval = st.approve_credit(
                &config,
                rt.store(),
                from_caller.state_address(),
                to_caller.state_address(),
                DelegationOptions {
                    credit_limit: params.credit_limit,
                    gas_fee_limit: params.gas_fee_limit,
                    ttl: params.ttl,
                },
                rt.curr_epoch(),
            );

            // For convenience, set the approvee's sponsor to the approver if it was created
            if to_caller.created() {
                st.set_account_sponsor(
                    &config,
                    rt.store(),
                    to_caller.state_address(),
                    Some(from_caller.state_address()),
                    rt.curr_epoch(),
                )?;
            }
            approval
        })?;

        emit_evm_event(
            rt,
            CreditApproved {
                from: from_caller.event_address(),
                to: to_caller.event_address(),
                credit_limit: approval.credit_limit.clone(),
                gas_fee_limit: approval.gas_allowance_limit.clone(),
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
    pub fn revoke_credit(rt: &impl Runtime, params: RevokeCreditParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        let from_caller =
            Caller::new_delegated(rt, rt.message().caller(), None, CallerOption::Auth)?;
        let to_caller = Caller::new_delegated(rt, params.to, None, CallerOption::None)?;

        rt.transaction(|st: &mut State, rt| {
            st.revoke_credit(
                rt.store(),
                from_caller.state_address(),
                to_caller.state_address(),
            )
        })?;

        emit_evm_event(
            rt,
            CreditRevoked::new(from_caller.event_address(), to_caller.event_address()),
        )?;

        Ok(())
    }

    /// Sets or unsets a default credit and gas sponsor from one account to another.
    ///
    /// If `sponsor` does not exist, the default sponsor is unset.
    /// The `from` address must be delegated (only delegated addresses can use credit).
    /// The `from` address must be the message origin or caller.
    /// The `sponsor` address must be delegated (only delegated addresses can own credit).
    pub fn set_account_sponsor(
        rt: &impl Runtime,
        params: SetSponsorParams,
    ) -> Result<(), ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        let caller =
            Caller::new_delegated(rt, rt.message().caller(), params.0, CallerOption::Auth)?;
        let config = get_config(rt)?;

        rt.transaction(|st: &mut State, rt| {
            st.set_account_sponsor(
                &config,
                rt.store(),
                caller.state_address(),
                caller.sponsor_state_address(),
                rt.curr_epoch(),
            )
        })?;

        if let Some(sponsor) = caller.sponsor_address() {
            emit_evm_event(rt, GasSponsorSet::mew(sponsor))?;
        } else {
            emit_evm_event(rt, GasSponsorUnset::new())?;
        }

        Ok(())
    }

    /// Returns the account for an address.
    pub fn get_account(
        rt: &impl Runtime,
        params: GetAccountParams,
    ) -> Result<Option<Account>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        let caller = Caller::new(rt, params.0, None, CallerOption::None)?;

        let account = rt
            .state::<State>()?
            .get_account(rt.store(), caller.state_address())?
            .map(|mut account| {
                // Resolve the credit sponsor
                account.credit_sponsor = account
                    .credit_sponsor
                    .map(|sponsor| to_delegated_address(rt, sponsor))
                    .transpose()?;

                account.to_shared(rt)
            });

        account.transpose()
    }

    /// Returns the credit approval from one account to another if it exists.
    pub fn get_credit_approval(
        rt: &impl Runtime,
        params: GetCreditApprovalParams,
    ) -> Result<Option<CreditApproval>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        let from_caller = Caller::new(rt, params.from, None, CallerOption::None)?;
        let to_caller = Caller::new(rt, params.to, None, CallerOption::None)?;

        let approval = rt.state::<State>()?.get_credit_approval(
            rt.store(),
            from_caller.state_address(),
            to_caller.state_address(),
        )?;

        Ok(approval)
    }

    /// Adds or updates a blob subscription.
    ///
    /// The subscriber will only need credits for blobs that are not already covered by one of
    /// their existing subscriptions.
    ///
    /// The `sponsor` will be the subscriber (the account responsible for payment), if it exists
    /// and there is an approval from `sponsor` to `from`.
    ///
    /// The `from` address must be delegated (only delegated addresses can use credit).
    /// The `sponsor` address must be delegated (only delegated addresses can use credit).
    pub fn add_blob(rt: &impl Runtime, params: AddBlobParams) -> Result<Subscription, ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        let from = if is_bucket_address(rt, rt.message().caller())? {
            params.from
        } else {
            rt.message().caller()
        };
        let caller = Caller::new_delegated(rt, from, params.sponsor, CallerOption::Auth)?;
        let token_amount = rt.message().value_received();
        let config = get_config(rt)?;

        let mut capacity_used = 0;
        let (sub, token_rebate) = rt.transaction(|st: &mut State, rt| {
            let initial_capacity_used = st.blobs.bytes_size();
            let res = st.add_blob(
                rt.store(),
                &config,
                caller.state_address(),
                caller.sponsor_state_address(),
                AddBlobStateParams::from_actor_params(
                    params.clone(),
                    rt.curr_epoch(),
                    token_amount,
                ),
            )?;
            capacity_used = st.blobs.bytes_size() - initial_capacity_used;
            Ok(res)
        })?;

        // Send back unspent tokens
        if !token_rebate.is_zero() {
            extract_send_result(rt.send_simple(
                &caller.state_address(),
                METHOD_SEND,
                None,
                token_rebate,
            ))?;
        }

        emit_evm_event(
            rt,
            sol_blobs::BlobAdded {
                subscriber: caller.event_address(),
                hash: &params.hash,
                size: params.size,
                expiry: sub.expiry,
                bytes_used: capacity_used,
            },
        )?;

        Ok(sub)
    }

    /// Returns a blob by hash if it exists.
    pub fn get_blob(rt: &impl Runtime, params: GetBlobParams) -> Result<Option<Blob>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        match rt.state::<State>()?.get_blob(rt.store(), params.0)? {
            Some(blob) => Ok(Some(blob.to_shared(rt)?)),
            None => Ok(None),
        }
    }

    /// Deletes a blob subscription.
    ///
    /// The `sponsor` will be the subscriber (the account responsible for payment), if it exists
    /// and there is an approval from `sponsor` to `from`.
    ///
    /// The `from` address must be delegated (only delegated addresses can use credit).
    /// The `sponsor` address must be delegated (only delegated addresses can use credit).
    pub fn delete_blob(rt: &impl Runtime, params: DeleteBlobParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        let from = if is_bucket_address(rt, rt.message().caller())? {
            params.from
        } else {
            rt.message().caller()
        };

        let caller = Caller::new_delegated(rt, from, params.sponsor, CallerOption::Auth)?;

        let mut capacity_released = 0;
        let (_, size, _) = rt.transaction(|st: &mut State, rt| {
            let initial_capacity_used = st.blobs.bytes_size();
            let res = st.delete_blob(
                rt.store(),
                caller.state_address(),
                caller.sponsor_state_address(),
                DeleteBlobStateParams::from_actor_params(params.clone(), rt.curr_epoch()),
            )?;
            capacity_released = initial_capacity_used - st.blobs.bytes_size();
            Ok(res)
        })?;

        emit_evm_event(
            rt,
            sol_blobs::BlobDeleted {
                subscriber: caller.event_address(),
                hash: &params.hash,
                size,
                bytes_released: capacity_released,
            },
        )?;

        Ok(())
    }

    /// Deletes a blob subscription and adds another in a single call.
    ///
    /// This method is more efficient than two separate calls to `delete_blob` and `add_blob`,
    /// and is useful for some blob workflows like replacing a key in a bucket actor.
    ///
    /// The `sponsor` will be the subscriber (the account responsible for payment), if it exists
    /// and there is an approval from `sponsor` to `from`.
    ///
    /// The `from` address must be delegated (only delegated addresses can use credit).
    /// The `sponsor` address must be delegated (only delegated addresses can use credit).
    pub fn overwrite_blob(
        rt: &impl Runtime,
        params: OverwriteBlobParams,
    ) -> Result<Subscription, ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        let from = if is_bucket_address(rt, rt.message().caller())? {
            params.add.from
        } else {
            rt.message().caller()
        };

        let caller = Caller::new_delegated(rt, from, params.add.sponsor, CallerOption::Auth)?;
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

            let initial_capacity_used = st.blobs.bytes_size();
            let (delete, delete_size, _) = if overwrite {
                st.delete_blob(
                    rt.store(),
                    caller.state_address(),
                    caller.sponsor_state_address(),
                    DeleteBlobStateParams {
                        hash: params.old_hash,
                        id: add_params.id.clone(),
                        epoch: rt.curr_epoch(),
                        skip_credit_return: false,
                    },
                )?
            } else {
                (false, 0, false)
            };
            capacity_released = initial_capacity_used - st.blobs.bytes_size();

            let initial_capacity_used = st.blobs.bytes_size();
            let (subscription, _) = st.add_blob(
                rt.store(),
                &config,
                caller.state_address(),
                caller.sponsor_state_address(),
                AddBlobStateParams::from_actor_params(
                    add_params,
                    rt.curr_epoch(),
                    TokenAmount::zero(),
                ),
            )?;
            capacity_used = st.blobs.bytes_size() - initial_capacity_used;

            Ok((delete, delete_size, subscription))
        })?;

        if delete {
            delete_from_disc(params.old_hash)?;
        }

        if overwrite {
            emit_evm_event(
                rt,
                sol_blobs::BlobDeleted {
                    subscriber: caller.event_address(),
                    hash: &params.old_hash,
                    size: delete_size,
                    bytes_released: capacity_released,
                },
            )?;
        }
        emit_evm_event(
            rt,
            sol_blobs::BlobAdded {
                subscriber: caller.event_address(),
                hash: &add_hash,
                size: add_size,
                expiry: sub.expiry,
                bytes_used: capacity_used,
            },
        )?;

        Ok(sub)
    }
}
