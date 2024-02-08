// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::fvm::state::FvmGenesisState;
use crate::fvm::store::memory::MemoryBlockstore;
use anyhow::{anyhow, Context};
use cid::multihash::Code;
use cid::Cid;
use fendermint_actors::Manifest as CustomActorManifest;
use fendermint_eth_hardhat::ContractSourceAndName;
use fendermint_vm_actor_interface::init::AddressMap;
use fendermint_vm_actor_interface::{account, burntfunds, cron, eam, init, reward, system};
use fendermint_vm_genesis::Genesis;
use fvm_ipld_blockstore::{Block, Blockstore};
use fvm_ipld_car::load_car_unchecked;
use fvm_ipld_encoding::{CborStore, DAG_CBOR};
use fvm_shared::econ::TokenAmount;
use fvm_shared::ActorID;
use num_traits::Zero;
use std::collections::BTreeSet;

type CodeByName = (String, Cid);

pub(crate) struct BuiltInActorDeployer<'a> {
    eth_builtin_ids: &'a BTreeSet<ActorID>,
    eth_libs: &'a Vec<ContractSourceAndName>,
}

impl<'a> BuiltInActorDeployer<'a> {
    pub fn new(
        eth_builtin_ids: &'a BTreeSet<ActorID>,
        eth_libs: &'a Vec<ContractSourceAndName>,
    ) -> Self {
        Self {
            eth_builtin_ids,
            eth_libs,
        }
    }

    pub async fn process_genesis<DB>(
        &self,
        state: &mut FvmGenesisState<DB>,
        genesis: &Genesis,
    ) -> anyhow::Result<AddressMap>
    where
        DB: Blockstore + 'static + Send + Sync + Clone,
    {
        let (mem_store, mut builtin) = self.load_builtin(&state.builtin_actor_bundle).await?;

        let (new_cid, eam_code_cid) = self
            .replace_eam_actor(state, &mem_store, &mut builtin)
            .await?;

        self.deploy_actors(state, new_cid, genesis, eam_code_cid)
    }

    fn deploy_actors<DB>(
        &self,
        state: &mut FvmGenesisState<DB>,
        builtin_actors_cid: Cid,
        genesis: &Genesis,
        eam_code_cid: Cid,
    ) -> anyhow::Result<AddressMap>
    where
        DB: Blockstore + 'static + Send + Sync + Clone,
    {
        // System actor
        state
            .create_builtin_actor(
                system::SYSTEM_ACTOR_CODE_ID,
                system::SYSTEM_ACTOR_ID,
                &system::State {
                    builtin_actors: builtin_actors_cid,
                },
                TokenAmount::zero(),
                None,
            )
            .context("failed to create system actor")?;

        // Init actor
        let (init_state, addr_to_id) = init::State::new(
            state.store(),
            genesis.chain_name.clone(),
            &genesis.accounts,
            &self.eth_builtin_ids,
            self.eth_libs.len() as u64,
        )
        .context("failed to create init state")?;

        state
            .create_builtin_actor(
                init::INIT_ACTOR_CODE_ID,
                init::INIT_ACTOR_ID,
                &init_state,
                TokenAmount::zero(),
                None,
            )
            .context("failed to create init actor")?;

        // Cron actor
        state
            .create_builtin_actor(
                cron::CRON_ACTOR_CODE_ID,
                cron::CRON_ACTOR_ID,
                &cron::State {
                    entries: vec![], // TODO: Maybe with the IPC.
                },
                TokenAmount::zero(),
                None,
            )
            .context("failed to create cron actor")?;

        // Burnt funds actor (it's just an account).
        state
            .create_builtin_actor(
                account::ACCOUNT_ACTOR_CODE_ID,
                burntfunds::BURNT_FUNDS_ACTOR_ID,
                &account::State {
                    address: burntfunds::BURNT_FUNDS_ACTOR_ADDR,
                },
                TokenAmount::zero(),
                None,
            )
            .context("failed to create burnt funds actor")?;

        // A placeholder for the reward actor, beause I don't think
        // using the one in the builtin actors library would be appropriate.
        // This effectively burns the miner rewards. Better than panicking.
        state
            .create_builtin_actor(
                account::ACCOUNT_ACTOR_CODE_ID,
                reward::REWARD_ACTOR_ID,
                &account::State {
                    address: reward::REWARD_ACTOR_ADDR,
                },
                TokenAmount::zero(),
                None,
            )
            .context("failed to create reward actor")?;

        let eam_state = fendermint_actor_eam::State::new(
            &state.store(),
            genesis
                .contract_deployers
                .clone()
                .into_iter()
                .map(|a| a.0)
                .collect(),
        )?;
        state
            .create_actor_internal(
                eam_code_cid,
                eam::EAM_ACTOR_ID,
                &eam_state,
                TokenAmount::zero(),
                None,
            )
            .context("failed to create EAM actor")?;

        Ok(addr_to_id)
    }

    async fn replace_eam_actor<DB>(
        &self,
        state: &mut FvmGenesisState<DB>,
        store: &MemoryBlockstore,
        builtin: &mut Vec<CodeByName>,
    ) -> anyhow::Result<(Cid, Cid)>
    where
        DB: Blockstore + 'static + Send + Sync + Clone,
    {
        let (custom_manifest_version, custom_manifest_data_cid): (u32, Cid) =
            parse_bundle(&store, &state.custom_actor_bundle).await?;
        let custom_actor_manifest =
            CustomActorManifest::load(&store, &custom_manifest_data_cid, custom_manifest_version)?;

        let code = custom_actor_manifest
            .code_by_name(fendermint_actor_eam::IPC_EAM_ACTOR_NAME)
            .ok_or_else(|| anyhow!("ipc eam actor not found"))?;

        for (_, code_cid) in builtin.iter_mut().filter(|(name, _)| name == "eam") {
            *code_cid = *code
        }

        let data = fvm_ipld_encoding::to_vec(&builtin)?;
        let new_root_cid = store.put(
            Code::Blake2b256,
            &Block {
                codec: DAG_CBOR,
                data,
            },
        )?;

        store.copy_to(state.store())?;

        Ok((new_root_cid, *code))
    }

    async fn load_builtin(
        &self,
        builtin_actor_bundle: &[u8],
    ) -> anyhow::Result<(MemoryBlockstore, Vec<CodeByName>)> {
        let mem_store = MemoryBlockstore::new();

        // Load the builtin actor bundle.
        let (ver, root_cid): (u32, Cid) = parse_bundle(&mem_store, builtin_actor_bundle).await?;

        if ver != 1 {
            return Err(anyhow!("unsupported manifest version {}", ver));
        }

        let vec: Vec<CodeByName> = match mem_store.get_cbor(&root_cid)? {
            Some(vec) => vec,
            None => {
                return Err(anyhow!("cannot find manifest root cid {}", root_cid));
            }
        };

        Ok((mem_store, vec))
    }
}

async fn parse_bundle<DB: Blockstore>(store: &DB, bundle: &[u8]) -> anyhow::Result<(u32, Cid)> {
    let bundle_roots = load_car_unchecked(&store, bundle).await?;
    let bundle_root = match bundle_roots.as_slice() {
        [root] => root,
        roots => {
            return Err(anyhow!(
                "expected one root in builtin actor bundle; got {}",
                roots.len()
            ))
        }
    };

    let (manifest_version, manifest_data_cid): (u32, Cid) = match store.get_cbor(bundle_root)? {
        Some(vd) => vd,
        None => {
            return Err(anyhow!(
                "no manifest information in bundle root {}",
                bundle_root
            ))
        }
    };

    Ok((manifest_version, manifest_data_cid))
}
