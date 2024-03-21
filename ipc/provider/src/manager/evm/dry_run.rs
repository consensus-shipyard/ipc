// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use anyhow::anyhow;
use ethers::abi::{Function, FunctionExt, Tokenizable};
use ethers::types::{Address, Selector, U256};
use ethers_contract::encode_function_data;
use ipc_actors_abis::register_subnet_facet;
use ipc_api::evm::payload_to_evm_address;
use ipc_api::subnet::ConstructParams;
use serde::Serialize;
use std::collections::btree_map::BTreeMap;

#[derive(Serialize)]
pub struct MockedTxn {
    from: Address,
    to: Address,
    value: U256,
    pub calldata: Vec<u8>,
    method: Selector,
}

pub struct EvmDryRun;

impl EvmDryRun {
    pub fn create_subnet(
        &self,
        from: &fvm_shared::address::Address,
        params: ConstructParams,
    ) -> anyhow::Result<MockedTxn> {
        let converted = register_subnet_facet::ConstructorParams::try_from(params)?;

        let to = converted.ipc_gateway_addr;
        let from = payload_to_evm_address(from.payload())?;

        let (calldata, method) = to_evm_calldata(
            &register_subnet_facet::REGISTERSUBNETFACET_ABI.functions,
            "newSubnetActor",
            converted,
        )?;

        Ok(MockedTxn {
            from,
            to,
            value: U256::zero(),
            method,
            calldata,
        })
    }
}

fn to_evm_calldata<T: Tokenizable>(
    functions: &BTreeMap<String, Vec<Function>>,
    func_name: &str,
    args: T,
) -> anyhow::Result<(Vec<u8>, Selector)> {
    let func = functions
        .get(func_name)
        .ok_or_else(|| anyhow!("function {} not found in abi", func_name))?
        .first()
        .ok_or_else(|| anyhow!("function is empty, abi does not seem to be valid"))?;

    let selector = func.selector();
    let evm_data = encode_function_data(func, args)?.to_vec();

    Ok((evm_data, selector))
}
