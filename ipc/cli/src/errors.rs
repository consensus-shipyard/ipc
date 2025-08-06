// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

//! User-friendly error types and messages for the IPC CLI
//!
//! This module provides structured error handling with clear, actionable messages
//! that help users understand what went wrong and how to fix it.

use std::fmt::{self, Display};
use std::path::PathBuf;
use thiserror::Error;

/// Main error type for IPC CLI operations
#[derive(Debug, Error)]
pub enum CliError {
    #[error(transparent)]
    Wallet(#[from] WalletError),
    
    #[error(transparent)]
    Subnet(#[from] SubnetError),
    
    #[error(transparent)]
    Node(#[from] NodeError),
    
    #[error(transparent)]
    Config(#[from] ConfigError),
    
    #[error(transparent)]
    CrossMsg(#[from] CrossMsgError),
    
    #[error(transparent)]
    Validation(#[from] ValidationError),
    
    #[error(transparent)]
    Network(#[from] NetworkError),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Wallet-related errors with user-friendly messages
#[derive(Debug, Error)]
pub enum WalletError {
    #[error("No wallet key found. Please import or create a wallet first using:\n  • ipc wallet new --wallet-type evm\n  • ipc wallet import --wallet-type evm --path <key-file>")]
    NoWallet,
    
    #[error("Cannot read from stdin yet. Please provide a key file using --path:\n  ipc wallet import --wallet-type {wallet_type} --path <key-file>")]
    StdinNotSupported { wallet_type: String },
    
    #[error("Cannot read key file at '{path}'.\nPlease check:\n  • The file exists\n  • You have read permissions\n  • The path is correct")]
    CannotReadKeyFile { path: PathBuf },
    
    #[error("Invalid key format for {wallet_type} wallet.\n{details}\nExpected format:\n{expected_format}")]
    InvalidKeyFormat {
        wallet_type: String,
        details: String,
        expected_format: String,
    },
    
    #[error("Wallet type '{wallet_type}' is not supported.\nSupported types are: evm, fvm")]
    UnsupportedWalletType { wallet_type: String },
    
    #[error("Private key operations are only supported for EVM wallets.\nFor FVM wallets, please provide a key file using --path")]
    PrivateKeyOnlyForEvm,
    
    #[error("FVM wallet requires a key type to be specified.\nPlease use --key-type with one of: secp256k1, bls, secp256k1-ledger")]
    FvmKeyTypeRequired,
    
    #[error("Wallet '{address}' not found in keystore.\nAvailable wallets:\n{available}")]
    WalletNotFound { address: String, available: String },
    
    #[error("Cannot set default wallet: no wallets in keystore.\nPlease import or create a wallet first.")]
    NoWalletsForDefault,
    
    #[error("Insufficient balance in wallet '{address}'.\nRequired: {required}\nAvailable: {available}")]
    InsufficientBalance {
        address: String,
        required: String,
        available: String,
    },
}

/// Subnet-related errors with user-friendly messages
#[derive(Debug, Error)]
pub enum SubnetError {
    #[error("Subnet '{subnet_id}' not found.\n{suggestion}")]
    SubnetNotFound { subnet_id: String, suggestion: String },
    
    #[error("Subnet '{subnet_id}' is not configured in your config.\nAdd it using: ipc config subnet --id {subnet_id}")]
    SubnetNotConfigured { subnet_id: String },
    
    #[error("Invalid subnet ID format: '{input}'.\nExpected format: /r<root-id>/f<subnet-id>[/f<subnet-id>...]\nExample: /r314159/f01234")]
    InvalidSubnetId { input: String },
    
    #[error("Cannot create subnet: parent subnet '{parent}' is not active.\nPlease ensure the parent subnet is running and reachable.")]
    ParentSubnetNotActive { parent: String },
    
    #[error("You are already a validator in subnet '{subnet_id}'.\nCurrent stake: {current_stake}")]
    AlreadyValidator { subnet_id: String, current_stake: String },
    
    #[error("Minimum collateral requirement not met for subnet '{subnet_id}'.\nRequired: {required}\nProvided: {provided}")]
    InsufficientCollateral {
        subnet_id: String,
        required: String,
        provided: String,
    },
    
    #[error("Cannot leave subnet '{subnet_id}': you have unclaimed collateral.\nPlease run: ipc subnet claim --subnet {subnet_id}")]
    UnclaimedCollateral { subnet_id: String },
    
    #[error("Subnet '{subnet_id}' is already activated.\nCurrent status: {status}")]
    SubnetAlreadyActive { subnet_id: String, status: String },
}

/// Node-related errors with user-friendly messages
#[derive(Debug, Error)]
pub enum NodeError {
    #[error("Node home directory '{path}' does not exist.\nCreate it using: mkdir -p {path}")]
    HomeDirectoryNotFound { path: PathBuf },
    
    #[error("Node home directory '{path}' is not a directory.\nPlease remove the file and create a directory instead.")]
    HomeNotDirectory { path: PathBuf },
    
    #[error("Node is not initialized in '{path}'.\nRequired file missing: {missing_file}\nRun: ipc node init --home {path}")]
    NodeNotInitialized { path: PathBuf, missing_file: String },
    
    #[error("CometBFT is not installed or not in PATH.\nPlease install CometBFT following the instructions at:\nhttps://docs.cometbft.com/v0.38/guides/install")]
    CometBftNotInstalled,
    
    #[error("Port {port} is already in use by another process.\n{service} cannot start.\nEither stop the other process or change the port in your configuration.")]
    PortInUse { port: u16, service: String },
    
    #[error("Cannot fetch peers from subnet '{subnet_id}'.\n{reason}\nYou can manually add peers using: ipc node add-peer")]
    CannotFetchPeers { subnet_id: String, reason: String },
    
    #[error("Invalid node configuration in '{path}'.\n{details}\nPlease check your configuration file.")]
    InvalidNodeConfig { path: PathBuf, details: String },
    
    #[error("Service '{service}' failed to start after {attempts} attempts.\nLast error: {last_error}\nCheck the logs for more details.")]
    ServiceStartFailed {
        service: String,
        attempts: u32,
        last_error: String,
    },
}

/// Configuration-related errors
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Configuration file not found at '{path}'.\nCreate a default config using: ipc config init")]
    ConfigNotFound { path: PathBuf },
    
    #[error("Invalid configuration file at '{path}'.\n{details}\nExample of valid config:\n{example}")]
    InvalidConfig {
        path: PathBuf,
        details: String,
        example: String,
    },
    
    #[error("Cannot write configuration to '{path}'.\nCheck that you have write permissions to the directory.")]
    CannotWriteConfig { path: PathBuf },
}

/// Cross-message errors
#[derive(Debug, Error)]
pub enum CrossMsgError {
    #[error("Invalid token amount: '{input}'.\nExpected format: <number> or <number>.<decimals>\nExamples: 100, 1.5, 0.001")]
    InvalidTokenAmount { input: String },
    
    #[error("Cross-message propagation failed.\n{reason}\nMake sure both subnets are active and synced.")]
    PropagationFailed { reason: String },
    
    #[error("No pending cross-messages to propagate between subnets.")]
    NoPendingMessages,
}

/// Validation errors
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Invalid Ethereum address: '{input}'.\nExpected format: 0x followed by 40 hexadecimal characters\nExample: 0x742d35Cc6634C0532925a3b844Bc9e7595f6fed7")]
    InvalidEthAddress { input: String },
    
    #[error("Invalid amount: '{input}'.\nAmount must be a positive number.")]
    InvalidAmount { input: String },
    
    #[error("Required parameter '{param}' is missing.\nPlease provide it using --{param}")]
    MissingParameter { param: String },
    
    #[error("Invalid parameter value for '{param}': '{value}'.\n{expected}")]
    InvalidParameter {
        param: String,
        value: String,
        expected: String,
    },
}

/// Network and connection errors
#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("Cannot connect to subnet RPC at '{endpoint}'.\nPlease check:\n  • The subnet is running\n  • The endpoint URL is correct\n  • Your network connection")]
    RpcConnectionFailed { endpoint: String },
    
    #[error("Request timed out after {seconds} seconds.\nThe subnet might be:\n  • Overloaded\n  • Syncing\n  • Unreachable\nTry again later or check the subnet status.")]
    RequestTimeout { seconds: u64 },
    
    #[error("Transaction failed: {reason}\nTransaction hash: {tx_hash}\nCheck the transaction status on the explorer.")]
    TransactionFailed { reason: String, tx_hash: String },
}

/// Helper trait to convert anyhow errors to our typed errors
pub trait IntoCliError {
    fn into_cli_error(self) -> CliError;
}

impl IntoCliError for anyhow::Error {
    fn into_cli_error(self) -> CliError {
        // Try to downcast to our known error types first
        if let Some(cli_err) = self.downcast_ref::<CliError>() {
            return cli_err.clone().into();
        }
        
        // Otherwise, wrap as internal error
        CliError::Internal(self.to_string())
    }
}

/// Extension trait for Result types to provide context-specific error messages
pub trait CliResultExt<T> {
    /// Add wallet-specific context to errors
    fn wallet_context(self, context: &str) -> Result<T, CliError>;
    
    /// Add subnet-specific context to errors
    fn subnet_context(self, context: &str) -> Result<T, CliError>;
    
    /// Add node-specific context to errors
    fn node_context(self, context: &str) -> Result<T, CliError>;
}

impl<T, E> CliResultExt<T> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn wallet_context(self, context: &str) -> Result<T, CliError> {
        self.map_err(|e| CliError::Internal(format!("{}: {}", context, e)))
    }
    
    fn subnet_context(self, context: &str) -> Result<T, CliError> {
        self.map_err(|e| CliError::Internal(format!("{}: {}", context, e)))
    }
    
    fn node_context(self, context: &str) -> Result<T, CliError> {
        self.map_err(|e| CliError::Internal(format!("{}: {}", context, e)))
    }
}

// Make CliError cloneable for the IntoCliError trait
impl Clone for CliError {
    fn clone(&self) -> Self {
        match self {
            CliError::Wallet(e) => CliError::Wallet(e.clone()),
            CliError::Subnet(e) => CliError::Subnet(e.clone()),
            CliError::Node(e) => CliError::Node(e.clone()),
            CliError::Config(e) => CliError::Config(e.clone()),
            CliError::CrossMsg(e) => CliError::CrossMsg(e.clone()),
            CliError::Validation(e) => CliError::Validation(e.clone()),
            CliError::Network(e) => CliError::Network(e.clone()),
            CliError::Internal(s) => CliError::Internal(s.clone()),
        }
    }
}

// Implement Clone for all error types
impl Clone for WalletError {
    fn clone(&self) -> Self {
        match self {
            WalletError::NoWallet => WalletError::NoWallet,
            WalletError::StdinNotSupported { wallet_type } => 
                WalletError::StdinNotSupported { wallet_type: wallet_type.clone() },
            WalletError::CannotReadKeyFile { path } => 
                WalletError::CannotReadKeyFile { path: path.clone() },
            WalletError::InvalidKeyFormat { wallet_type, details, expected_format } => 
                WalletError::InvalidKeyFormat { 
                    wallet_type: wallet_type.clone(), 
                    details: details.clone(), 
                    expected_format: expected_format.clone() 
                },
            WalletError::UnsupportedWalletType { wallet_type } => 
                WalletError::UnsupportedWalletType { wallet_type: wallet_type.clone() },
            WalletError::PrivateKeyOnlyForEvm => WalletError::PrivateKeyOnlyForEvm,
            WalletError::FvmKeyTypeRequired => WalletError::FvmKeyTypeRequired,
            WalletError::WalletNotFound { address, available } => 
                WalletError::WalletNotFound { address: address.clone(), available: available.clone() },
            WalletError::NoWalletsForDefault => WalletError::NoWalletsForDefault,
            WalletError::InsufficientBalance { address, required, available } => 
                WalletError::InsufficientBalance { 
                    address: address.clone(), 
                    required: required.clone(), 
                    available: available.clone() 
                },
        }
    }
}

// Implement Clone for other error types similarly...
impl Clone for SubnetError {
    fn clone(&self) -> Self {
        match self {
            SubnetError::SubnetNotFound { subnet_id, suggestion } => 
                SubnetError::SubnetNotFound { subnet_id: subnet_id.clone(), suggestion: suggestion.clone() },
            SubnetError::SubnetNotConfigured { subnet_id } => 
                SubnetError::SubnetNotConfigured { subnet_id: subnet_id.clone() },
            SubnetError::InvalidSubnetId { input } => 
                SubnetError::InvalidSubnetId { input: input.clone() },
            SubnetError::ParentSubnetNotActive { parent } => 
                SubnetError::ParentSubnetNotActive { parent: parent.clone() },
            SubnetError::AlreadyValidator { subnet_id, current_stake } => 
                SubnetError::AlreadyValidator { subnet_id: subnet_id.clone(), current_stake: current_stake.clone() },
            SubnetError::InsufficientCollateral { subnet_id, required, provided } => 
                SubnetError::InsufficientCollateral { 
                    subnet_id: subnet_id.clone(), 
                    required: required.clone(), 
                    provided: provided.clone() 
                },
            SubnetError::UnclaimedCollateral { subnet_id } => 
                SubnetError::UnclaimedCollateral { subnet_id: subnet_id.clone() },
            SubnetError::SubnetAlreadyActive { subnet_id, status } => 
                SubnetError::SubnetAlreadyActive { subnet_id: subnet_id.clone(), status: status.clone() },
        }
    }
}

impl Clone for NodeError {
    fn clone(&self) -> Self {
        match self {
            NodeError::HomeDirectoryNotFound { path } => 
                NodeError::HomeDirectoryNotFound { path: path.clone() },
            NodeError::HomeNotDirectory { path } => 
                NodeError::HomeNotDirectory { path: path.clone() },
            NodeError::NodeNotInitialized { path, missing_file } => 
                NodeError::NodeNotInitialized { path: path.clone(), missing_file: missing_file.clone() },
            NodeError::CometBftNotInstalled => NodeError::CometBftNotInstalled,
            NodeError::PortInUse { port, service } => 
                NodeError::PortInUse { port: *port, service: service.clone() },
            NodeError::CannotFetchPeers { subnet_id, reason } => 
                NodeError::CannotFetchPeers { subnet_id: subnet_id.clone(), reason: reason.clone() },
            NodeError::InvalidNodeConfig { path, details } => 
                NodeError::InvalidNodeConfig { path: path.clone(), details: details.clone() },
            NodeError::ServiceStartFailed { service, attempts, last_error } => 
                NodeError::ServiceStartFailed { 
                    service: service.clone(), 
                    attempts: *attempts, 
                    last_error: last_error.clone() 
                },
        }
    }
}

impl Clone for ConfigError {
    fn clone(&self) -> Self {
        match self {
            ConfigError::ConfigNotFound { path } => 
                ConfigError::ConfigNotFound { path: path.clone() },
            ConfigError::InvalidConfig { path, details, example } => 
                ConfigError::InvalidConfig { 
                    path: path.clone(), 
                    details: details.clone(), 
                    example: example.clone() 
                },
            ConfigError::CannotWriteConfig { path } => 
                ConfigError::CannotWriteConfig { path: path.clone() },
        }
    }
}

impl Clone for CrossMsgError {
    fn clone(&self) -> Self {
        match self {
            CrossMsgError::InvalidTokenAmount { input } => 
                CrossMsgError::InvalidTokenAmount { input: input.clone() },
            CrossMsgError::PropagationFailed { reason } => 
                CrossMsgError::PropagationFailed { reason: reason.clone() },
            CrossMsgError::NoPendingMessages => CrossMsgError::NoPendingMessages,
        }
    }
}

impl Clone for ValidationError {
    fn clone(&self) -> Self {
        match self {
            ValidationError::InvalidEthAddress { input } => 
                ValidationError::InvalidEthAddress { input: input.clone() },
            ValidationError::InvalidAmount { input } => 
                ValidationError::InvalidAmount { input: input.clone() },
            ValidationError::MissingParameter { param } => 
                ValidationError::MissingParameter { param: param.clone() },
            ValidationError::InvalidParameter { param, value, expected } => 
                ValidationError::InvalidParameter { 
                    param: param.clone(), 
                    value: value.clone(), 
                    expected: expected.clone() 
                },
        }
    }
}

impl Clone for NetworkError {
    fn clone(&self) -> Self {
        match self {
            NetworkError::RpcConnectionFailed { endpoint } => 
                NetworkError::RpcConnectionFailed { endpoint: endpoint.clone() },
            NetworkError::RequestTimeout { seconds } => 
                NetworkError::RequestTimeout { seconds: *seconds },
            NetworkError::TransactionFailed { reason, tx_hash } => 
                NetworkError::TransactionFailed { reason: reason.clone(), tx_hash: tx_hash.clone() },
        }
    }
}