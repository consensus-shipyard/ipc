// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::{anyhow, Context};
use ethers::types as et;
use ethers::{abi::Tokenize, utils::keccak256};

use fvm_ipld_blockstore::Blockstore;
use fvm_shared::ActorID;

use fendermint_crypto::SecretKey;
use fendermint_vm_actor_interface::{
    eam::EthAddress,
    ipc::{ValidatorMerkleTree, GATEWAY_ACTOR_ID},
};
use fendermint_vm_genesis::Validator;
use fendermint_vm_ipc_actors::gateway_getter_facet::{GatewayGetterFacet, SubnetID};
use fendermint_vm_ipc_actors::gateway_router_facet::{BottomUpCheckpoint, GatewayRouterFacet};
use fendermint_vm_message::signed::sign_secp256k1;

use super::{
    fevm::{ContractCaller, MockProvider},
    FvmExecState,
};

#[derive(Clone)]
pub struct GatewayCaller<DB> {
    addr: EthAddress,
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
            addr,
            getter: ContractCaller::new(addr, GatewayGetterFacet::new),
            router: ContractCaller::new(addr, GatewayRouterFacet::new),
        }
    }

    pub fn addr(&self) -> EthAddress {
        self.addr
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
        checkpoint: BottomUpCheckpoint,
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

    /// Construct the input parameters for adding a signature to the checkpoint.
    ///
    /// This will need to be broadcasted as a transaction.
    pub fn add_checkpoint_signature_calldata(
        &self,
        checkpoint: BottomUpCheckpoint,
        power_table: &[Validator],
        validator: &Validator,
        secret_key: &SecretKey,
    ) -> anyhow::Result<et::Bytes> {
        debug_assert_eq!(validator.public_key.0, secret_key.public_key());

        let height = checkpoint.block_height;
        let weight = et::U256::from(validator.power.0);

        let hash = keccak256(ethers::abi::encode(&checkpoint.into_tokens()));
        let signature = et::Bytes::from(sign_secp256k1(secret_key, &hash));

        let tree =
            ValidatorMerkleTree::new(power_table).context("failed to construct Merkle tree")?;

        let membership_proof = tree
            .prove(validator)
            .context("failed to construct Merkle proof")?
            .into_iter()
            .map(|p| p.into())
            .collect();

        let call = self.router.contract().add_checkpoint_signature(
            height,
            membership_proof,
            weight,
            signature,
        );

        let calldata = call
            .calldata()
            .ok_or_else(|| anyhow!("no calldata for adding signature"))?;

        Ok(calldata)
    }
}
