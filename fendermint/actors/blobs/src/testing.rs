// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_blobs_shared::{
    blobs::AddBlobParams, credit::BuyCreditParams, method::Method,
};
use fendermint_actor_recall_config_shared::{RecallConfig, RECALL_CONFIG_ACTOR_ADDR};
use fil_actors_runtime::test_utils::{expect_empty, MockRuntime, SYSTEM_ACTOR_CODE_ID};
use fil_actors_runtime::SYSTEM_ACTOR_ADDR;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::ipld_block::IpldBlock;
use fvm_shared::{
    address::Address, clock::ChainEpoch, econ::TokenAmount, error::ExitCode, sys::SendFlags,
    MethodNum,
};
use num_traits::Zero;
use recall_actor_sdk::evm::to_actor_event;

use crate::{
    actor::BlobsActor,
    sol_facade::{
        blobs as sol_blobs,
        credit::{CreditApproved, CreditPurchased, CreditRevoked},
    },
    State,
};

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

pub fn expect_get_config(rt: &MockRuntime) {
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

pub fn expect_emitted_purchase_event(
    rt: &MockRuntime,
    params: &BuyCreditParams,
    amount: TokenAmount,
) {
    let event = to_actor_event(CreditPurchased::new(params.0, amount)).unwrap();
    rt.expect_emitted_event(event);
}

pub fn expect_emitted_approve_event(
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

pub fn expect_emitted_revoke_event(rt: &MockRuntime, from: Address, to: Address) {
    let event = to_actor_event(CreditRevoked::new(from, to)).unwrap();
    rt.expect_emitted_event(event);
}

pub fn expect_emitted_add_event(
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

pub fn check_approval_used<BS: Blockstore>(
    state: &State,
    store: &BS,
    caller: Address,
    sponsor: Address,
) {
    assert_ne!(caller, sponsor);
    let subscriber_account = state.get_account(&store, sponsor).unwrap().unwrap();
    let subscriber_approval = subscriber_account
        .approvals_to
        .hamt(store)
        .unwrap()
        .get(&caller)
        .unwrap()
        .unwrap();
    assert_eq!(
        subscriber_approval.credit_used,
        state.credits.credit_debited.clone() + subscriber_account.credit_committed.clone()
    );
    let origin_account = state.get_account(&store, caller).unwrap().unwrap();
    let origin_approval = origin_account
        .approvals_from
        .hamt(store)
        .unwrap()
        .get(&sponsor)
        .unwrap()
        .unwrap();
    assert_eq!(
        subscriber_approval.credit_used,
        &state.credits.credit_debited + &subscriber_account.credit_committed
    );
    assert_eq!(subscriber_approval.credit_used, origin_approval.credit_used);
}
