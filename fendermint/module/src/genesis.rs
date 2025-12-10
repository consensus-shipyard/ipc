// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Genesis module trait for initializing module-specific actors.
//!
//! This trait allows modules to participate in genesis state creation
//! by initializing their own actors and state.

use anyhow::Result;
use cid::Cid;
use fendermint_vm_genesis::Genesis;
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::address::Address;
use fvm_shared::econ::TokenAmount;
use fvm_shared::ActorID;

/// State context provided to genesis modules.
///
/// This provides access to the state tree and other genesis parameters
/// that modules need to initialize their actors.
///
/// # Note on Generic Methods
///
/// This trait is generic over some type parameters, making it not directly
/// trait-object-safe. Implementations should use concrete types when
/// calling these methods.
pub trait GenesisState: Send + Sync {
    /// Get a reference to the blockstore
    fn blockstore(&self) -> &dyn Blockstore;

    /// Create a new actor in the state tree
    ///
    /// # Arguments
    ///
    /// * `addr` - The address of the actor to create
    /// * `actor` - The actor state to store
    ///
    /// # Returns
    ///
    /// The ActorID assigned to this actor
    fn create_actor(
        &mut self,
        addr: &Address,
        actor: fvm_shared::state::ActorState,
    ) -> Result<ActorID>;

    /// Put CBOR-serializable data into the blockstore and get its CID
    ///
    /// # Arguments
    ///
    /// * `data` - Raw CBOR bytes to store
    ///
    /// # Returns
    ///
    /// The CID of the stored data
    fn put_cbor_raw(&self, data: &[u8]) -> Result<Cid>;

    /// Get the initial circulating supply
    fn circ_supply(&self) -> &TokenAmount;

    /// Update the circulating supply
    fn add_to_circ_supply(&mut self, amount: &TokenAmount) -> Result<()>;

    /// Subtract from the circulating supply
    fn subtract_from_circ_supply(&mut self, amount: &TokenAmount) -> Result<()>;

    /// Create a custom actor with a specific ID and optional delegated address.
    ///
    /// This is used by plugins to create actors with predetermined IDs,
    /// typically for system actors that need well-known addresses.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the actor (for looking up code CID in manifest)
    /// * `id` - The actor ID to assign
    /// * `state` - The actor's initial state (will be CBOR-serialized)
    /// * `balance` - Initial token balance
    /// * `delegated_address` - Optional f4 address for Ethereum compatibility
    ///
    /// # Returns
    ///
    /// Ok(()) if successful, or an error if the actor couldn't be created
    fn create_custom_actor(
        &mut self,
        name: &str,
        id: ActorID,
        state: &impl serde::Serialize,
        balance: TokenAmount,
        delegated_address: Option<Address>,
    ) -> Result<()>;
}

/// Module trait for initializing actors during genesis.
///
/// Modules can implement this trait to create their own actors and
/// initialize state during the genesis process.
///
/// # Example
///
/// ```ignore
/// struct MyModule;
///
/// impl GenesisModule for MyModule {
///     fn initialize_actors<BS: Blockstore>(
///         &self,
///         state: &mut dyn GenesisState,
///         genesis: &Genesis,
///     ) -> Result<()> {
///         // Create your module's actors
///         let my_actor_state = fvm_shared::state::ActorState {
///             code: MY_ACTOR_CODE_CID,
///             state: state.put_cbor(&MyActorState::default())?,
///             sequence: 0,
///             balance: TokenAmount::zero(),
///             delegated_address: None,
///         };
///
///         state.create_actor(
///             &MY_ACTOR_ADDRESS,
///             my_actor_state,
///         )?;
///
///         Ok(())
///     }
///
///     fn name(&self) -> &str {
///         "my-module"
///     }
/// }
/// ```
pub trait GenesisModule: Send + Sync {
    /// Initialize module-specific actors during genesis.
    ///
    /// This is called after core actors are initialized but before
    /// the genesis state is finalized.
    ///
    /// # Arguments
    ///
    /// * `state` - The genesis state to modify (must be passed as concrete type)
    /// * `genesis` - The genesis configuration
    ///
    /// # Returns
    ///
    /// * `Ok(())` if initialization succeeded
    /// * `Err(e)` if initialization failed
    ///
    /// # Note
    ///
    /// The state parameter should be a concrete type implementing GenesisState,
    /// not a trait object, due to the generic methods in GenesisState.
    fn initialize_actors<S: GenesisState>(
        &self,
        state: &mut S,
        genesis: &Genesis,
    ) -> Result<()>;

    /// Get the module name for logging.
    fn name(&self) -> &str;

    /// Optional: Validate genesis configuration before initialization.
    ///
    /// This is called before any actors are created. Modules can use
    /// this to validate their genesis parameters.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the configuration is valid
    /// * `Err(e)` if the configuration is invalid
    fn validate_genesis(&self, _genesis: &Genesis) -> Result<()> {
        Ok(()) // Default: no validation
    }
}

/// Default no-op genesis module that doesn't initialize any actors.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoOpGenesisModule;

impl GenesisModule for NoOpGenesisModule {
    fn initialize_actors<S: GenesisState>(
        &self,
        _state: &mut S,
        _genesis: &Genesis,
    ) -> Result<()> {
        // No actors to initialize
        Ok(())
    }

    fn name(&self) -> &str {
        "noop"
    }

    fn validate_genesis(&self, _genesis: &Genesis) -> Result<()> {
        // No validation needed
        Ok(())
    }
}

impl std::fmt::Display for NoOpGenesisModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NoOpGenesisModule")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_op_genesis_module_default() {
        let _module = NoOpGenesisModule::default();
    }

    #[test]
    fn test_no_op_genesis_module_name() {
        let module = NoOpGenesisModule;
        assert_eq!(module.name(), "noop");
    }

    #[test]
    fn test_no_op_genesis_module_clone() {
        let module1 = NoOpGenesisModule;
        let _module2 = module1;
        let _module3 = module1; // NoOpGenesisModule is Copy
    }

    #[test]
    fn test_no_op_genesis_module_display() {
        let module = NoOpGenesisModule;
        let display = format!("{}", module);
        assert_eq!(display, "NoOpGenesisModule");
    }
}
