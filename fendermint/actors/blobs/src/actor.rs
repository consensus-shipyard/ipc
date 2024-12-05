// Copyright 2024 Textile
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::HashSet;

use fendermint_actor_blobs_shared::params::{
    AddBlobParams, ApproveCreditParams, BuyCreditParams, DeleteBlobParams, FinalizeBlobParams,
    GetAccountParams, GetAddedBlobsParams, GetBlobParams, GetBlobStatusParams,
    GetCreditApprovalParams, GetPendingBlobsParams, GetStatsReturn, RevokeCreditParams,
    SetAccountBlobTtlStatusParams, SetBlobPendingParams,
};
use fendermint_actor_blobs_shared::state::{
    Account, Blob, BlobStatus, CreditApproval, Hash, PublicKey, Subscription, SubscriptionId,
};

use crate::{ConstructorParams, State, BLOBS_ACTOR_NAME};
use fendermint_actor_blobs_shared::Method;
use fendermint_actor_machine::{resolve_external, resolve_external_non_machine};
use fil_actors_runtime::{
    actor_dispatch, actor_error, extract_send_result,
    runtime::{ActorCode, Runtime},
    ActorError, FIRST_EXPORTED_METHOD_NUMBER, SYSTEM_ACTOR_ADDR,
};
use fvm_ipld_encoding::ipld_block::IpldBlock;
use fvm_shared::address::Address;
use fvm_shared::{MethodNum, METHOD_SEND};

#[cfg(feature = "fil-actor")]
fil_actors_runtime::wasm_trampoline!(BlobsActor);

pub struct BlobsActor;

type BlobTuple = (Hash, HashSet<(Address, SubscriptionId, PublicKey)>);

impl BlobsActor {
    fn constructor(rt: &impl Runtime, params: ConstructorParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;
        let state = State::new(params.capacity, params.debit_rate);
        rt.create(&state)
    }

    fn get_stats(rt: &impl Runtime) -> Result<GetStatsReturn, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let stats = rt.state::<State>()?.get_stats(rt.current_balance());
        Ok(stats)
    }

    fn buy_credit(rt: &impl Runtime, params: BuyCreditParams) -> Result<Account, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let recipient = resolve_external_non_machine(rt, params.0)?;
        rt.transaction(|st: &mut State, rt| {
            st.buy_credit(recipient, rt.message().value_received(), rt.curr_epoch())
        })
    }

    fn approve_credit(
        rt: &impl Runtime,
        params: ApproveCreditParams,
    ) -> Result<CreditApproval, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let from = resolve_external_non_machine(rt, params.from)?;
        let (origin, caller) = if rt.message().origin() == rt.message().caller() {
            let (origin, _) = resolve_external(rt, rt.message().origin())?;
            (origin, origin)
        } else {
            let (origin, _) = resolve_external(rt, rt.message().origin())?;
            let (caller, _) = resolve_external(rt, rt.message().caller())?;
            (origin, caller)
        };
        // Credit owner must be the transaction origin or caller
        if from != caller && from != origin {
            return Err(ActorError::illegal_argument(format!(
                "from {} does not match origin or caller",
                from
            )));
        }
        let receiver = resolve_external_non_machine(rt, params.receiver)?;
        let required_caller = if let Some(required_caller) = params.required_caller {
            let (required_caller, _) = resolve_external(rt, required_caller)?;
            Some(required_caller)
        } else {
            None
        };
        rt.transaction(|st: &mut State, rt| {
            st.approve_credit(
                from,
                receiver,
                required_caller,
                rt.curr_epoch(),
                params.limit,
                params.ttl,
            )
        })
    }

    fn get_credit_approval(
        rt: &impl Runtime,
        params: GetCreditApprovalParams,
    ) -> Result<Option<CreditApproval>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let from = resolve_external_non_machine(rt, params.from)?;
        let receiver = resolve_external_non_machine(rt, params.receiver)?;
        let (caller, _) = resolve_external(rt, params.caller)?;
        let approval = rt
            .state::<State>()?
            .get_credit_approval(from, receiver, caller);
        Ok(approval)
    }

    fn revoke_credit(rt: &impl Runtime, params: RevokeCreditParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let from = resolve_external_non_machine(rt, params.from)?;
        let (origin, caller) = if rt.message().origin() == rt.message().caller() {
            let (origin, _) = resolve_external(rt, rt.message().origin())?;
            (origin, origin)
        } else {
            let (origin, _) = resolve_external(rt, rt.message().origin())?;
            let (caller, _) = resolve_external(rt, rt.message().caller())?;
            (origin, caller)
        };
        // Credit owner must be the transaction origin or caller
        if from != caller && from != origin {
            return Err(ActorError::illegal_argument(format!(
                "from {} does not match origin or caller",
                from
            )));
        }
        let receiver = resolve_external_non_machine(rt, params.receiver)?;
        let required_caller = if let Some(required_caller) = params.required_caller {
            let (required_caller, _) = resolve_external(rt, required_caller)?;
            Some(required_caller)
        } else {
            None
        };
        rt.transaction(|st: &mut State, _| st.revoke_credit(from, receiver, required_caller))
    }

    fn get_account(
        rt: &impl Runtime,
        params: GetAccountParams,
    ) -> Result<Option<Account>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let account = rt.state::<State>()?.get_account(params.0);
        Ok(account)
    }

    fn debit_accounts(rt: &impl Runtime) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;
        let deletes = rt.transaction(|st: &mut State, _| st.debit_accounts(rt.curr_epoch()))?;
        for hash in deletes {
            delete_from_disc(hash)?;
        }
        Ok(())
    }

    fn add_blob(rt: &impl Runtime, params: AddBlobParams) -> Result<Subscription, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let (origin, _) = resolve_external(rt, rt.message().origin())?;
        let (caller, _) = resolve_external(rt, rt.message().caller())?;
        // The blob subscriber will be the sponsor if specified and approved
        let subscriber = if let Some(sponsor) = params.sponsor {
            resolve_external_non_machine(rt, sponsor)?
        } else {
            origin
        };
        let tokens_received = rt.message().value_received();
        let (subscription, unspent_tokens) = rt.transaction(|st: &mut State, rt| {
            st.add_blob(
                origin,
                caller,
                subscriber,
                rt.curr_epoch(),
                params.hash,
                params.metadata_hash,
                params.id,
                params.size,
                params.ttl,
                params.source,
                tokens_received,
            )
        })?;
        // Send the tokens not required for the buying back
        if !unspent_tokens.is_zero() {
            extract_send_result(rt.send_simple(&origin, METHOD_SEND, None, unspent_tokens))?;
        }
        Ok(subscription)
    }

    fn get_blob(rt: &impl Runtime, params: GetBlobParams) -> Result<Option<Blob>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let blob = rt.state::<State>()?.get_blob(params.0);
        Ok(blob)
    }

    fn get_blob_status(
        rt: &impl Runtime,
        params: GetBlobStatusParams,
    ) -> Result<Option<BlobStatus>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let status =
            rt.state::<State>()?
                .get_blob_status(params.subscriber, params.hash, params.id);
        Ok(status)
    }

    fn get_added_blobs(
        rt: &impl Runtime,
        params: GetAddedBlobsParams,
    ) -> Result<Vec<BlobTuple>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let added = rt.state::<State>()?.get_added_blobs(params.0);
        Ok(added)
    }

    fn get_pending_blobs(
        rt: &impl Runtime,
        params: GetPendingBlobsParams,
    ) -> Result<Vec<BlobTuple>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let pending = rt.state::<State>()?.get_pending_blobs(params.0);
        Ok(pending)
    }

    fn set_blob_pending(rt: &impl Runtime, params: SetBlobPendingParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;
        // We control this method call and can guarantee subscriber is an external address,
        // i.e., no need to resolve its external address.
        rt.transaction(|st: &mut State, _| {
            st.set_blob_pending(params.subscriber, params.hash, params.id, params.source)
        })
    }

    fn finalize_blob(rt: &impl Runtime, params: FinalizeBlobParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;
        // We control this method call and can guarantee subscriber is an external address,
        // i.e., no need to resolve its external address.
        rt.transaction(|st: &mut State, _| {
            st.finalize_blob(
                params.subscriber,
                rt.curr_epoch(),
                params.hash,
                params.id,
                params.status,
            )
        })
    }

    fn delete_blob(rt: &impl Runtime, params: DeleteBlobParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        let (origin, _) = resolve_external(rt, rt.message().origin())?;
        let (caller, _) = resolve_external(rt, rt.message().caller())?;
        // The blob subscriber will be the sponsor if specified and approved
        let subscriber = if let Some(sponsor) = params.sponsor {
            resolve_external_non_machine(rt, sponsor)?
        } else {
            origin
        };
        let delete = rt.transaction(|st: &mut State, _| {
            st.delete_blob(
                origin,
                caller,
                subscriber,
                rt.curr_epoch(),
                params.hash,
                params.id,
            )
        })?;
        if delete {
            delete_from_disc(params.hash)?;
        }
        Ok(())
    }

    fn set_account_blob_ttl_status(
        rt: &impl Runtime,
        params: SetAccountBlobTtlStatusParams,
    ) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;
        
        rt.transaction(|st: &mut State, _| {
            st.set_ttl_status(params.account, params.status, rt.curr_epoch())
        })?;
        Ok(())
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

fn delete_from_disc(hash: Hash) -> Result<(), ActorError> {
    #[cfg(feature = "fil-actor")]
    {
        blobs_actor_sdk::hash_rm(hash.0).map_err(|en| {
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
        GetStats => get_stats,
        BuyCredit => buy_credit,
        ApproveCredit => approve_credit,
        GetCreditApproval => get_credit_approval,
        RevokeCredit => revoke_credit,
        GetAccount => get_account,
        DebitAccounts => debit_accounts,
        AddBlob => add_blob,
        GetBlob => get_blob,
        GetBlobStatus => get_blob_status,
        GetAddedBlobs => get_added_blobs,
        GetPendingBlobs => get_pending_blobs,
        SetBlobPending => set_blob_pending,
        FinalizeBlob => finalize_blob,
        DeleteBlob => delete_blob,
        SetAccountBlobTtlStatus => set_account_blob_ttl_status,
        _ => fallback,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fil_actors_evm_shared::address::EthAddress;
    use fil_actors_runtime::test_utils::{
        expect_empty, MockRuntime, ETHACCOUNT_ACTOR_CODE_ID, EVM_ACTOR_CODE_ID,
        SYSTEM_ACTOR_CODE_ID,
    };
    use fil_actors_runtime::SYSTEM_ACTOR_ADDR;
    use fvm_ipld_encoding::ipld_block::IpldBlock;
    use fvm_shared::address::Address;
    use fvm_shared::bigint::BigInt;
    use fvm_shared::clock::ChainEpoch;
    use fvm_shared::econ::TokenAmount;
    use fvm_shared::{error::ExitCode, METHOD_SEND};
    use num_traits::Zero;
    use rand::RngCore;

    pub fn new_hash(size: usize) -> (Hash, u64) {
        let mut rng = rand::thread_rng();
        let mut data = vec![0u8; size];
        rng.fill_bytes(&mut data);
        (
            Hash(*iroh_base::hash::Hash::new(&data).as_bytes()),
            size as u64,
        )
    }

    pub fn new_pk() -> PublicKey {
        let mut rng = rand::thread_rng();
        let mut data = [0u8; 32];
        rng.fill_bytes(&mut data);
        PublicKey(data)
    }

    pub fn construct_and_verify(capacity: u64, debit_rate: u64) -> MockRuntime {
        let rt = MockRuntime {
            receiver: Address::new_id(10),
            ..Default::default()
        };
        rt.set_caller(*SYSTEM_ACTOR_CODE_ID, SYSTEM_ACTOR_ADDR);
        rt.expect_validate_caller_addr(vec![SYSTEM_ACTOR_ADDR]);
        let result = rt
            .call::<BlobsActor>(
                Method::Constructor as u64,
                IpldBlock::serialize_cbor(&ConstructorParams {
                    capacity,
                    debit_rate,
                })
                .unwrap(),
            )
            .unwrap();
        expect_empty(result);
        rt.verify();
        rt.reset();
        rt
    }

    #[test]
    fn test_buy_credit() {
        let rt = construct_and_verify(1024 * 1024, 1);

        let id_addr = Address::new_id(110);
        let eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000000"
        ));
        let f4_eth_addr = Address::new_delegated(10, &eth_addr.0).unwrap();

        rt.set_delegated_address(id_addr.id().unwrap(), f4_eth_addr);
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, id_addr);
        rt.set_origin(id_addr);

        let mut expected_credits = BigInt::from(1000000000000000000u64);
        rt.set_received(TokenAmount::from_whole(1));
        rt.expect_validate_caller_any();
        let fund_params = BuyCreditParams(f4_eth_addr);
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
        rt.verify();

        expected_credits += BigInt::from(1000000000u64);
        rt.set_received(TokenAmount::from_nano(1));
        rt.expect_validate_caller_any();
        let fund_params = BuyCreditParams(f4_eth_addr);
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
        rt.verify();

        expected_credits += BigInt::from(1u64);
        rt.set_received(TokenAmount::from_atto(1));
        rt.expect_validate_caller_any();
        let fund_params = BuyCreditParams(f4_eth_addr);
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
        rt.verify();
    }

    #[test]
    fn test_approve_credit() {
        let rt = construct_and_verify(1024 * 1024, 1);

        // Credit owner
        let owner_id_addr = Address::new_id(110);
        let owner_eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000000"
        ));
        let owner_f4_eth_addr = Address::new_delegated(10, &owner_eth_addr.0).unwrap();
        rt.set_delegated_address(owner_id_addr.id().unwrap(), owner_f4_eth_addr);

        // Credit receiver
        let receiver_id_addr = Address::new_id(111);
        let receiver_eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000001"
        ));
        let receiver_f4_eth_addr = Address::new_delegated(10, &receiver_eth_addr.0).unwrap();
        rt.set_delegated_address(receiver_id_addr.id().unwrap(), receiver_f4_eth_addr);
        rt.set_address_actor_type(receiver_id_addr, *ETHACCOUNT_ACTOR_CODE_ID);

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
        let approve_params = ApproveCreditParams {
            from: owner_id_addr,
            receiver: receiver_id_addr,
            required_caller: None,
            limit: None,
            ttl: None,
        };
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
        let approve_params = ApproveCreditParams {
            from: owner_id_addr,
            receiver: receiver_id_addr,
            required_caller: None,
            limit: None,
            ttl: None,
        };
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
            from: receiver_id_addr, // mismatch
            receiver: receiver_id_addr,
            required_caller: None,
            limit: None,
            ttl: None,
        };
        let result = rt.call::<BlobsActor>(
            Method::ApproveCredit as u64,
            IpldBlock::serialize_cbor(&approve_params).unwrap(),
        );
        let expected_return = Err(ActorError::illegal_argument(format!(
            "from {} does not match origin or caller",
            receiver_f4_eth_addr
        )));
        assert_eq!(result, expected_return);
        rt.verify();
    }

    #[test]
    fn test_revoke_credit() {
        let rt = construct_and_verify(1024 * 1024, 1);

        // Credit owner
        let owner_id_addr = Address::new_id(110);
        let owner_eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000000"
        ));
        let owner_f4_eth_addr = Address::new_delegated(10, &owner_eth_addr.0).unwrap();
        rt.set_delegated_address(owner_id_addr.id().unwrap(), owner_f4_eth_addr);

        // Credit receiver
        let receiver_id_addr = Address::new_id(111);
        let receiver_eth_addr = EthAddress(hex_literal::hex!(
            "CAFEB0BA00000000000000000000000000000001"
        ));
        let receiver_f4_eth_addr = Address::new_delegated(10, &receiver_eth_addr.0).unwrap();
        rt.set_delegated_address(receiver_id_addr.id().unwrap(), receiver_f4_eth_addr);
        rt.set_address_actor_type(receiver_id_addr, *ETHACCOUNT_ACTOR_CODE_ID);

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
        let approve_params = ApproveCreditParams {
            from: owner_id_addr,
            receiver: receiver_id_addr,
            required_caller: None,
            limit: None,
            ttl: None,
        };
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
            receiver: receiver_id_addr,
            required_caller: None,
        };
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
            receiver: receiver_id_addr,
            required_caller: None,
        };
        let result = rt.call::<BlobsActor>(
            Method::RevokeCredit as u64,
            IpldBlock::serialize_cbor(&revoke_params).unwrap(),
        );
        assert!(result.is_ok());
        rt.verify();

        // Caller/origin mismatch with from
        rt.set_caller(*EVM_ACTOR_CODE_ID, proxy_id_addr);
        rt.set_origin(owner_id_addr);
        rt.expect_validate_caller_any();
        let revoke_params = RevokeCreditParams {
            from: receiver_id_addr, // mismatch
            receiver: receiver_id_addr,
            required_caller: None,
        };
        let result = rt.call::<BlobsActor>(
            Method::RevokeCredit as u64,
            IpldBlock::serialize_cbor(&revoke_params).unwrap(),
        );
        let expected_return = Err(ActorError::illegal_argument(format!(
            "from {} does not match origin or caller",
            receiver_f4_eth_addr
        )));
        assert_eq!(result, expected_return);
        rt.verify();
    }

    #[test]
    fn test_add_blob() {
        let rt = construct_and_verify(1024 * 1024, 1);

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
            id: SubscriptionId::Default,
            size: hash.1,
            ttl: Some(3600),
        };
        let result = rt.call::<BlobsActor>(
            Method::AddBlob as u64,
            IpldBlock::serialize_cbor(&add_params).unwrap(),
        );
        assert!(result.is_err());
        rt.verify();

        // Fund an account
        rt.set_received(TokenAmount::from_whole(1));
        rt.expect_validate_caller_any();
        let fund_params = BuyCreditParams(f4_eth_addr);
        let result = rt.call::<BlobsActor>(
            Method::BuyCredit as u64,
            IpldBlock::serialize_cbor(&fund_params).unwrap(),
        );
        assert!(result.is_ok());
        rt.verify();

        // Try with sufficient balance
        rt.set_epoch(ChainEpoch::from(5));
        rt.expect_validate_caller_any();
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
        assert!(!subscription.auto_renew);
        assert_eq!(subscription.delegate, None);
        rt.verify();
    }

    #[test]
    fn test_add_blob_inline_buy() {
        let rt = construct_and_verify(1024 * 1024, 1);

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
            hash: hash.0,
            metadata_hash: new_hash(1024).0,
            source: new_pk(),
            id: SubscriptionId::Default,
            size: hash.1,
            ttl: Some(3600),
        };
        let tokens_sent = TokenAmount::from_whole(1);
        rt.set_received(tokens_sent.clone());
        rt.set_balance(tokens_sent.clone());
        let tokens_required_atto = add_params.size * add_params.ttl.unwrap() as u64;
        let expected_tokens_unspent = tokens_sent.atto() - tokens_required_atto;
        rt.expect_send_simple(
            f4_eth_addr,
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
            id: SubscriptionId::Default,
            size: hash.1,
            ttl: Some(3600),
        };
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
            id: SubscriptionId::Default,
            size: hash.1,
            ttl: Some(3600),
        };
        let result = rt.call::<BlobsActor>(
            Method::AddBlob as u64,
            IpldBlock::serialize_cbor(&add_params).unwrap(),
        );
        assert!(result.is_ok());
        rt.verify();
    }

    #[test]
    fn test_add_blob_with_sponsor() {
        let rt = construct_and_verify(1024 * 1024, 1);

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
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, sponsor_id_addr);
        rt.set_received(TokenAmount::from_whole(1));
        rt.expect_validate_caller_any();
        let fund_params = BuyCreditParams(sponsor_id_addr);
        let buy_credit_result = rt.call::<BlobsActor>(
            Method::BuyCredit as u64,
            IpldBlock::serialize_cbor(&fund_params).unwrap(),
        );
        assert!(buy_credit_result.is_ok());
        rt.verify();

        // Sponsors approves credit
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, sponsor_id_addr);
        rt.set_origin(sponsor_id_addr);
        rt.expect_validate_caller_any();
        let approve_params = ApproveCreditParams {
            from: sponsor_id_addr,
            receiver: spender_id_addr,
            required_caller: None,
            limit: None,
            ttl: None,
        };
        let result = rt.call::<BlobsActor>(
            Method::ApproveCredit as u64,
            IpldBlock::serialize_cbor(&approve_params).unwrap(),
        );
        assert!(result.is_ok());
        rt.verify();

        rt.set_origin(sponsor_id_addr);
        rt.set_caller(*ETHACCOUNT_ACTOR_CODE_ID, sponsor_id_addr);
        rt.expect_validate_caller_any();
        let approve_params = ApproveCreditParams {
            from: sponsor_id_addr,
            receiver: spender_id_addr,
            required_caller: None,
            limit: None,
            ttl: None,
        };
        let approve_result = rt.call::<BlobsActor>(
            Method::ApproveCredit as u64,
            IpldBlock::serialize_cbor(&approve_params).unwrap(),
        );
        assert!(approve_result.is_ok());
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
            id: SubscriptionId::Default,
            size: hash.1,
            ttl: Some(3600),
        };
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
            id: SubscriptionId::Default,
            size: hash.1,
            ttl: Some(3600),
        };
        let response = rt.call::<BlobsActor>(
            Method::AddBlob as u64,
            IpldBlock::serialize_cbor(&add_params).unwrap(),
        );
        assert!(response.is_err());
        rt.verify();
    }
}
