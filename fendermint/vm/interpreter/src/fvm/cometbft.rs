// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::fvm::state::FvmExecState;
use crate::fvm::{FvmMessageInterpreter, PowerUpdates};
use anyhow::Context;
use fendermint_vm_genesis::{Power, Validator};
use fvm_ipld_blockstore::Blockstore;
use tendermint_rpc::Client;

/// The end block update for cometbft
pub struct EndBlockUpdate {
    pub consensus: Option<tendermint::consensus::Params>,
    pub validators: PowerUpdates,
}

/// Potential updates to cometbft consensus parameters. Currently only block `max_gas` needs to be
/// updated, but in the future, more parameter could be updated.
pub struct ConsensusBlockUpdate {
    max_gas: Option<u64>,
}

/// Convert validator power to tendermint validator update.
/// TODO: the import is quite strange, `Validator` and `Power` are imported from `genesis` crate,
/// TODO: which should be from a `type` or `validator` crate.
pub fn to_validator_updates(
    validators: Vec<Validator<Power>>,
) -> anyhow::Result<Vec<tendermint::validator::Update>> {
    let mut updates = vec![];
    for v in validators {
        updates.push(tendermint::validator::Update {
            pub_key: tendermint::PublicKey::try_from(v.public_key)?,
            power: tendermint::vote::Power::try_from(v.power.0)?,
        });
    }
    Ok(updates)
}

impl TryFrom<EndBlockUpdate> for tendermint::abci::response::EndBlock {
    type Error = anyhow::Error;

    fn try_from(value: EndBlockUpdate) -> Result<Self, Self::Error> {
        let validator_updates = to_validator_updates(value.validators.0)
            .context("failed to convert validator updates")?;

        Ok(tendermint::abci::response::EndBlock {
            validator_updates,
            consensus_param_updates: value.consensus,
            events: Vec::new(), // TODO: Events from epoch transitions?
        })
    }
}

impl<DB, TC> FvmMessageInterpreter<DB, TC>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
    TC: Client + Clone + Send + Sync + 'static,
{
    pub(crate) async fn update_cometbft_consensus_params(
        &self,
        state: &mut FvmExecState<DB>,
        end_block: &mut EndBlockUpdate,
    ) -> anyhow::Result<()> {
        let mut updates = ConsensusBlockUpdate::empty();

        state.gas_market().process_consensus_update(&mut updates);

        if !updates.is_some() {
            return Ok(());
        }

        let params = self
            .client
            .consensus_params(tendermint::block::Height::try_from(state.block_height())?)
            .await?
            .consensus_params;
        end_block.with_consensus(updates.apply(params));

        Ok(())
    }
}

impl ConsensusBlockUpdate {
    pub fn empty() -> Self {
        Self { max_gas: None }
    }

    pub fn is_some(&self) -> bool {
        self.max_gas.is_some()
    }

    pub fn apply(self, mut params: tendermint::consensus::Params) -> tendermint::consensus::Params {
        if let Some(ref max_gas) = self.max_gas {
            params.block.max_gas = *max_gas as i64;
        }
        params
    }

    pub fn process_block_size(&mut self, block_gas_limit: u64) {
        self.max_gas = Some(block_gas_limit);
    }
}

impl EndBlockUpdate {
    pub fn new(power: PowerUpdates) -> Self {
        Self {
            validators: power,
            consensus: None,
        }
    }

    pub fn with_consensus(&mut self, params: tendermint::consensus::Params) {
        self.consensus = Some(params)
    }
}
