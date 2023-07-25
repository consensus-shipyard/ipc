// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use crate::checkpoint::CheckpointManager;
use crate::config::subnet::NetworkType;
use crate::config::Subnet;
use crate::lotus::client::LotusJsonRPCClient;
use crate::manager::fevm::FevmSubnetManager;
use crate::manager::{EthSubnetManager, LotusSubnetManager};
use anyhow::anyhow;
use ipc_identity::PersistentKeyStore;
use ipc_identity::Wallet;
use ipc_sdk::subnet_id::SubnetID;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

async fn parent_fevm_child_fvm(
    parent: &Subnet,
    child: &Subnet,
    fvm_wallet_store: Arc<RwLock<Wallet>>,
    evm_wallet_store: Arc<RwLock<PersistentKeyStore<ethers::types::Address>>>,
) -> anyhow::Result<Vec<Box<dyn CheckpointManager>>> {
    if parent.network_type() != NetworkType::Fevm || child.network_type() != NetworkType::Fvm {
        return Err(anyhow!("parent not fevm or child not fvm"));
    }

    let mut managers = vec![];

    let fevm = FevmSubnetManager::new(
        EthSubnetManager::from_subnet_with_wallet_store(parent, evm_wallet_store.clone())?,
        LotusJsonRPCClient::from_subnet_with_wallet_store(child, fvm_wallet_store.clone()),
    );
    let fvm = LotusSubnetManager::new(
        LotusJsonRPCClient::from_subnet_with_wallet_store(child, fvm_wallet_store.clone()),
        child.gateway_addr(),
    );
    let m: Box<dyn CheckpointManager> = Box::new(
        crate::checkpoint::bottomup::BottomUpManager::new(parent.clone(), child.clone(), fevm, fvm)
            .await?,
    );

    managers.push(m);

    let fevm = FevmSubnetManager::new(
        EthSubnetManager::from_subnet_with_wallet_store(parent, evm_wallet_store.clone())?,
        LotusJsonRPCClient::from_subnet_with_wallet_store(child, fvm_wallet_store.clone()),
    );
    let fvm = LotusSubnetManager::new(
        LotusJsonRPCClient::from_subnet_with_wallet_store(child, fvm_wallet_store.clone()),
        child.gateway_addr(),
    );
    let m: Box<dyn CheckpointManager> = Box::new(
        crate::checkpoint::topdown::TopDownManager::new(parent.clone(), child.clone(), fevm, fvm)
            .await?,
    );

    managers.push(m);

    Ok(managers)
}

async fn parent_fvm_child_fvm(
    parent: &Subnet,
    child: &Subnet,
    fvm_wallet_store: Arc<RwLock<Wallet>>,
) -> anyhow::Result<Vec<Box<dyn CheckpointManager>>> {
    if parent.network_type() != NetworkType::Fevm || child.network_type() != NetworkType::Fvm {
        return Err(anyhow!("parent not fevm or child not fvm"));
    }

    let mut managers = vec![];

    let fvm_p = LotusSubnetManager::new(
        LotusJsonRPCClient::from_subnet_with_wallet_store(parent, fvm_wallet_store.clone()),
        parent.gateway_addr(),
    );
    let fvm_c = LotusSubnetManager::new(
        LotusJsonRPCClient::from_subnet_with_wallet_store(child, fvm_wallet_store.clone()),
        child.gateway_addr(),
    );
    let m: Box<dyn CheckpointManager> = Box::new(
        crate::checkpoint::bottomup::BottomUpManager::new(
            parent.clone(),
            child.clone(),
            fvm_p,
            fvm_c,
        )
        .await?,
    );

    managers.push(m);

    let fvm_p = LotusSubnetManager::new(
        LotusJsonRPCClient::from_subnet_with_wallet_store(parent, fvm_wallet_store.clone()),
        parent.gateway_addr(),
    );
    let fvm_c = LotusSubnetManager::new(
        LotusJsonRPCClient::from_subnet_with_wallet_store(child, fvm_wallet_store),
        child.gateway_addr(),
    );
    let m: Box<dyn CheckpointManager> = Box::new(
        crate::checkpoint::topdown::TopDownManager::new(
            parent.clone(),
            child.clone(),
            fvm_p,
            fvm_c,
        )
        .await?,
    );

    managers.push(m);

    Ok(managers)
}

pub async fn setup_manager_from_subnet(
    subnets: &HashMap<SubnetID, Subnet>,
    s: &Subnet,
    fvm_wallet_store: Arc<RwLock<Wallet>>,
    evm_wallet_store: Arc<RwLock<PersistentKeyStore<ethers::types::Address>>>,
) -> anyhow::Result<Vec<Box<dyn CheckpointManager>>> {
    let parent = if let Some(p) = s.id.parent() && subnets.contains_key(&p) {
        subnets.get(&p).unwrap()
    } else {
        log::info!("subnet has no parent configured: {:}, not managing checkpoints", s.id);
        return Ok(vec![]);
    };

    match (parent.network_type(), s.network_type()) {
        (NetworkType::Fvm, NetworkType::Fvm) => {
            log::info!("setup parent: {:?} fvm, child: {:?} fvm", parent.id, s.id);
            parent_fvm_child_fvm(parent, s, fvm_wallet_store).await
        }
        (NetworkType::Fvm, NetworkType::Fevm) => {
            unimplemented!()
        }
        (NetworkType::Fevm, NetworkType::Fvm) => {
            log::info!("setup parent: {:?} fevm, child: {:?} fvm", parent.id, s.id);
            parent_fevm_child_fvm(parent, s, fvm_wallet_store, evm_wallet_store).await
        }
        (NetworkType::Fevm, NetworkType::Fevm) => {
            todo!()
        }
    }
}

pub async fn setup_managers_from_config(
    subnets: &HashMap<SubnetID, Subnet>,
    fvm_wallet_store: Arc<RwLock<Wallet>>,
    evm_wallet_store: Arc<RwLock<PersistentKeyStore<ethers::types::Address>>>,
) -> anyhow::Result<Vec<Box<dyn CheckpointManager>>> {
    let mut managers = vec![];

    for s in subnets.values() {
        log::info!("config checkpoint manager for subnet: {:}", s.id);

        let subnet_managers = setup_manager_from_subnet(
            subnets,
            s,
            fvm_wallet_store.clone(),
            evm_wallet_store.clone(),
        )
        .await?;
        managers.extend(subnet_managers);
    }

    for m in managers.iter() {
        log::info!("we are managing checkpoints with: {m:}");
    }

    Ok(managers)
}
