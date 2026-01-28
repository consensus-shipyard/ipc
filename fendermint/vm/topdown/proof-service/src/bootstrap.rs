// Copyright 2022-2025 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! Bootstrap helpers for integrating the proof service with on-chain state.

use crate::config;
use crate::PowerEntries;
use anyhow::{Context, Result};
use ipc_api::subnet_id::SubnetID;
use num_bigint::Sign;

/// Fetch an F3 certificate for a specific instance from the parent chain.
///
/// Uses the F3 light client RPC (not the Lotus JSON-RPC wrapper), which supports fetching
/// certificates by instance ID.
pub async fn fetch_certificate(
    parent_rpc_url: &str,
    subnet_id: &SubnetID,
    instance_id: u64,
) -> Result<filecoin_f3_certs::FinalityCertificate> {
    let network = config::f3_network_name(subnet_id);
    let light_client = filecoin_f3_lightclient::LightClient::new(parent_rpc_url, &network)
        .context("failed to create F3 light client")?;
    light_client
        .get_certificate(instance_id)
        .await
        .context("failed to fetch F3 certificate by instance")
}

/// Convert the on-chain F3LightClientActor power table into GPBFT `PowerEntries`.
///
/// This preserves participant IDs, which are required for certificate verification.
pub fn power_entries_from_actor(
    entries: &[fendermint_actor_f3_light_client::types::PowerEntry],
) -> PowerEntries {
    PowerEntries(
        entries
            .iter()
            .map(|e| filecoin_f3_gpbft::PowerEntry {
                id: e.id,
                power: num_bigint::BigInt::from_bytes_be(Sign::Plus, &e.power_be),
                pub_key: filecoin_f3_gpbft::PubKey(e.public_key.clone()),
            })
            .collect(),
    )
}
