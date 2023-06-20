// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use async_trait::async_trait;
use fendermint_vm_actor_interface::{cron, eam, init, system, EMPTY_ARR};
use fendermint_vm_core::{chainid, Timestamp};
use fendermint_vm_genesis::{ActorMeta, Genesis, Validator};
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::chainid::ChainID;
use fvm_shared::econ::TokenAmount;
use fvm_shared::version::NetworkVersion;
use num_traits::Zero;

use crate::GenesisInterpreter;

use super::state::FvmGenesisState;
use super::FvmMessageInterpreter;

pub struct FvmGenesisOutput {
    pub chain_id: ChainID,
    pub timestamp: Timestamp,
    pub network_version: NetworkVersion,
    pub base_fee: TokenAmount,
    pub circ_supply: TokenAmount,
    pub validators: Vec<Validator>,
}

#[async_trait]
impl<DB> GenesisInterpreter for FvmMessageInterpreter<DB>
where
    DB: Blockstore + 'static + Send + Sync + Clone,
{
    type State = FvmGenesisState<DB>;
    type Genesis = Genesis;
    type Output = FvmGenesisOutput;

    /// Initialize actor states from the Genesis spec.
    ///
    /// This method doesn't create all builtin Filecoin actors,
    /// it leaves out the ones specific to file storage.
    ///
    /// The ones included are:
    /// * system
    /// * init
    /// * cron
    /// * EAM
    ///
    /// TODO:
    /// * burnt funds?
    /// * faucet?
    /// * IPC
    ///
    /// See [Lotus](https://github.com/filecoin-project/lotus/blob/v1.20.4/chain/gen/genesis/genesis.go) for reference
    /// and the [ref-fvm tester](https://github.com/filecoin-project/ref-fvm/blob/fvm%40v3.1.0/testing/integration/src/tester.rs#L99-L103).
    async fn init(
        &self,
        mut state: Self::State,
        genesis: Self::Genesis,
    ) -> anyhow::Result<(Self::State, Self::Output)> {
        // NOTE: We could consider adding the chain ID to the interpreter
        //       and rejecting genesis if it doesn't match the expectation,
        //       but the Tendermint genesis file also has this field, and
        //       presumably Tendermint checks that its peers have the same.
        let chain_id = chainid::from_str_hashed(&genesis.chain_name)?;

        // Currently we just pass them back as they are, but later we should
        // store them in the IPC actors; or in case of a snapshot restore them
        // from the state.
        let output = FvmGenesisOutput {
            chain_id,
            timestamp: genesis.timestamp,
            network_version: genesis.network_version,
            circ_supply: circ_supply(&genesis),
            base_fee: genesis.base_fee,
            validators: genesis.validators,
        };

        // System actor
        let system_state = system::State {
            builtin_actors: state.manifest_data_cid,
        };
        state.create_actor(
            system::SYSTEM_ACTOR_CODE_ID,
            system::SYSTEM_ACTOR_ID,
            &system_state,
            TokenAmount::zero(),
            None,
        )?;

        // Init actor
        let (init_state, addr_to_id) =
            init::State::new(state.store(), genesis.chain_name.clone(), &genesis.accounts)?;

        state.create_actor(
            init::INIT_ACTOR_CODE_ID,
            init::INIT_ACTOR_ID,
            &init_state,
            TokenAmount::zero(),
            None,
        )?;

        // Cron actor
        let cron_state = cron::State {
            entries: vec![], // TODO: Maybe with the IPC.
        };
        state.create_actor(
            cron::CRON_ACTOR_CODE_ID,
            cron::CRON_ACTOR_ID,
            &cron_state,
            TokenAmount::zero(),
            None,
        )?;

        // Ethereum Account Manager (EAM) actor
        state.create_actor(
            eam::EAM_ACTOR_CODE_ID,
            eam::EAM_ACTOR_ID,
            &EMPTY_ARR,
            TokenAmount::zero(),
            None,
        )?;

        // Create accounts
        let mut next_id = init::FIRST_NON_SINGLETON_ADDR + addr_to_id.len() as u64;
        for a in genesis.accounts {
            let balance = a.balance;
            match a.meta {
                ActorMeta::Account(acct) => {
                    state.create_account_actor(acct, balance, &addr_to_id)?;
                }
                ActorMeta::Multisig(ms) => {
                    state.create_multisig_actor(ms, balance, &addr_to_id, next_id)?;
                    next_id += 1;
                }
            }
        }

        Ok((state, output))
    }
}

fn circ_supply(g: &Genesis) -> TokenAmount {
    g.accounts
        .iter()
        .fold(TokenAmount::zero(), |s, a| s + a.balance.clone())
}

#[cfg(test)]
mod tests {
    use fendermint_vm_genesis::Genesis;
    use quickcheck::Arbitrary;

    use crate::{
        fvm::{bundle::bundle_path, store::memory::MemoryBlockstore, FvmMessageInterpreter},
        GenesisInterpreter,
    };

    use super::FvmGenesisState;

    #[tokio::test]
    async fn load_genesis() {
        let mut g = quickcheck::Gen::new(5);
        let genesis = Genesis::arbitrary(&mut g);
        let bundle = std::fs::read(bundle_path()).expect("failed to read bundle");
        let store = MemoryBlockstore::new();

        let state = FvmGenesisState::new(store, &bundle)
            .await
            .expect("failed to create state");

        let interpreter = FvmMessageInterpreter::new();

        let (state, out) = interpreter
            .init(state, genesis.clone())
            .await
            .expect("failed to create actors");

        let _state_root = state.commit().expect("failed to commit");
        assert_eq!(out.validators, genesis.validators);
    }
}
