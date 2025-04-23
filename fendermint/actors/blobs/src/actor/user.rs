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
use fendermint_actor_recall_config_shared::get_config;
use fil_actors_runtime::{extract_send_result, runtime::Runtime, ActorError};
use fvm_shared::{econ::TokenAmount, METHOD_SEND};
use num_traits::Zero;
use recall_actor_sdk::{
    caller::{Caller, CallerOption},
    evm::emit_evm_event,
    util::is_bucket_address,
    util::to_delegated_address,
};

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
        let (delete, size) = rt.transaction(|st: &mut State, rt| {
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

        if delete {
            delete_from_disc(params.hash)?;
        }

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

    /// Deletes a blob subscription and adds another in a sinlge call.
    ///
    /// This method is more efficient than two separate calls to `delete_blob` and `add_blob`,
    /// and is useful for some blob workflows like a replacing a key in a bucket actor.
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
            let (delete, delete_size) = if overwrite {
                st.delete_blob(
                    rt.store(),
                    caller.state_address(),
                    caller.sponsor_state_address(),
                    DeleteBlobStateParams {
                        hash: params.old_hash,
                        id: add_params.id.clone(),
                        epoch: rt.curr_epoch(),
                    },
                )?
            } else {
                (false, 0)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{
        construct_and_verify, expect_emitted_add_event, expect_emitted_approve_event,
        expect_emitted_purchase_event, expect_emitted_revoke_event, expect_get_config,
    };
    use cid::Cid;
    use fendermint_actor_blobs_shared::{
        blobs::{BlobStatus, SubscriptionId},
        method::Method,
    };
    use fendermint_actor_blobs_testing::{new_hash, new_pk, setup_logs};
    use fil_actors_evm_shared::address::EthAddress;
    use fil_actors_runtime::test_utils::{
        MockRuntime, ETHACCOUNT_ACTOR_CODE_ID, EVM_ACTOR_CODE_ID,
    };
    use fil_actors_runtime::ADM_ACTOR_ADDR;
    use fvm_ipld_encoding::ipld_block::IpldBlock;
    use fvm_shared::sys::SendFlags;
    use fvm_shared::{
        address::Address, bigint::BigInt, clock::ChainEpoch, error::ExitCode, MethodNum,
    };
    use recall_actor_sdk::util::Kind;

    fn expect_retrieve_bucket_code_cid(rt: &MockRuntime, code_cid: Cid) {
        rt.expect_send(
            ADM_ACTOR_ADDR,
            2892692559 as MethodNum,
            IpldBlock::serialize_cbor(&Kind::Bucket).unwrap(),
            TokenAmount::zero(),
            None,
            SendFlags::READ_ONLY,
            IpldBlock::serialize_cbor(&code_cid).unwrap(),
            ExitCode::OK,
            None,
        );
    }

    #[test]
    fn test_buy_credit() {
        setup_logs();
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
            .deserialize::<Account>()
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
            .deserialize::<Account>()
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
            .deserialize::<Account>()
            .unwrap();
        assert_eq!(result.credit_free, expected_credits);
        assert_eq!(result.gas_allowance, expected_gas_allowance);
        rt.verify();
    }

    #[test]
    fn test_approve_credit() {
        setup_logs();
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

        // Proxy caller (caller mismatch with from, hence proxy is the one who approves)
        rt.set_caller(*EVM_ACTOR_CODE_ID, proxy_id_addr);
        rt.set_origin(owner_id_addr);
        rt.expect_validate_caller_any();
        expect_get_config(&rt);
        let approve_params = ApproveCreditParams {
            to: to_id_addr,
            caller_allowlist: None,
            credit_limit: None,
            gas_fee_limit: None,
            ttl: None,
        };
        expect_emitted_approve_event(
            &rt,
            proxy_f4_eth_addr,
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
    }

    #[test]
    fn test_approve_credit_to_new_account() {
        setup_logs();
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
        rt.expect_send_simple(
            receiver_f4_eth_addr,
            METHOD_SEND,
            None,
            TokenAmount::zero(),
            None,
            ExitCode::OK,
        );
        let approve_params = ApproveCreditParams {
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
        setup_logs();
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
            to: to_id_addr,
            for_caller: None,
        };
        let result = rt.call::<BlobsActor>(
            Method::RevokeCredit as u64,
            IpldBlock::serialize_cbor(&revoke_params).unwrap(),
        );
        let expected_return = Err(ActorError::not_found(format!(
            "{} not found in accounts",
            proxy_id_addr
        )));
        assert_eq!(result, expected_return);
        rt.verify();
    }

    #[test]
    fn test_add_blob() {
        setup_logs();
        let rt = construct_and_verify();

        let token_credit_rate = BigInt::from(1000000000000000000u64);

        let id_addr = Address::new_id(110);
        let eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000000"
        ));
        let f4_eth_addr = Address::new_delegated(10, &eth_addr.0).unwrap();

        rt.set_delegated_address(id_addr.id().unwrap(), f4_eth_addr);
        let code_cid = ETHACCOUNT_ACTOR_CODE_ID.clone();
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, id_addr);
        rt.set_origin(id_addr);
        rt.set_epoch(ChainEpoch::from(0));

        // Try without first funding
        rt.expect_validate_caller_any();
        let hash = new_hash(1024);
        let add_params = AddBlobParams {
            from: id_addr,
            sponsor: None,
            source: new_pk(),
            hash: hash.0,
            metadata_hash: new_hash(1024).0,
            id: SubscriptionId::default(),
            size: hash.1,
            ttl: Some(3600),
        };
        expect_retrieve_bucket_code_cid(&rt, code_cid);
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
        rt.set_received(TokenAmount::zero());
        rt.set_epoch(ChainEpoch::from(5));
        rt.expect_validate_caller_any();
        expect_retrieve_bucket_code_cid(&rt, code_cid);
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
            .deserialize::<Option<Blob>>()
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
        setup_logs();
        let rt = construct_and_verify();

        let id_addr = Address::new_id(110);
        let eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000000"
        ));
        let f4_eth_addr = Address::new_delegated(10, &eth_addr.0).unwrap();

        rt.set_delegated_address(id_addr.id().unwrap(), f4_eth_addr);
        let code_cid = ETHACCOUNT_ACTOR_CODE_ID.clone();
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, id_addr);
        rt.set_origin(id_addr);
        rt.set_epoch(ChainEpoch::from(0));

        // Try sending a lot
        rt.expect_validate_caller_any();
        let hash = new_hash(1024);
        let add_params = AddBlobParams {
            from: id_addr,
            sponsor: None,
            source: new_pk(),
            hash: hash.0,
            metadata_hash: new_hash(1024).0,
            id: SubscriptionId::default(),
            size: hash.1,
            ttl: Some(3600),
        };
        let tokens_sent = TokenAmount::from_whole(1);
        rt.set_received(tokens_sent.clone());
        rt.set_balance(tokens_sent.clone());
        let tokens_required_atto = add_params.size * add_params.ttl.unwrap() as u64;
        let expected_tokens_unspent = tokens_sent.atto() - tokens_required_atto;
        expect_retrieve_bucket_code_cid(&rt, code_cid);
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
            from: id_addr,
            sponsor: None,
            hash: hash.0,
            metadata_hash: new_hash(1024).0,
            source: new_pk(),
            id: SubscriptionId::default(),
            size: hash.1,
            ttl: Some(3600),
        };
        expect_retrieve_bucket_code_cid(&rt, code_cid);
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
            from: id_addr,
            sponsor: None,
            hash: hash.0,
            metadata_hash: new_hash(1024).0,
            source: new_pk(),
            id: SubscriptionId::default(),
            size: hash.1,
            ttl: Some(3600),
        };
        expect_retrieve_bucket_code_cid(&rt, code_cid);
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
        setup_logs();
        let rt = construct_and_verify();
        let code_cid = ETHACCOUNT_ACTOR_CODE_ID.clone();

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
            from: spender_id_addr,
            sponsor: Some(sponsor_id_addr),
            hash: hash.0,
            metadata_hash: new_hash(1024).0,
            source: new_pk(),
            id: SubscriptionId::default(),
            size: hash.1,
            ttl: Some(3600),
        };
        expect_retrieve_bucket_code_cid(&rt, code_cid);
        expect_get_config(&rt);
        expect_emitted_add_event(&rt, 0, &add_params, sponsor_f4_eth_addr, add_params.size);
        let response = rt.call::<BlobsActor>(
            Method::AddBlob as u64,
            IpldBlock::serialize_cbor(&add_params).unwrap(),
        );
        assert!(response.is_ok());
        rt.verify();

        // Try sending non-zero -> cannot buy for a sponsor, tokens are sent back
        rt.set_origin(spender_id_addr);
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, spender_id_addr);
        rt.expect_validate_caller_any();
        let received = TokenAmount::from_whole(1);
        rt.set_received(received.clone());
        rt.set_balance(received.clone());
        let hash = new_hash(1024);
        let add_params = AddBlobParams {
            from: spender_id_addr,
            sponsor: Some(sponsor_id_addr),
            hash: hash.0,
            metadata_hash: new_hash(1024).0,
            source: new_pk(),
            id: SubscriptionId::default(),
            size: hash.1,
            ttl: Some(3600),
        };
        expect_retrieve_bucket_code_cid(&rt, code_cid);
        expect_get_config(&rt);
        expect_emitted_add_event(&rt, 0, &add_params, sponsor_f4_eth_addr, add_params.size);
        rt.expect_send_simple(
            spender_id_addr,
            METHOD_SEND,
            None,
            received,
            None,
            ExitCode::OK,
        );
        let response = rt.call::<BlobsActor>(
            Method::AddBlob as u64,
            IpldBlock::serialize_cbor(&add_params).unwrap(),
        );
        assert!(response.is_ok());
        rt.verify();
    }
}
