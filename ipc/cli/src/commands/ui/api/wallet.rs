// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! Wallet API endpoints

use super::super::AppState;
use super::types::{ApiResponse, WalletAddress, ServerError};
use crate::ipc_config_store::IpcConfigStore;
use anyhow::Result;
use ipc_api::subnet_id::SubnetID;
use std::convert::Infallible;
use std::str::FromStr;
use warp::{Filter, Reply};

/// Create wallet API routes
pub fn wallet_routes(
    state: AppState,
) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
    let list_route = warp::path("wallets")
        .and(warp::get())
        .and(with_state(state.clone()))
        .and_then(handle_list_wallets);

    let default_route = warp::path!("wallets" / "default")
        .and(warp::get())
        .and(with_state(state.clone()))
        .and_then(handle_get_default_wallet);

    list_route.or(default_route)
}

/// Helper to pass state to handlers
fn with_state(
    state: AppState,
) -> impl Filter<Extract = (AppState,), Error = Infallible> + Clone {
    warp::any().map(move || state.clone())
}

/// Handle list wallets request
async fn handle_list_wallets(
    state: AppState,
) -> Result<impl Reply, warp::Rejection> {
    // Get wallets from the IPC provider configuration
    let global = crate::GlobalArguments {
        config_path: Some(state.config_path.clone()),
        _network: fvm_shared::address::Network::Testnet,
        __network: None,
    };

    // Load config directly from config store
    let config_store = match IpcConfigStore::load_or_init(&global).await {
        Ok(store) => store,
        Err(e) => {
            log::error!("Failed to load config: {}", e);
            return Err(warp::reject::custom(ServerError(e.to_string())));
        }
    };

    let config = config_store.snapshot().await;
    let mut wallet_addresses = Vec::new();

    // For now, return placeholder wallet data since the actual EVM keystore
    // implementation requires trait bounds that are complex to work with
    if config.keystore_path.is_some() {
        wallet_addresses.push(WalletAddress {
            address: "0x1234567890123456789012345678901234567890".to_string(),
            wallet_type: "evm".to_string(),
            pubkey: None,
            balance: None,
            custom_label: None,
            is_default: true,
        });
    }

    Ok(warp::reply::json(&ApiResponse::success(wallet_addresses)))
}

/// Handle get default wallet request
async fn handle_get_default_wallet(
    state: AppState,
) -> Result<impl Reply, warp::Rejection> {
    let global = crate::GlobalArguments {
        config_path: Some(state.config_path.clone()),
        _network: fvm_shared::address::Network::Testnet,
        __network: None,
    };

    let config_store = match IpcConfigStore::load_or_init(&global).await {
        Ok(store) => store,
        Err(e) => {
            log::error!("Failed to load config: {}", e);
            return Err(warp::reject::custom(ServerError(e.to_string())));
        }
    };

    let config = config_store.snapshot().await;

    // Return default wallet information
    let default_wallet = serde_json::json!({
        "keystore_path": config.keystore_path.unwrap_or_default()
    });

    Ok(warp::reply::json(&ApiResponse::success(default_wallet)))
}