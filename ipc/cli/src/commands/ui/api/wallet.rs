// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! Wallet API endpoints

use super::super::AppState;
use super::types::{ApiResponse, WalletAddress, ServerError};
use crate::{get_ipc_provider, GlobalArguments};
use anyhow::Result;
use ipc_api::{subnet_id::SubnetID, ethers_address_to_fil_address};
use ipc_wallet::{EthKeyAddress, EvmKeyStore};
use std::convert::Infallible;
use std::str::FromStr;
use warp::{Filter, Reply};

/// Create wallet API routes
pub fn wallet_routes(
    state: AppState,
) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
    let list_route = warp::path("wallets")
        .and(warp::get())
        .and(warp::query::<std::collections::HashMap<String, String>>())
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
    query: std::collections::HashMap<String, String>,
    state: AppState,
) -> Result<impl Reply, warp::Rejection> {
    let global = GlobalArguments {
        config_path: Some(state.config_path.clone()),
        _network: fvm_shared::address::Network::Testnet,
        __network: None,
    };

    let mut wallet_addresses = Vec::new();

    // Get IPC provider to access wallets
    let provider = match get_ipc_provider(&global) {
        Ok(provider) => provider,
        Err(e) => {
            log::error!("Failed to get IPC provider: {}", e);
            return Err(warp::reject::custom(ServerError(e.to_string())));
        }
    };

    // Fetch EVM wallet addresses
    match provider.evm_wallet() {
        Ok(wallet) => {
            // Get the list of addresses without holding the lock
            let addresses_result = {
                wallet.read().unwrap().list()
            };

            match addresses_result {
                Ok(addresses) => {
                    for address in addresses.iter() {
                        // Skip default key placeholder
                        if *address == EthKeyAddress::default() || address.to_string() == "default-key" {
                            continue;
                        }

                        // Get public key for the address without holding lock across await
                        let pubkey = {
                            match wallet.read().unwrap().get(address) {
                                Ok(Some(key_info)) => {
                                    match libsecp256k1::SecretKey::parse_slice(key_info.private_key()) {
                                        Ok(sk) => {
                                            let pub_key = libsecp256k1::PublicKey::from_secret_key(&sk);
                                            Some(hex::encode(pub_key.serialize()))
                                        }
                                        Err(_) => None,
                                    }
                                }
                                _ => None,
                            }
                        };

                        // Get balance if subnet specified
                        let balance = if let Some(subnet_str) = query.get("subnet") {
                            // Convert EVM address to FIL address for balance checking
                            match ethers_address_to_fil_address(&(address.clone()).into()) {
                                Ok(fil_addr) => {
                                    match SubnetID::from_str(subnet_str) {
                                        Ok(subnet_id) => {
                                            match provider.wallet_balance(&subnet_id, &fil_addr).await {
                                                Ok(amount) => Some(amount.to_string()),
                                                Err(_) => None,
                                            }
                                        }
                                        Err(_) => None,
                                    }
                                }
                                Err(_) => None,
                            }
                        } else {
                            None
                        };

                        wallet_addresses.push(WalletAddress {
                            address: address.to_string(),
                            wallet_type: "evm".to_string(),
                            pubkey,
                            balance,
                            custom_label: None,
                            is_default: wallet_addresses.is_empty(), // First one is default
                        });
                    }
                }
                Err(e) => {
                    log::warn!("Failed to list EVM wallet addresses: {}", e);
                }
            }
        }
        Err(e) => {
            log::warn!("Failed to get EVM wallet: {}", e);
        }
    }

    // Fetch FVM wallet addresses
    match provider.fvm_wallet() {
        Ok(wallet) => {
            // Get the list of addresses without holding the lock
            let addresses_result = {
                wallet.read().unwrap().list_addrs()
            };

            match addresses_result {
                Ok(addresses) => {
                    for address in addresses.iter() {
                        // Get public key for the address without holding lock across await
                        let pubkey = {
                            match wallet.write().unwrap().export(address) {
                                Ok(key_info) => {
                                    match libsecp256k1::SecretKey::parse_slice(key_info.private_key()) {
                                        Ok(sk) => {
                                            let pub_key = libsecp256k1::PublicKey::from_secret_key(&sk);
                                            Some(hex::encode(pub_key.serialize()))
                                        }
                                        Err(_) => None,
                                    }
                                }
                                Err(_) => None,
                            }
                        };

                        // Get balance if subnet specified
                        let balance = if let Some(subnet_str) = query.get("subnet") {
                            match SubnetID::from_str(subnet_str) {
                                Ok(subnet_id) => {
                                    match provider.wallet_balance(&subnet_id, address).await {
                                        Ok(amount) => Some(amount.to_string()),
                                        Err(_) => None,
                                    }
                                }
                                Err(_) => None,
                            }
                        } else {
                            None
                        };

                        wallet_addresses.push(WalletAddress {
                            address: address.to_string(),
                            wallet_type: "fvm".to_string(),
                            pubkey,
                            balance,
                            custom_label: None,
                            is_default: wallet_addresses.is_empty() && wallet_addresses.iter().all(|w| w.wallet_type != "fvm"),
                        });
                    }
                }
                Err(e) => {
                    log::warn!("Failed to list FVM wallet addresses: {}", e);
                }
            }
        }
        Err(e) => {
            log::warn!("Failed to get FVM wallet: {}", e);
        }
    }

    Ok(warp::reply::json(&ApiResponse::success(wallet_addresses)))
}

/// Handle get default wallet request
async fn handle_get_default_wallet(
    state: AppState,
) -> Result<impl Reply, warp::Rejection> {
    // Get the first available wallet address as default
    let query = std::collections::HashMap::new();
    match handle_list_wallets(query, state).await {
        Ok(reply) => {
            // Extract the first wallet as default
            Ok(reply)
        }
        Err(e) => Err(e)
    }
}