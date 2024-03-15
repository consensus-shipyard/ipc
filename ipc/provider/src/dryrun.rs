// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

//! Performs dry run for the ipc provider instead of directly submitter the txn on chain

use crate::preflight::Preflight;
use anyhow::anyhow;
use base64::Engine;
use ethers_contract::core::abi::{Function, Tokenizable};
use ethers_contract::encode_function_data;
use fvm_ipld_encoding::{BytesSer, RawBytes};
use ipc_actors_abis::register_subnet_facet;
use ipc_api::subnet::ConstructParams;
use std::collections::BTreeMap;

pub struct IPCDryRunProvider {
    pub(crate) preflight: Preflight,
}

impl IPCDryRunProvider {
    pub fn create_subnet(&self, params: ConstructParams) -> anyhow::Result<()> {
        let params = self.preflight.create_subnet(params)?;
        let params = register_subnet_facet::ConstructorParams::try_from(params)?;

        let data = to_fvm_calldata(
            &register_subnet_facet::REGISTERSUBNETFACET_ABI.functions,
            "newSubnetActor",
            params,
        )?;
        log::info!(
            "params: {}",
            base64::engine::general_purpose::STANDARD.encode(data)
        );

        Ok(())
    }
}

fn to_fvm_calldata<T: Tokenizable>(
    functions: &BTreeMap<String, Vec<Function>>,
    func_name: &str,
    args: T,
) -> anyhow::Result<Vec<u8>> {
    let func = functions
        .get(func_name)
        .ok_or_else(|| anyhow!("function {} not found in abi", func_name))?
        .first()
        .ok_or_else(|| anyhow!("function is empty, abi does not seem to be valid"))?;

    let evm_data = encode_function_data(func, args)?.to_vec();
    let params = RawBytes::serialize(BytesSer(&evm_data))?;
    Ok(params.to_vec())
}
