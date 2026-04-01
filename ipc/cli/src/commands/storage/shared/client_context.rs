// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use anyhow::{anyhow, Context, Result};
use fendermint_crypto::SecretKey;
use fvm_shared::address::Address;
use ipc_wallet::EvmKeyStore;
use std::path::PathBuf;
use std::str::FromStr;

use crate::commands::storage::config::{
    resolve_client_config_path, resolve_provider_config_path, StorageClientConfig, StorageConfig,
};
use crate::GlobalArguments;

pub struct WriteContext {
    pub rpc_url: String,
    pub secret_key: SecretKey,
}

pub fn resolve_rpc_url(config: Option<PathBuf>) -> Result<String> {
    let client_config_path = resolve_client_config_path(config.clone());
    let provider_config_path = resolve_provider_config_path(config);

    if client_config_path.exists() {
        let client_cfg = StorageClientConfig::load(&client_config_path).with_context(|| {
            format!(
                "failed to load client storage config at {}",
                client_config_path.display()
            )
        })?;
        if client_cfg.tendermint_rpc_url.trim().is_empty() {
            return Err(anyhow!(
                "client storage config at {} has empty tendermint-rpc-url",
                client_config_path.display()
            ));
        }
        Ok(client_cfg.tendermint_rpc_url)
    } else if provider_config_path.exists() {
        Ok(StorageConfig::load(&provider_config_path)
            .with_context(|| {
                format!(
                    "failed to load provider storage config at {}",
                    provider_config_path.display()
                )
            })?
            .tendermint_rpc_url)
    } else {
        Err(anyhow!(
            "No storage config found. Expected one of:\n\
             - client config: {}\n\
             - provider config: {}\n\
             Initialize client mode with 'ipc-cli storage client init ...' or pass --config.",
            client_config_path.display(),
            provider_config_path.display()
        ))
    }
}

pub fn resolve_default_owner_from_client_config(config: Option<PathBuf>) -> Result<Option<Address>> {
    let client_config_path = resolve_client_config_path(config);
    if !client_config_path.exists() {
        return Ok(None);
    }
    let client_cfg = StorageClientConfig::load(&client_config_path).with_context(|| {
        format!(
            "failed to load client storage config at {}",
            client_config_path.display()
        )
    })?;
    if let Some(addr) = client_cfg.address {
        return Ok(Some(crate::require_fil_addr_from_str(&addr)?));
    }
    Ok(None)
}

pub fn resolve_write_context(global: &GlobalArguments, config: Option<PathBuf>) -> Result<WriteContext> {
    let client_config_path = resolve_client_config_path(config.clone());
    let provider_config_path = resolve_provider_config_path(config);

    if client_config_path.exists() {
        let client_cfg = StorageClientConfig::load(&client_config_path).with_context(|| {
            format!(
                "failed to load client storage config at {}",
                client_config_path.display()
            )
        })?;
        if client_cfg.tendermint_rpc_url.trim().is_empty() {
            return Err(anyhow!(
                "client storage config at {} has empty tendermint-rpc-url",
                client_config_path.display()
            ));
        }

        let provider = crate::commands::get_ipc_provider(global)
            .context("failed to load IPC provider config to resolve client-mode signer key")?;
        let keystore = provider
            .evm_wallet()
            .context("failed to access EVM wallet for client-mode signer")?;
        let mut keystore = keystore.write().unwrap();

        let configured_evm = client_cfg
            .address
            .as_ref()
            .and_then(|s| ethers::types::Address::from_str(s).ok())
            .map(Into::into);
        let signer_evm = if let Some(addr) = configured_evm {
            Some(addr)
        } else {
            keystore
                .get_default()
                .context("failed to get default EVM wallet address")?
        }
        .ok_or_else(|| {
            anyhow!(
                "no signer key available in client mode: set `address` in storage client config \
                 to an EVM address present in your wallet, or set a default with \
                 'ipc-cli wallet set-default --wallet-type evm --address <0x...>'"
            )
        })?;
        let key_info = keystore
            .get(&signer_evm)
            .context("failed to load EVM wallet key for client-mode signer")?
            .ok_or_else(|| anyhow!("configured/default EVM wallet key {} not found", signer_evm))?;
        let secret_key = SecretKey::try_from(key_info.private_key().to_vec())
            .context("configured/default EVM key is not a valid secp256k1 key")?;

        Ok(WriteContext {
            rpc_url: client_cfg.tendermint_rpc_url,
            secret_key,
        })
    } else if provider_config_path.exists() {
        let cfg = StorageConfig::load(&provider_config_path).with_context(|| {
            format!(
                "failed to load provider storage config at {}",
                provider_config_path.display()
            )
        })?;
        let secret_key = fendermint_rpc::message::SignedMessageFactory::read_secret_key(
            &cfg.secret_key_file,
        )
        .with_context(|| {
            format!(
                "failed to read provider secret key from {}",
                cfg.secret_key_file.display()
            )
        })?;
        Ok(WriteContext {
            rpc_url: cfg.tendermint_rpc_url,
            secret_key,
        })
    } else {
        Err(anyhow!(
            "No storage config found. Expected one of:\n\
             - client config: {}\n\
             - provider config: {}\n\
             Initialize client mode with 'ipc-cli storage client init ...' or pass --config.",
            client_config_path.display(),
            provider_config_path.display()
        ))
    }
}
