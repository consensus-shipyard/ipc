// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use super::{
    fevm::{ContractCaller, MockProvider},
    FvmExecState,
};
use fendermint_vm_actor_interface::{eam::EthAddress, ipc};
use fendermint_vm_ipc_actors::gateway_getter_facet::GatewayGetterFacet;
use fvm_ipld_blockstore::Blockstore;

pub struct GatewayCaller {
    getter: ContractCaller<GatewayGetterFacet<MockProvider>>,
}

impl GatewayCaller {
    pub fn new() -> Self {
        let addr = EthAddress::from_id(ipc::GATEWAY_ACTOR_ID);
        Self {
            getter: ContractCaller::new(addr, GatewayGetterFacet::new),
        }
    }

    pub fn bottom_up_check_period<DB: Blockstore>(
        &self,
        state: &mut FvmExecState<DB>,
    ) -> anyhow::Result<u64> {
        self.getter.call(state, |c| c.bottom_up_check_period())
    }
}

impl Default for GatewayCaller {
    fn default() -> Self {
        Self::new()
    }
}
