// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT

pub mod bottomup;
pub mod topdown;

use crate::config::Subnet;
use crate::lotus::message::ipc::IPCReadGatewayStateResponse;
use crate::lotus::LotusClient;
use cid::Cid;
use fvm_shared::address::Address;
use std::str::FromStr;

pub async fn gateway_state(
    client: &(impl LotusClient + Sync),
    subnet: &Subnet,
) -> anyhow::Result<IPCReadGatewayStateResponse> {
    let child_head = client.chain_head().await?;
    let cid_map = child_head.cids.first().unwrap().clone();
    let child_tip_set = Cid::try_from(cid_map)?;

    client
        .ipc_read_gateway_state(&subnet.gateway_addr(), child_tip_set)
        .await
}

/// Returns the first cid in the chain head
pub(crate) async fn chain_head_cid(client: &(impl LotusClient + Sync)) -> anyhow::Result<Cid> {
    let child_head = client.chain_head().await?;
    let cid_map = child_head.cids.first().unwrap();
    Cid::try_from(cid_map)
}

/// Obtain the validators in the subnet from the parent subnet of the manager
pub(crate) async fn child_validators(
    parent_client: &(impl LotusClient + Sync),
    child_subnet: &Subnet,
) -> anyhow::Result<Vec<Address>> {
    let parent_head = parent_client.chain_head().await?;

    // A key assumption we make now is that each block has exactly one tip set. We panic
    // if this is not the case as it violates our assumption.
    // TODO: update this logic once the assumption changes (i.e., mainnet)
    assert_eq!(parent_head.cids.len(), 1);

    let cid_map = parent_head.cids.first().unwrap().clone();
    let parent_tip_set = Cid::try_from(cid_map)?;

    let subnet_actor_state = parent_client
        .ipc_read_subnet_actor_state(&child_subnet.id, parent_tip_set)
        .await?;

    match subnet_actor_state.validator_set.validators {
        None => Ok(vec![]),
        Some(validators) => {
            let mut vs = vec![];
            for v in validators {
                let addr = Address::from_str(&v.addr)?;
                if child_subnet.accounts().contains(&addr) {
                    vs.push(addr);
                }
            }
            Ok(vs)
        }
    }
}
