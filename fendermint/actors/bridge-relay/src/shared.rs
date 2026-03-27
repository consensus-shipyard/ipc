// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Shared types for the bridge-relay actor: state, params, return values, and method IDs.

use cid::Cid;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::tuple::{Deserialize_tuple, Serialize_tuple};
use fvm_ipld_hamt::{BytesKey, Hamt};
use fvm_shared::address::Address;
use fvm_shared::econ::TokenAmount;
use fvm_shared::METHOD_CONSTRUCTOR;
use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};

// ─── Constants ────────────────────────────────────────────────────────────────

pub const BRIDGE_RELAY_ACTOR_NAME: &str = "bridge-relay";

/// Bitwidth for the HAMT used to store processed transfer IDs.
pub const PROCESSED_HAMT_BITWIDTH: u32 = 5;

// ─── State ────────────────────────────────────────────────────────────────────

/// Persistent actor state stored in the FVM blockstore.
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct State {
    /// HAMT root CID mapping transferId (bytes32 as [u8;32]) → epoch processed.
    /// Used for replay protection.
    pub processed_transfers: Cid,

    /// Validation rules for incoming lock events.
    pub validation_rules: ValidationRules,

    /// The authorised BridgeLock contract address on Filecoin Calibration.
    pub bridge_lock_addr: Address,

    /// The BridgeMint contract address on Ethereum Sepolia (destination for relay).
    pub bridge_mint_addr: Address,

    /// Total number of transfers successfully relayed.
    pub relay_count: u64,

    /// Total number of transfers rejected (validation or replay failures).
    pub reject_count: u64,
}

impl State {
    /// Create a new State with an empty processed-transfers HAMT.
    pub fn new<BS: Blockstore>(
        store: &BS,
        bridge_lock_addr: Address,
        bridge_mint_addr: Address,
        validation_rules: ValidationRules,
    ) -> anyhow::Result<Self> {
        let mut empty_hamt: Hamt<_, (), BytesKey> =
            Hamt::new_with_bit_width(store, PROCESSED_HAMT_BITWIDTH);
        let processed_transfers = empty_hamt
            .flush()
            .map_err(|e| anyhow::anyhow!("failed to create processed-transfers HAMT: {e}"))?;

        Ok(Self {
            processed_transfers,
            validation_rules,
            bridge_lock_addr,
            bridge_mint_addr,
            relay_count: 0,
            reject_count: 0,
        })
    }

    /// Returns true if the transfer has already been processed (replay protection).
    pub fn is_processed<BS: Blockstore>(
        &self,
        store: &BS,
        transfer_id: &TransferId,
    ) -> anyhow::Result<bool> {
        let hamt: Hamt<_, u64, BytesKey> =
            Hamt::load_with_bit_width(&self.processed_transfers, store, PROCESSED_HAMT_BITWIDTH)
                .map_err(|e| anyhow::anyhow!("failed to load processed-transfers HAMT: {e}"))?;
        let key = transfer_id_key(transfer_id);
        Ok(hamt
            .get(&key)
            .map_err(|e| anyhow::anyhow!("HAMT get error: {e}"))?
            .is_some())
    }

    /// Mark a transfer as processed at the given epoch.
    pub fn mark_processed<BS: Blockstore>(
        &mut self,
        store: &BS,
        transfer_id: &TransferId,
        epoch: u64,
    ) -> anyhow::Result<()> {
        let mut hamt: Hamt<_, u64, BytesKey> =
            Hamt::load_with_bit_width(&self.processed_transfers, store, PROCESSED_HAMT_BITWIDTH)
                .map_err(|e| anyhow::anyhow!("failed to load processed-transfers HAMT: {e}"))?;
        let key = transfer_id_key(transfer_id);
        hamt.set(key, epoch)
            .map_err(|e| anyhow::anyhow!("HAMT set error: {e}"))?;
        self.processed_transfers = hamt
            .flush()
            .map_err(|e| anyhow::anyhow!("HAMT flush error: {e}"))?;
        Ok(())
    }
}

/// Convert a 32-byte transfer ID into a HAMT key (BytesKey).
fn transfer_id_key(id: &TransferId) -> BytesKey {
    BytesKey(id.to_vec())
}

// ─── Core types ───────────────────────────────────────────────────────────────

/// A 32-byte transfer identifier (matches BridgeLock's keccak256-derived transferId).
pub type TransferId = [u8; 32];

/// Represents a lock event emitted by BridgeLock.sol.
#[derive(Debug, Clone, Serialize_tuple, Deserialize_tuple)]
pub struct TokensLockedEvent {
    /// The ERC20 token address on Filecoin that was locked.
    pub token: Address,
    /// The sender (initiator of the lock) on Filecoin.
    pub sender: Address,
    /// The intended recipient on Ethereum.
    pub recipient: Address,
    /// The token amount locked (in the token's smallest unit).
    pub amount: TokenAmount,
    /// Unique transfer identifier for correlation and replay protection.
    pub transfer_id: TransferId,
}

// ─── Validation rules ─────────────────────────────────────────────────────────

/// Configurable validation rules applied to each incoming lock event.
#[derive(Debug, Clone, Serialize_tuple, Deserialize_tuple)]
pub struct ValidationRules {
    /// Minimum token amount allowed per transfer (0 = no minimum).
    pub min_amount: TokenAmount,
    /// Maximum token amount allowed per transfer (0 = no maximum).
    pub max_amount: TokenAmount,
    /// If non-empty, only tokens in this list are relayed.
    /// Stored as a sorted vec of addresses for deterministic iteration.
    pub allowed_tokens: Vec<Address>,
}

impl Default for ValidationRules {
    fn default() -> Self {
        Self {
            min_amount: TokenAmount::from_atto(0u64),
            max_amount: TokenAmount::from_atto(0u64),
            allowed_tokens: vec![],
        }
    }
}

impl ValidationRules {
    /// Validate a lock event against these rules.
    /// Returns Ok(()) if valid, Err(ValidationError) otherwise.
    pub fn validate(&self, event: &TokensLockedEvent) -> Result<(), ValidationError> {
        // Amount must be positive
        if event.amount <= TokenAmount::from_atto(0u64) {
            return Err(ValidationError::ZeroAmount);
        }

        // Minimum amount check
        if self.min_amount > TokenAmount::from_atto(0u64) && event.amount < self.min_amount {
            return Err(ValidationError::AmountBelowMinimum {
                amount: event.amount.clone(),
                min: self.min_amount.clone(),
            });
        }

        // Maximum amount check
        if self.max_amount > TokenAmount::from_atto(0u64) && event.amount > self.max_amount {
            return Err(ValidationError::AmountAboveMaximum {
                amount: event.amount.clone(),
                max: self.max_amount.clone(),
            });
        }

        // Token allowlist check
        if !self.allowed_tokens.is_empty() && !self.allowed_tokens.contains(&event.token) {
            return Err(ValidationError::TokenNotAllowed {
                token: event.token.clone(),
            });
        }

        // Recipient must not be the zero address (Address::default is the null address)
        if event.recipient == Address::new_id(0) {
            return Err(ValidationError::InvalidRecipient);
        }

        Ok(())
    }
}

// ─── Errors ──────────────────────────────────────────────────────────────────

/// Reasons why a lock event may be rejected by the actor.
#[derive(Debug, Clone, thiserror::Error, Serialize, Deserialize)]
pub enum ValidationError {
    #[error("transfer amount is zero")]
    ZeroAmount,
    #[error("transfer amount {amount} is below minimum {min}")]
    AmountBelowMinimum {
        amount: TokenAmount,
        min: TokenAmount,
    },
    #[error("transfer amount {amount} exceeds maximum {max}")]
    AmountAboveMaximum {
        amount: TokenAmount,
        max: TokenAmount,
    },
    #[error("token {token} is not in the allowlist")]
    TokenNotAllowed { token: Address },
    #[error("recipient address is invalid (zero address)")]
    InvalidRecipient,
    #[error("transfer {transfer_id:?} has already been processed (replay attempt)")]
    DuplicateTransfer { transfer_id: TransferId },
}

// ─── Method parameters and return types ──────────────────────────────────────

/// Constructor parameters.
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct ConstructorParams {
    /// BridgeLock contract address on Filecoin Calibration.
    pub bridge_lock_addr: Address,
    /// BridgeMint contract address on Ethereum Sepolia.
    pub bridge_mint_addr: Address,
    /// Initial validation rules.
    pub validation_rules: ValidationRules,
}

/// Parameters for the `RelayLockEvent` method.
/// Carries the decoded TokensLocked event from BridgeLock.sol.
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct RelayLockEventParams {
    pub event: TokensLockedEvent,
}

/// Return value from `RelayLockEvent`.
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct RelayLockEventReturn {
    /// True if the message was successfully relayed.
    pub success: bool,
    /// If rejected, contains the human-readable reason.
    pub rejection_reason: Option<String>,
}

/// Parameters for `UpdateValidationRules` (admin-only).
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct UpdateValidationRulesParams {
    pub rules: ValidationRules,
}

/// Parameters for `UpdateAddresses` (admin-only).
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct UpdateAddressesParams {
    pub bridge_lock_addr: Address,
    pub bridge_mint_addr: Address,
}

/// Parameters for `IsProcessed`.
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct IsProcessedParams {
    pub transfer_id: TransferId,
}

/// Return value from `GetStats`.
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct StatsReturn {
    pub relay_count: u64,
    pub reject_count: u64,
    pub bridge_lock_addr: Address,
    pub bridge_mint_addr: Address,
}

// ─── Method IDs ──────────────────────────────────────────────────────────────

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    /// Relay a TokensLocked event from BridgeLock to BridgeMint.
    RelayLockEvent = frc42_dispatch::method_hash!("RelayLockEvent"),
    /// Update the validation rules (admin only).
    UpdateValidationRules = frc42_dispatch::method_hash!("UpdateValidationRules"),
    /// Update BridgeLock / BridgeMint addresses (admin only).
    UpdateAddresses = frc42_dispatch::method_hash!("UpdateAddresses"),
    /// Return relay and reject counts.
    GetStats = frc42_dispatch::method_hash!("GetStats"),
    /// Check if a transferId has been processed.
    IsProcessed = frc42_dispatch::method_hash!("IsProcessed"),
}
