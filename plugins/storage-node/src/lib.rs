// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Storage Node Module Implementation
//!
//! This module integrates the storage-node functionality into Fendermint
//! through the module system. It uses `RecallExecutor` for FVM execution
//! with storage-node specific features.

pub mod helpers;

use anyhow::Result;
use async_trait::async_trait;
use fendermint_module::{
    cli::{CliModule, CommandArgs, CommandDef},
    externs::NoOpExterns,
    genesis::{GenesisModule, GenesisState},
    message::{ApplyMessageResponse, MessageApplyRet, MessageHandlerModule, MessageHandlerState},
    service::{ModuleResources, ServiceContext, ServiceModule},
    ExecutorModule, ModuleBundle,
};
use fendermint_vm_genesis::Genesis;
use fvm::call_manager::{CallManager, DefaultCallManager};
use fvm::engine::EnginePool;
use fvm::kernel::Kernel;
use fvm::machine::DefaultMachine;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::RawBytes;
use fvm_shared::address::Address;
use fvm_shared::error::ExitCode;
use std::collections::HashMap;
use std::fmt;
use storage_node_executor::RecallExecutor;

/// Plugin constructor for auto-discovery.
///
/// This function is called by the plugin system to create an instance.
/// Returns the concrete type directly (not trait object due to associated types).
pub fn create_plugin() -> StorageNodeModule {
    StorageNodeModule::default()
}

/// Storage node module bundle.
///
/// This module integrates storage-node functionality into Fendermint by:
/// - Using `RecallExecutor` for FVM execution with storage features
/// - Providing hooks for storage-node specific operations
/// - Enabling storage-node actors and functionality
#[derive(Debug, Clone, Default)]
pub struct StorageNodeModule;

impl ModuleBundle for StorageNodeModule {
    type Kernel = fvm::DefaultKernel<
        DefaultCallManager<DefaultMachine<fvm_ipld_blockstore::MemoryBlockstore, NoOpExterns>>,
    >;

    fn name(&self) -> &'static str {
        "storage-node"
    }

    fn version(&self) -> &'static str {
        "0.1.0"
    }

    fn description(&self) -> &'static str {
        "Storage node module with RecallExecutor integration"
    }
}

impl<K> ExecutorModule<K> for StorageNodeModule
where
    K: Kernel,
    <<K as Kernel>::CallManager as CallManager>::Machine: Send,
{
    type Executor = RecallExecutor<K>;

    fn create_executor(
        engine: EnginePool,
        machine: <<K as Kernel>::CallManager as CallManager>::Machine,
    ) -> Result<Self::Executor> {
        RecallExecutor::new(engine, machine)
    }
}

// MessageHandlerModule - Handle storage-specific IPC messages
#[async_trait]
impl MessageHandlerModule for StorageNodeModule {
    async fn handle_message<DB: Blockstore + Send + Sync>(
        &self,
        _state: &mut dyn MessageHandlerState,
        msg: &fendermint_vm_message::ipc::IpcMessage,
    ) -> Result<Option<ApplyMessageResponse>> {
        use fendermint_vm_message::ipc::IpcMessage;

        match msg {
            IpcMessage::ReadRequestPending(read_request) => {
                tracing::debug!(
                    request_id = %read_request.id,
                    "Storage plugin handling ReadRequestPending"
                );

                // TODO: Implement actual storage logic here
                // For now, return a placeholder response
                Ok(Some(ApplyMessageResponse {
                    apply_ret: MessageApplyRet {
                        from: Address::new_id(0),
                        to: Address::new_id(1),
                        method_num: 0,
                        gas_limit: 10_000_000,
                        exit_code: ExitCode::OK,
                        gas_used: 100,
                        return_data: RawBytes::default(),
                        emitters: HashMap::new(),
                    },
                    domain_hash: None,
                }))
            }
            IpcMessage::ReadRequestClosed(read_request) => {
                tracing::debug!(
                    request_id = %read_request.id,
                    "Storage plugin handling ReadRequestClosed"
                );

                // TODO: Implement actual storage logic here
                Ok(Some(ApplyMessageResponse {
                    apply_ret: MessageApplyRet {
                        from: Address::new_id(0),
                        to: Address::new_id(1),
                        method_num: 0,
                        gas_limit: 10_000_000,
                        exit_code: ExitCode::OK,
                        gas_used: 100,
                        return_data: RawBytes::default(),
                        emitters: HashMap::new(),
                    },
                    domain_hash: None,
                }))
            }
            _ => {
                // Not a storage-node message
                Ok(None)
            }
        }
    }

    fn message_types(&self) -> &[&str] {
        &["ReadRequestPending", "ReadRequestClosed"]
    }

    async fn validate_message(
        &self,
        msg: &fendermint_vm_message::ipc::IpcMessage,
    ) -> Result<bool> {
        use fendermint_vm_message::ipc::IpcMessage;

        match msg {
            IpcMessage::ReadRequestPending(_) | IpcMessage::ReadRequestClosed(_) => {
                // TODO: Add validation logic
                Ok(true)
            }
            _ => Ok(true), // Don't validate messages we don't handle
        }
    }
}

// GenesisModule - delegate to no-op for now
impl GenesisModule for StorageNodeModule {
    fn initialize_actors<S: GenesisState>(
        &self,
        _state: &mut S,
        _genesis: &Genesis,
    ) -> Result<()> {
        // For now, no custom genesis initialization
        // Future: Initialize storage-node actors and state
        Ok(())
    }

    fn name(&self) -> &str {
        "storage-node"
    }

    fn validate_genesis(&self, _genesis: &Genesis) -> Result<()> {
        // Future: Validate storage-node configuration
        Ok(())
    }
}

// ServiceModule - delegate to no-op for now
#[async_trait]
impl ServiceModule for StorageNodeModule {
    async fn initialize_services(
        &self,
        _ctx: &ServiceContext,
    ) -> Result<Vec<tokio::task::JoinHandle<()>>> {
        // Future: Initialize storage-node background services
        // (IPLD resolver, Iroh manager, etc.)
        Ok(vec![])
    }

    fn resources(&self) -> ModuleResources {
        // Future: Provide shared resources
        ModuleResources::empty()
    }

    async fn health_check(&self) -> Result<bool> {
        // Future: Check health of storage-node services
        Ok(true)
    }

    async fn shutdown(&self) -> Result<()> {
        // Future: Clean shutdown of storage-node services
        Ok(())
    }
}

// CliModule - delegate to no-op for now
#[async_trait]
impl CliModule for StorageNodeModule {
    fn commands(&self) -> Vec<CommandDef> {
        // Future: Add storage-node CLI commands
        // e.g., storage-node status, storage-node list-blobs, etc.
        vec![]
    }

    async fn execute(&self, _args: &CommandArgs) -> Result<()> {
        // Future: Execute storage-node commands
        Ok(())
    }

    fn complete(&self, _command: &str, _arg: &str) -> Vec<String> {
        vec![]
    }
}

impl fmt::Display for StorageNodeModule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "StorageNodeModule")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_name() {
        let module = StorageNodeModule;
        assert_eq!(ModuleBundle::name(&module), "storage-node");
    }

    #[test]
    fn test_module_version() {
        let module = StorageNodeModule;
        assert_eq!(ModuleBundle::version(&module), "0.1.0");
    }

    #[test]
    fn test_module_display() {
        let module = StorageNodeModule;
        assert_eq!(format!("{}", module), "StorageNodeModule");
    }

    #[tokio::test]
    async fn test_message_handler_no_custom_messages() {
        use fendermint_vm_core::Timestamp;
        use fendermint_vm_message::ipc::{IpcMessage, ParentFinality};

        let module = StorageNodeModule;
        let msg = IpcMessage::TopDownExec(ParentFinality {
            height: 0,
            block_hash: vec![],
        });

        // Create a simple test state
        struct TestState {
            height: ChainEpoch,
            timestamp: Timestamp,
            base_fee: TokenAmount,
            chain_id: u64,
        }

        impl MessageHandlerState for TestState {
            fn block_height(&self) -> ChainEpoch {
                self.height
            }
            fn timestamp(&self) -> fendermint_vm_core::Timestamp {
                self.timestamp
            }
            fn base_fee(&self) -> &TokenAmount {
                &self.base_fee
            }
            fn chain_id(&self) -> u64 {
                self.chain_id
            }
        }

        let mut state = TestState {
            height: 0,
            timestamp: Timestamp(0),
            base_fee: TokenAmount::zero(),
            chain_id: 1,
        };

        let result = module.handle_message(&mut state, &msg).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none()); // No custom handling
    }

    #[tokio::test]
    async fn test_service_module_defaults() {
        let module = StorageNodeModule;

        assert!(module.health_check().await.is_ok());
        assert!(module.shutdown().await.is_ok());
    }
}
