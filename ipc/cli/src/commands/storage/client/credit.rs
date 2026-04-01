// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

//! Credit subcommand for buying and querying storage credits.

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use clap::{Args, Subcommand};
use ipc_api::evm::payload_to_evm_address;
use ipc_wallet::EvmKeyStore;
use num_traits::Zero;
use std::path::PathBuf;
use std::time::Duration;

use fendermint_actor_blobs_shared::{
    accounts::Account, method::Method as BlobsMethod, BLOBS_ACTOR_ADDR,
};
use ethers::abi::{encode as abi_encode, Token};
use fendermint_rpc::client::FendermintClient;
use fendermint_rpc::message::SignedMessageFactory;
use fendermint_rpc::tx::{TxClient, TxCommit};
use fendermint_rpc::QueryClient;
use fendermint_vm_actor_interface::eam::EthAddress;
use fendermint_vm_actor_interface::evm;
use fendermint_vm_message::query::FvmQueryHeight;
use fvm_ipld_encoding::{BytesSer, RawBytes};
use fvm_shared::address::Address;
use fvm_shared::chainid::ChainID;
use fvm_shared::econ::TokenAmount;

use crate::commands::storage::bucket;
use crate::commands::storage::client_context::resolve_write_context;
use crate::commands::storage::config::{
    resolve_client_config_path, resolve_provider_config_path, StorageClientConfig, StorageConfig,
};
use crate::{CommandLineHandler, GlobalArguments};

#[derive(Debug, Args)]
#[command(name = "credit", about = "Buy and query storage credits")]
pub struct CreditCommandArgs {
    #[command(subcommand)]
    command: CreditCommands,
}

#[derive(Debug, Subcommand)]
pub enum CreditCommands {
    /// Buy storage credits by sending tokens to the blobs actor
    Buy(BuyCreditArgs),
    /// Get account credit information
    Info(CreditInfoArgs),
}

impl CreditCommandArgs {
    pub async fn handle(&self, global: &GlobalArguments) -> anyhow::Result<()> {
        match &self.command {
            CreditCommands::Buy(args) => BuyCredit::handle(global, args).await,
            CreditCommands::Info(args) => CreditInfo::handle(global, args).await,
        }
    }
}

// ---------------------------------------------------------------------------
// Buy
// ---------------------------------------------------------------------------

#[derive(Debug, Args)]
pub struct BuyCreditArgs {
    /// Amount of tokens to spend (in FIL/ether units, e.g. 0.1)
    #[arg(value_name = "AMOUNT")]
    pub amount: f64,

    /// Recipient address (defaults to the operator key address)
    #[arg(long)]
    pub to: Option<String>,

    /// Storage client/provider config file
    #[arg(long)]
    pub config: Option<PathBuf>,
}

pub struct BuyCredit;

#[async_trait]
impl CommandLineHandler for BuyCredit {
    type Arguments = BuyCreditArgs;

    async fn handle(global: &GlobalArguments, args: &Self::Arguments) -> Result<()> {
        if args.amount <= 0.0 {
            return Err(anyhow!("Amount must be positive"));
        }

        let write_ctx = resolve_write_context(global, args.config.clone())?;
        let rpc_url = write_ctx.rpc_url;
        let secret_key = write_ctx.secret_key;

        let fm_client = FendermintClient::new_http(rpc_url.parse()?, None)?;

        let chain_id = bucket::query_chain_id(&fm_client)
            .await
            .context("Failed to query chain ID")?;

        let pub_key = secret_key.public_key();
        // Use delegated sender address for storage txs.
        let sender_eth = EthAddress::new_secp256k1(&pub_key.serialize())
            .context("failed to derive delegated sender address from signer key")?;
        let addr = Address::new_delegated(10, &sender_eth.0)
            .context("failed to construct delegated sender address")?;

        // Determine recipient — blobs actor requires a delegated (f410) address
        let recipient = if let Some(ref to_str) = args.to {
            crate::require_fil_addr_from_str(to_str)?
        } else {
            let eth_addr = EthAddress::new_secp256k1(&pub_key.serialize())
                .context("failed to derive delegated address from operator key")?;
            Address::new_delegated(10, &eth_addr.0).context("failed to construct f410 address")?
        };

        let state = fm_client
            .actor_state(&addr, FvmQueryHeight::default())
            .await
            .context("Failed to get actor state")?;
        let sequence = state.value.map(|(_, s)| s.sequence).ok_or_else(|| {
            anyhow!(
                "sender actor {} does not exist on-chain at {}. Fund/initialize this delegated \
                 address first (or use a signer key whose delegated address exists on this subnet).",
                addr,
                rpc_url
            )
        })?;

        let mf = SignedMessageFactory::new(secret_key, addr, sequence, ChainID::from(chain_id));
        let mut bound_client = fm_client.bind(mf);

        // The blobs actor is an EVM actor; invoke through InvokeContract with ABI calldata.
        let recipient_eth = payload_to_evm_address(recipient.payload())
            .context("BuyCredit recipient must be an EVM/delegated address")?;
        let mut calldata = Vec::with_capacity(4 + 32);
        // buyCredit(address) selector from credit facade ABI.
        calldata.extend_from_slice(&[0xa3, 0x8e, 0xae, 0x9f]);
        calldata.extend_from_slice(&abi_encode(&[Token::Address(recipient_eth)]));
        let invoke_params = RawBytes::serialize(BytesSer(&calldata))
            .context("Failed to serialize FEVM calldata for BuyCredit")?;

        // Convert amount to TokenAmount (nano precision)
        let value = crate::f64_to_token_amount(args.amount)?;
        let gas_params = bucket::tx_gas_params(
            &bound_client,
            addr,
            BLOBS_ACTOR_ADDR,
            evm::Method::InvokeContract as u64,
            invoke_params,
            value.clone(),
        )
        .await
        .context("Failed to estimate BuyCredit gas parameters")?;

        println!(
            "Buying credit for {} with {} FIL...",
            recipient, args.amount
        );

        let res = TxClient::<TxCommit>::fevm_invoke(
            &mut bound_client,
            BLOBS_ACTOR_ADDR,
            calldata.into(),
            value,
            gas_params,
        )
        .await
        .map_err(|e| {
            anyhow!(
                "Failed to send BuyCredit transaction: {} (sender={} recipient={} rpc={})",
                e,
                addr,
                recipient,
                rpc_url
            )
        })?;

        if res.response.check_tx.code.is_err() {
            let log = &res.response.check_tx.log;
            let info = &res.response.check_tx.info;
            return Err(anyhow!(
                "BuyCredit check_tx failed (code {:?}): log={} info={} sender={} recipient={} rpc={}",
                res.response.check_tx.code,
                if log.is_empty() { "<empty>" } else { log },
                if info.is_empty() { "<empty>" } else { info },
                addr,
                recipient,
                rpc_url,
            ));
        }

        if res.response.deliver_tx.code.is_err() {
            let log = &res.response.deliver_tx.log;
            let info = &res.response.deliver_tx.info;
            return Err(anyhow!(
                "BuyCredit deliver_tx failed (code {:?}): log={} info={} sender={} recipient={} rpc={}",
                res.response.deliver_tx.code,
                if log.is_empty() { "<empty>" } else { log },
                if info.is_empty() { "<empty>" } else { info },
                addr,
                recipient,
                rpc_url
            ));
        }

        println!("Credit purchased successfully for {}", recipient);

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Info
// ---------------------------------------------------------------------------

#[derive(Debug, Args)]
pub struct CreditInfoArgs {
    /// Account address to query (defaults to client-config/provider key when available)
    #[arg(long)]
    pub address: Option<String>,

    /// Storage client/provider config file
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Tendermint RPC URL (used when no storage config exists; overrides config when provided)
    #[arg(long)]
    pub rpc_url: Option<String>,

    /// Output in JSON format
    #[arg(long)]
    pub json: bool,
}

pub struct CreditInfo;

#[async_trait]
impl CommandLineHandler for CreditInfo {
    type Arguments = CreditInfoArgs;

    async fn handle(global: &GlobalArguments, args: &Self::Arguments) -> Result<()> {
        let client_config_path = resolve_client_config_path(args.config.clone());
        let provider_config_path = resolve_provider_config_path(args.config.clone());
        let mut default_query_addr = None;

        let (rpc_url, rpc_source) = if client_config_path.exists() {
            let client_cfg = StorageClientConfig::load(&client_config_path).with_context(|| {
                format!(
                    "failed to load client storage config at {}",
                    client_config_path.display()
                )
            })?;
            if args.address.is_none() {
                if let Some(addr) = client_cfg.address {
                    default_query_addr = Some(crate::require_fil_addr_from_str(&addr)?);
                }
            }
            if let Some(url) = args.rpc_url.clone() {
                (url, "--rpc-url".to_string())
            } else {
                (
                    client_cfg.tendermint_rpc_url,
                    format!("storage client config '{}'", client_config_path.display()),
                )
            }
        } else if provider_config_path.exists() {
            let config = StorageConfig::load(&provider_config_path).with_context(|| {
                format!(
                    "failed to load provider storage config at {}",
                    provider_config_path.display()
                )
            })?;
            if args.address.is_none() {
                let secret_key = SignedMessageFactory::read_secret_key(&config.secret_key_file)
                    .with_context(|| {
                        format!(
                            "failed to read provider secret key from {}",
                            config.secret_key_file.display()
                        )
                    })?;
                // Keep query defaults consistent with write paths: use delegated (f410) address.
                let eth_addr = EthAddress::new_secp256k1(&secret_key.public_key().serialize())
                    .context("failed to derive delegated address from provider secret key")?;
                default_query_addr = Some(
                    Address::new_delegated(10, &eth_addr.0)
                        .context("failed to construct delegated query address")?,
                );
            }
            if let Some(url) = args.rpc_url.clone() {
                (url, "--rpc-url".to_string())
            } else {
                (
                    config.tendermint_rpc_url,
                    format!("storage provider config '{}'", provider_config_path.display()),
                )
            }
        } else {
            if args.address.is_none() {
                let provider = crate::commands::get_ipc_provider(global).context(
                    "failed to load IPC provider config to infer default wallet address",
                )?;
                if let Ok(wallet) = provider.evm_wallet() {
                    let mut wallet = wallet.write().unwrap();
                    if let Some(default_evm) = wallet.get_default()? {
                        let eth_addr: ethers::types::Address = default_evm.clone().into();
                        default_query_addr =
                            Some(ipc_api::ethers_address_to_fil_address(&eth_addr)?);
                    }
                }
            }
            if let Some(url) = args.rpc_url.clone() {
                (url, "--rpc-url".to_string())
            } else {
                (
                    "http://127.0.0.1:26657".to_string(),
                    "default localhost RPC".to_string(),
                )
            }
        };
        let rpc_endpoint = rpc_url.parse().with_context(|| {
            format!(
                "Invalid Tendermint RPC URL '{}' from {}",
                rpc_url, rpc_source
            )
        })?;
        let fm_client = FendermintClient::new_http(rpc_endpoint, None)?;

        // Determine the address to query.
        let query_addr = if let Some(ref addr_str) = args.address {
            crate::require_fil_addr_from_str(addr_str)?
        } else if let Some(addr) = default_query_addr {
            addr
        } else {
            return Err(anyhow!(
                "No default address available. For user mode, pass --address (and optionally --rpc-url). \
                 For provider mode, run 'ipc-cli storage node init' to generate storage config."
            ));
        };

        // Query the GetAccount method on the blobs actor
        let params_bytes =
            RawBytes::serialize(query_addr).context("Failed to serialize address")?;

        let msg = fvm_shared::message::Message {
            version: Default::default(),
            from: fendermint_vm_actor_interface::system::SYSTEM_ACTOR_ADDR,
            to: BLOBS_ACTOR_ADDR,
            sequence: 0,
            value: TokenAmount::zero(),
            method_num: BlobsMethod::GetAccount as u64,
            params: params_bytes,
            gas_limit: 10_000_000_000,
            gas_fee_cap: TokenAmount::zero(),
            gas_premium: TokenAmount::zero(),
        };

        let response = tokio::time::timeout(
            Duration::from_secs(15),
            fm_client.call(msg, FvmQueryHeight::default()),
        )
        .await
        .map_err(|_| {
            anyhow!(
                "Timed out after 15s querying GetAccount via Tendermint RPC at {}",
                rpc_url
            )
        })?
        .with_context(|| {
            format!(
                "Failed to query GetAccount (address={} rpc={})",
                query_addr, rpc_url
            )
        })?;

        if response.value.code.is_err() {
            return Err(anyhow!(
                "GetAccount query failed (code {:?}): log={} info={} (address={} rpc={})",
                response.value.code,
                if response.value.log.is_empty() {
                    "<empty>"
                } else {
                    &response.value.log
                },
                if response.value.info.is_empty() {
                    "<empty>"
                } else {
                    &response.value.info
                },
                query_addr,
                rpc_url
            ));
        }

        let return_data = fendermint_rpc::response::decode_data(&response.value.data)
            .context("Failed to decode response data")?;

        let account: Option<Account> =
            fvm_ipld_encoding::from_slice(&return_data).context("Failed to decode Account")?;

        match account {
            Some(acct) => {
                if args.json {
                    let output = serde_json::json!({
                        "address": query_addr.to_string(),
                        "capacity_used": acct.capacity_used,
                        "credit_free": acct.credit_free.atto().to_string(),
                        "credit_committed": acct.credit_committed.atto().to_string(),
                        "credit_sponsor": acct.credit_sponsor.map(|a| a.to_string()),
                        "last_debit_epoch": acct.last_debit_epoch,
                        "max_ttl": acct.max_ttl,
                        "gas_allowance": acct.gas_allowance.atto().to_string(),
                        "approvals_to": acct.approvals_to.len(),
                        "approvals_from": acct.approvals_from.len(),
                    });
                    println!("{}", serde_json::to_string_pretty(&output)?);
                } else {
                    println!("Account: {}", query_addr);
                    println!("  Capacity used:    {} bytes", acct.capacity_used);
                    println!("  Credit free:      {}", acct.credit_free);
                    println!("  Credit committed: {}", acct.credit_committed);
                    if let Some(sponsor) = &acct.credit_sponsor {
                        println!("  Credit sponsor:   {}", sponsor);
                    }
                    println!("  Last debit epoch: {}", acct.last_debit_epoch);
                    println!("  Max TTL:          {} epochs", acct.max_ttl);
                    println!("  Gas allowance:    {}", acct.gas_allowance);
                    println!(
                        "  Approvals:        {} outgoing, {} incoming",
                        acct.approvals_to.len(),
                        acct.approvals_from.len()
                    );
                }
            }
            None => {
                println!("No account found for {}", query_addr);
            }
        }

        Ok(())
    }
}
