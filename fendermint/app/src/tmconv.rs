// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Conversions to Tendermint data types.
use fendermint_vm_genesis::Validator;
use fendermint_vm_interpreter::{
    fvm::{FvmApplyRet, FvmCheckRet, FvmQueryRet},
    Timestamp,
};
use fvm_shared::{error::ExitCode, event::StampedEvent};
use std::num::NonZeroU32;
use tendermint::abci::{response, Code, Event, EventAttribute};

use crate::{app::AppError, BlockHeight};

/// IPLD encoding of data types we know we must be able to encode.
macro_rules! must_encode {
    ($var:ident) => {
        fvm_ipld_encoding::to_vec(&$var)
            .unwrap_or_else(|e| panic!("error encoding {}: {}", stringify!($var), e))
    };
}

/// Response to delivery where the input was blatantly invalid.
/// This indicates that the validator who made the block was Byzantine.
pub fn invalid_deliver_tx(err: AppError, description: String) -> response::DeliverTx {
    response::DeliverTx {
        code: Code::Err(NonZeroU32::try_from(err as u32).expect("error codes are non-zero")),
        info: description,
        ..Default::default()
    }
}

/// Response to checks where the input was blatantly invalid.
/// This indicates that the user who sent the transaction is either attacking or has a faulty client.
pub fn invalid_check_tx(err: AppError, description: String) -> response::CheckTx {
    response::CheckTx {
        code: Code::Err(NonZeroU32::try_from(err as u32).expect("error codes are non-zero")),
        info: description,
        ..Default::default()
    }
}

/// Response to queries where the input was blatantly invalid.
pub fn invalid_query(err: AppError, description: String) -> response::Query {
    response::Query {
        code: Code::Err(NonZeroU32::try_from(err as u32).expect("error codes are non-zero")),
        info: description,
        ..Default::default()
    }
}

pub fn to_deliver_tx(ret: FvmApplyRet) -> response::DeliverTx {
    let receipt = ret.apply_ret.msg_receipt;
    let code = to_code(receipt.exit_code);

    // Based on the sanity check in the `DefaultExecutor`.
    // gas_cost = gas_fee_cap * gas_limit; this is how much the account is charged up front.
    // &base_fee_burn + &over_estimation_burn + &refund + &miner_tip == gas_cost
    // But that's in tokens. I guess the closes to what we want is the limit.
    let gas_wanted: i64 = ret.gas_limit.try_into().unwrap_or(i64::MAX);
    let gas_used: i64 = receipt.gas_used.try_into().unwrap_or(i64::MAX);

    let data: bytes::Bytes = receipt.return_data.to_vec().into();
    let events = to_events("message", ret.apply_ret.events);

    response::DeliverTx {
        code,
        data,
        log: Default::default(),
        info: ret
            .apply_ret
            .failure_info
            .map(|i| i.to_string())
            .unwrap_or_default(),
        gas_wanted,
        gas_used,
        events,
        codespace: Default::default(),
    }
}

pub fn to_check_tx(ret: FvmCheckRet) -> response::CheckTx {
    response::CheckTx {
        code: to_code(ret.exit_code),
        gas_wanted: ret.gas_limit.try_into().unwrap_or(i64::MAX),
        sender: ret.sender.to_string(),
        ..Default::default()
    }
}

pub fn to_code(exit_code: ExitCode) -> Code {
    if exit_code.is_success() {
        Code::Ok
    } else {
        Code::Err(NonZeroU32::try_from(exit_code.value()).expect("error codes are non-zero"))
    }
}

/// Map the return values from epoch boundary operations to validator updates.
///
/// (Currently just a placeholder).
pub fn to_end_block(_ret: ()) -> response::EndBlock {
    response::EndBlock {
        validator_updates: Vec::new(),
        consensus_param_updates: None,
        events: Vec::new(),
    }
}

/// Map the return values from cron operations.
pub fn to_begin_block(ret: FvmApplyRet) -> response::BeginBlock {
    let events = to_events("begin", ret.apply_ret.events);

    response::BeginBlock { events }
}

/// Convert events to key-value pairs.
pub fn to_events(kind: &str, stamped_events: Vec<StampedEvent>) -> Vec<Event> {
    stamped_events
        .into_iter()
        .map(|se| {
            let mut attrs = Vec::new();

            attrs.push(EventAttribute {
                key: "emitter".to_string(),
                value: se.emitter.to_string(),
                index: true,
            });

            for e in se.event.entries {
                attrs.push(EventAttribute {
                    key: e.key,
                    value: hex::encode(e.value),
                    index: !e.flags.is_empty(),
                });
            }

            Event::new(kind.to_string(), attrs)
        })
        .collect()
}

/// Map to query results.
pub fn to_query(ret: FvmQueryRet, block_height: BlockHeight) -> response::Query {
    let exit_code = match ret {
        FvmQueryRet::Ipld(None) | FvmQueryRet::ActorState(None) => ExitCode::USR_NOT_FOUND,
        FvmQueryRet::Ipld(_) | FvmQueryRet::ActorState(_) => ExitCode::OK,
    };

    // The return value has a `key` field which is supposed to be set to the data matched.
    // Although at this point I don't have access to the input like the CID looked up,
    // but I assume the query sender has. Rather than repeat everything, I'll add the key
    // where it gives some extra information, like the actor ID, just to keep this option visible.
    let (key, value) = match ret {
        FvmQueryRet::Ipld(None) | FvmQueryRet::ActorState(None) => (Vec::new(), Vec::new()),
        FvmQueryRet::Ipld(Some(bz)) => (Vec::new(), bz),
        FvmQueryRet::ActorState(Some(x)) => {
            let (id, st) = *x;
            let k = must_encode!(id);
            let v = must_encode!(st);
            (k, v)
        }
    };

    // The height here is the height of the block that was committed, not in which the app hash appeared,
    // so according to Tendermint docstrings we need to return plus one.
    let height = tendermint::block::Height::try_from(block_height + 1).expect("height too big");

    response::Query {
        code: to_code(exit_code),
        key: key.into(),
        value: value.into(),
        height,
        ..Default::default()
    }
}

/// Project Genesis validators to Tendermint.
pub fn to_validator_updates(
    validators: Vec<Validator>,
) -> anyhow::Result<Vec<tendermint::validator::Update>> {
    let mut updates = vec![];
    for v in validators {
        let bz = v.public_key.0.serialize();
        let key = k256::ecdsa::VerifyingKey::from_sec1_bytes(&bz)?;
        updates.push(tendermint::validator::Update {
            pub_key: tendermint::public_key::PublicKey::Secp256k1(key),
            power: tendermint::vote::Power::try_from(v.power.0)?,
        });
    }
    Ok(updates)
}

pub fn to_timestamp(time: tendermint::time::Time) -> Timestamp {
    Timestamp(
        time.unix_timestamp()
            .try_into()
            .expect("negative timestamp"),
    )
}
