#!/usr/bin/env rust-script

//! Filecoin Lotus RPC Connection Test
//!
//! This script tests connectivity to Filecoin Lotus RPC endpoints
//! like the IPC Calibration testnet.
//!
//! ```cargo
//! [dependencies]
//! tokio = { version = "1", features = ["full"] }
//! clap = { version = "4", features = ["derive"] }
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1.0"
//! reqwest = { version = "0.11", features = ["json"] }
//! anyhow = "1.0"
//! ```

use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Parser)]
#[command(name = "test_lotus_connection")]
#[command(about = "Test connection to Filecoin Lotus RPC endpoint")]
struct Args {
    #[arg(short, long, default_value = "https://filecoin-calibration.ipc.space/rpc/v1")]
    endpoint: String,

    #[arg(short, long)]
    verbose: bool,
}

#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    id: i32,
    jsonrpc: String,
    result: Option<Value>,
    error: Option<JsonRpcError>,
}

#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    id: i32,
    jsonrpc: String,
    method: String,
    params: Vec<Value>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    println!("🔍 Testing Filecoin Lotus RPC Connection");
    println!("📡 Endpoint: {}", args.endpoint);
    println!();

    let client = reqwest::Client::new();

    // Test 1: Version
    println!("1. Testing Filecoin version...");
    match test_version(&client, &args.endpoint).await {
        Ok(info) => {
            println!("   ✅ Connection successful!");
            println!("   📋 Version: {}", info.get("version").unwrap_or(&Value::String("unknown".to_string())));
            println!("   🔗 API Version: {}", info.get("api_version").unwrap_or(&Value::String("unknown".to_string())));
        }
        Err(e) => {
            println!("   ❌ Connection failed: {}", e);
            if e.to_string().contains("EnableEthRPC") || e.to_string().contains("LOTUS_FEVM_ENABLEETHRPC") {
                println!("   💡 Solution: Enable EthRPC on your node:");
                println!("      For Lotus: export LOTUS_FEVM_ENABLEETHRPC=1");
                println!("      For Fendermint: add [eth] enabled = true to config");
                println!("      Then restart your node");
            }
            return Err(e);
        }
    }

    // Test 2: Chain Head
    println!("\n2. Testing chain head...");
    match test_chain_head(&client, &args.endpoint).await {
        Ok(info) => {
            println!("   ✅ Chain head retrieved!");
            println!("   📦 Latest Height: {}", info.get("height").unwrap_or(&Value::Number(0.into())));
            println!("   🔗 Block Count: {}", info.get("block_count").unwrap_or(&Value::Number(0.into())));
        }
        Err(e) => {
            println!("   ⚠️  Chain head failed: {}", e);
        }
    }

    // Test 3: Network Name
    println!("\n3. Testing network name...");
    match test_network_name(&client, &args.endpoint).await {
        Ok(network_name) => {
            println!("   ✅ Network name retrieved!");
            println!("   🌐 Network: {}", network_name);
        }
        Err(e) => {
            println!("   ⚠️  Network name failed: {}", e);
        }
    }

    // Test 4: Wallet List (if available)
    println!("\n4. Testing wallet access...");
    match test_wallet_list(&client, &args.endpoint).await {
        Ok(wallet_count) => {
            println!("   ✅ Wallet access successful!");
            println!("   💰 Wallets available: {}", wallet_count);
        }
        Err(e) => {
            println!("   ⚠️  Wallet access failed: {}", e);
            println!("   📝 Note: This is normal for public endpoints");
        }
    }

    // Test 5: ETH RPC (if enabled)
    println!("\n5. Testing ETH RPC interface...");
    match test_eth_rpc(&client, &args.endpoint).await {
        Ok(info) => {
            println!("   ✅ ETH RPC is enabled!");
            println!("   ⚡ Chain ID: {}", info.get("chain_id").unwrap_or(&Value::String("unknown".to_string())));
            println!("   📊 Latest Block: {}", info.get("block_number").unwrap_or(&Value::String("unknown".to_string())));
        }
        Err(e) => {
            println!("   ❌ ETH RPC failed: {}", e);
            if e.to_string().contains("EnableEthRPC") || e.to_string().contains("module disabled") {
                println!("   💡 ETH RPC is disabled. Enable with:");
                println!("      export LOTUS_FEVM_ENABLEETHRPC=1");
            }
        }
    }

    println!("\n🎉 Filecoin connection test complete!");
    println!("\n📝 Summary:");
    println!("   • Filecoin node is accessible at: {}", args.endpoint);
    println!("   • Node is responding to Lotus RPC calls");

    println!("\n🚀 Next steps:");
    println!("   1. If ETH RPC is enabled, you can use Ethereum-style transactions");
    println!("   2. For native Filecoin transactions, use Lotus RPC methods");
    println!("   3. For benchmarking, determine which interface to use");

    Ok(())
}

async fn test_version(client: &reqwest::Client, endpoint: &str) -> Result<HashMap<String, Value>> {
    let request = JsonRpcRequest {
        id: 1,
        jsonrpc: "2.0".to_string(),
        method: "Filecoin.Version".to_string(),
        params: vec![],
    };

    let response = client
        .post(endpoint)
        .json(&request)
        .send()
        .await
        .context("Failed to send version request")?;

    let rpc_response: JsonRpcResponse = response
        .json()
        .await
        .context("Failed to parse version response")?;

    if let Some(error) = rpc_response.error {
        return Err(anyhow::anyhow!("RPC error: {}", error.message));
    }

    let result = rpc_response.result.context("No result in response")?;

    let mut info = HashMap::new();
    info.insert("version".to_string(), result.get("Version").unwrap_or(&Value::String("unknown".to_string())).clone());
    info.insert("api_version".to_string(), result.get("APIVersion").unwrap_or(&Value::String("unknown".to_string())).clone());

    Ok(info)
}

async fn test_chain_head(client: &reqwest::Client, endpoint: &str) -> Result<HashMap<String, Value>> {
    let request = JsonRpcRequest {
        id: 2,
        jsonrpc: "2.0".to_string(),
        method: "Filecoin.ChainHead".to_string(),
        params: vec![],
    };

    let response = client
        .post(endpoint)
        .json(&request)
        .send()
        .await
        .context("Failed to send chain head request")?;

    let rpc_response: JsonRpcResponse = response
        .json()
        .await
        .context("Failed to parse chain head response")?;

    if let Some(error) = rpc_response.error {
        return Err(anyhow::anyhow!("RPC error: {}", error.message));
    }

    let result = rpc_response.result.context("No result in response")?;

    let mut info = HashMap::new();
    info.insert("height".to_string(), result.get("Height").unwrap_or(&Value::Number(0.into())).clone());

    if let Some(blocks) = result.get("Blocks") {
        if let Some(blocks_array) = blocks.as_array() {
            info.insert("block_count".to_string(), Value::Number(blocks_array.len().into()));
        }
    }

    Ok(info)
}

async fn test_network_name(client: &reqwest::Client, endpoint: &str) -> Result<String> {
    let request = JsonRpcRequest {
        id: 3,
        jsonrpc: "2.0".to_string(),
        method: "Filecoin.StateNetworkName".to_string(),
        params: vec![],
    };

    let response = client
        .post(endpoint)
        .json(&request)
        .send()
        .await
        .context("Failed to send network name request")?;

    let rpc_response: JsonRpcResponse = response
        .json()
        .await
        .context("Failed to parse network name response")?;

    if let Some(error) = rpc_response.error {
        return Err(anyhow::anyhow!("RPC error: {}", error.message));
    }

    let result = rpc_response.result.context("No result in response")?;

    Ok(result.as_str().unwrap_or("unknown").to_string())
}

async fn test_wallet_list(client: &reqwest::Client, endpoint: &str) -> Result<usize> {
    let request = JsonRpcRequest {
        id: 4,
        jsonrpc: "2.0".to_string(),
        method: "Filecoin.WalletList".to_string(),
        params: vec![],
    };

    let response = client
        .post(endpoint)
        .json(&request)
        .send()
        .await
        .context("Failed to send wallet list request")?;

    let rpc_response: JsonRpcResponse = response
        .json()
        .await
        .context("Failed to parse wallet list response")?;

    if let Some(error) = rpc_response.error {
        return Err(anyhow::anyhow!("RPC error: {}", error.message));
    }

    let result = rpc_response.result.context("No result in response")?;

    if let Some(wallets) = result.as_array() {
        Ok(wallets.len())
    } else {
        Ok(0)
    }
}

async fn test_eth_rpc(client: &reqwest::Client, endpoint: &str) -> Result<HashMap<String, Value>> {
    // Test eth_chainId
    let request = JsonRpcRequest {
        id: 5,
        jsonrpc: "2.0".to_string(),
        method: "eth_chainId".to_string(),
        params: vec![],
    };

    let response = client
        .post(endpoint)
        .json(&request)
        .send()
        .await
        .context("Failed to send eth_chainId request")?;

    let rpc_response: JsonRpcResponse = response
        .json()
        .await
        .context("Failed to parse eth_chainId response")?;

    if let Some(error) = rpc_response.error {
        return Err(anyhow::anyhow!("RPC error: {}", error.message));
    }

    let result = rpc_response.result.context("No result in response")?;

    let mut info = HashMap::new();
    info.insert("chain_id".to_string(), result.clone());

    // Test eth_blockNumber
    let request2 = JsonRpcRequest {
        id: 6,
        jsonrpc: "2.0".to_string(),
        method: "eth_blockNumber".to_string(),
        params: vec![],
    };

    let response2 = client
        .post(endpoint)
        .json(&request2)
        .send()
        .await
        .context("Failed to send eth_blockNumber request")?;

    let rpc_response2: JsonRpcResponse = response2
        .json()
        .await
        .context("Failed to parse eth_blockNumber response")?;

    if let Some(result2) = rpc_response2.result {
        info.insert("block_number".to_string(), result2);
    }

    Ok(info)
}