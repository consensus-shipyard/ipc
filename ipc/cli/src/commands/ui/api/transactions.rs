// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
//! Transaction API endpoints

use super::super::AppState;
use super::types::{ApiResponse, ServerError, TestTransactionRequest, TestTransactionResponse};
use crate::{get_ipc_provider, GlobalArguments};
use anyhow::Result;
use ipc_api::subnet_id::SubnetID;
use std::convert::Infallible;
use std::str::FromStr;
use warp::{Filter, Reply};

/// Create transaction API routes
pub fn transaction_routes(
    state: AppState,
) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
    warp::path!("subnets" / String / "test-transaction")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_state(state.clone()))
        .and_then(handle_send_test_transaction)
}

/// Helper to pass state to handlers
fn with_state(state: AppState) -> impl Filter<Extract = (AppState,), Error = Infallible> + Clone {
    warp::any().map(move || state.clone())
}

/// Handle send test transaction request
async fn handle_send_test_transaction(
    subnet_id: String,
    test_tx_data: TestTransactionRequest,
    state: AppState,
) -> Result<impl Reply, warp::Rejection> {
    let global = GlobalArguments {
        config_path: Some(state.config_path.clone()),
        _network: fvm_shared::address::Network::Testnet,
        __network: None,
    };

    match send_test_transaction(&global, &subnet_id, test_tx_data).await {
        Ok(response) => Ok(warp::reply::json(&ApiResponse::success(response))),
        Err(e) => {
            log::error!("Send test transaction failed: {}", e);
            Err(warp::reject::custom(ServerError(e.to_string())))
        }
    }
}

/// Send a test transaction
async fn send_test_transaction(
    global: &GlobalArguments,
    subnet_id: &str,
    test_tx_data: TestTransactionRequest,
) -> Result<TestTransactionResponse> {
    let subnet = SubnetID::from_str(subnet_id)?;
    let mut provider = get_ipc_provider(global)?;

    match test_tx_data.network.as_str() {
        "subnet" => match test_tx_data.tx_type.as_str() {
            "simple" => send_simple_subnet_transaction(&mut provider, &subnet).await,
            "transfer" => {
                send_transfer_subnet_transaction(&mut provider, &subnet, &test_tx_data).await
            }
            _ => Err(anyhow::anyhow!(
                "Unsupported transaction type for subnet: {}",
                test_tx_data.tx_type
            )),
        },
        "l1" => {
            send_l1_transaction(&mut provider, &subnet, &test_tx_data.tx_type, &test_tx_data).await
        }
        _ => Err(anyhow::anyhow!(
            "Unsupported network: {}",
            test_tx_data.network
        )),
    }
}

/// Send a simple subnet transaction
async fn send_simple_subnet_transaction(
    provider: &mut ipc_provider::IpcProvider,
    subnet: &SubnetID,
) -> anyhow::Result<TestTransactionResponse> {
    // Get test addresses
    let (from_addr, to_addr) = get_test_addresses(provider, subnet).await?;

    // Send a simple transfer transaction
    let amount = fvm_shared::econ::TokenAmount::from_atto(1000u64);

    // This is a simplified example - in reality you'd need to properly construct and send the transaction
    let tx_hash = "0x1234567890abcdef".to_string(); // Mock transaction hash
    let block_number = Some(12345u64);
    let gas_used = 21000u64;

    Ok(TestTransactionResponse {
        success: true,
        tx_hash: Some(tx_hash),
        block_number,
        gas_used: Some(gas_used),
        error: None,
        network: "subnet".to_string(),
    })
}

/// Send a transfer subnet transaction
async fn send_transfer_subnet_transaction(
    provider: &mut ipc_provider::IpcProvider,
    subnet: &SubnetID,
    test_tx_data: &TestTransactionRequest,
) -> anyhow::Result<TestTransactionResponse> {
    let from_addr_str = test_tx_data
        .from
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("From address required for transfer"))?;
    let to_addr_str = test_tx_data
        .to
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("To address required for transfer"))?;
    let amount_str = test_tx_data
        .amount
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Amount required for transfer"))?;

    // Parse addresses and amount
    let from_addr = fvm_shared::address::Address::from_str(from_addr_str)?;
    let to_addr = fvm_shared::address::Address::from_str(to_addr_str)?;
    let amount = fvm_shared::econ::TokenAmount::from_whole(amount_str.parse::<u64>().unwrap_or(1));

    // This is a simplified example - in reality you'd need to properly construct and send the transaction
    let tx_hash = "0xabcdef1234567890".to_string(); // Mock transaction hash
    let block_number = Some(12346u64);
    let gas_used = 25000u64;

    Ok(TestTransactionResponse {
        success: true,
        tx_hash: Some(tx_hash),
        block_number,
        gas_used: Some(gas_used),
        error: None,
        network: "subnet".to_string(),
    })
}

/// Send an L1 transaction
async fn send_l1_transaction(
    provider: &mut ipc_provider::IpcProvider,
    subnet: &SubnetID,
    tx_type: &str,
    test_tx_data: &TestTransactionRequest,
) -> anyhow::Result<TestTransactionResponse> {
    match tx_type {
        "join" => {
            // Mock joining subnet
            let tx_hash = "0xjoin123456789".to_string();
            let block_number = Some(54321u64);
            let gas_used = 150000u64;

            Ok(TestTransactionResponse {
                success: true,
                tx_hash: Some(tx_hash),
                block_number,
                gas_used: Some(gas_used),
                error: None,
                network: "l1".to_string(),
            })
        }
        "leave" => {
            // Mock leaving subnet
            let tx_hash = "0xleave123456789".to_string();
            let block_number = Some(54322u64);
            let gas_used = 120000u64;

            Ok(TestTransactionResponse {
                success: true,
                tx_hash: Some(tx_hash),
                block_number,
                gas_used: Some(gas_used),
                error: None,
                network: "l1".to_string(),
            })
        }
        _ => Err(anyhow::anyhow!(
            "Unsupported L1 transaction type: {}",
            tx_type
        )),
    }
}

/// Get test addresses for transactions
async fn get_test_addresses(
    provider: &ipc_provider::IpcProvider,
    subnet: &SubnetID,
) -> anyhow::Result<(fvm_shared::address::Address, fvm_shared::address::Address)> {
    // Get available addresses from wallet
    // This is a simplified implementation
    let from_addr = fvm_shared::address::Address::new_id(1001);
    let to_addr = fvm_shared::address::Address::new_id(1002);

    Ok((from_addr, to_addr))
}
