// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use super::{
    fevm::{ContractCaller, MockProvider},
    FvmExecState,
};
use anyhow::Context;
use ethers::types as et;
use fendermint_vm_actor_interface::{
    eam::EthAddress,
    ipc::{ValidatorMerkleTree, GATEWAY_ACTOR_ID},
};
use fendermint_vm_genesis::Validator;
use fendermint_vm_ipc_actors::gateway_getter_facet::{GatewayGetterFacet, SubnetID};
use fendermint_vm_ipc_actors::gateway_router_facet::{BottomUpCheckpointNew, GatewayRouterFacet};
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::ActorID;

#[derive(Clone)]
pub struct GatewayCaller<DB> {
    getter: ContractCaller<GatewayGetterFacet<MockProvider>, DB>,
    router: ContractCaller<GatewayRouterFacet<MockProvider>, DB>,
}

impl<DB> Default for GatewayCaller<DB> {
    fn default() -> Self {
        Self::new(GATEWAY_ACTOR_ID)
    }
}

impl<DB> GatewayCaller<DB> {
    pub fn new(gateway_actor_id: ActorID) -> Self {
        let addr = EthAddress::from_id(gateway_actor_id);
        Self {
            getter: ContractCaller::new(addr, GatewayGetterFacet::new),
            router: ContractCaller::new(addr, GatewayRouterFacet::new),
        }
    }
}

impl<DB: Blockstore> GatewayCaller<DB> {
    /// Check that IPC is configured in this deployment.
    pub fn enabled(&self, state: &mut FvmExecState<DB>) -> anyhow::Result<bool> {
        match state.state_tree_mut().get_actor(GATEWAY_ACTOR_ID)? {
            None => Ok(false),
            Some(a) => Ok(!state.builtin_actors().is_placeholder_actor(&a.code)),
        }
    }

    /// Return true if the current subnet is the root subnet.
    pub fn is_root(&self, state: &mut FvmExecState<DB>) -> anyhow::Result<bool> {
        self.subnet_id(state).map(|id| id.route.is_empty())
    }

    /// Return the current subnet ID.
    pub fn subnet_id(&self, state: &mut FvmExecState<DB>) -> anyhow::Result<SubnetID> {
        self.getter.call(state, |c| c.get_network_name())
    }

    /// Fetch the period with which the current subnet has to submit checkpoints to its parent.
    pub fn bottom_up_check_period(&self, state: &mut FvmExecState<DB>) -> anyhow::Result<u64> {
        self.getter.call(state, |c| c.bottom_up_check_period())
    }

    /// Insert a new checkpoint at the period boundary.
    pub fn create_bottom_up_checkpoint(
        &self,
        state: &mut FvmExecState<DB>,
        checkpoint: BottomUpCheckpointNew,
        power_table: &[Validator],
    ) -> anyhow::Result<()> {
        // Construct a Merkle tree from the power table, which we can use to validate validator set membership
        // when the signatures are submitted in transactions for accumulation.
        let tree =
            ValidatorMerkleTree::new(power_table).context("failed to create validator tree")?;

        let total_power = power_table.iter().fold(et::U256::zero(), |p, v| {
            p.saturating_add(et::U256::from(v.power.0))
        });

        self.router.call(state, |c| {
            c.create_bottom_up_checkpoint(checkpoint, tree.root_hash().0, total_power)
        })
    }
}
