// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Executor module trait for customizing FVM execution.
//!
//! This trait allows modules to provide custom executor implementations,
//! enabling features like multi-party gas accounting, transaction sponsors,
//! or other execution-level modifications.

use anyhow::Result;
use fvm::call_manager::CallManager;
use fvm::engine::EnginePool;
use fvm::executor::Executor;
use fvm::kernel::Kernel;

/// Module trait for providing custom executor implementations.
///
/// Modules can implement this trait to provide their own executor type,
/// allowing them to customize message execution behavior. This is useful
/// for features that require deep integration with the execution flow,
/// such as multi-party gas accounting or custom transaction handling.
///
/// # Type Parameters
///
/// * `K` - The kernel type used by the executor
///
/// # Example
///
/// ```ignore
/// struct MyModule;
///
/// impl<K: Kernel> ExecutorModule<K> for MyModule {
///     type Executor = MyCustomExecutor<K>;
///
///     fn create_executor(
///         engine_pool: EnginePool,
///         machine: <K::CallManager as CallManager>::Machine,
///     ) -> Result<Self::Executor> {
///         MyCustomExecutor::new(engine_pool, machine)
///     }
/// }
/// ```
pub trait ExecutorModule<K: Kernel>
where
    <K::CallManager as CallManager>::Machine: Send,
{
    /// The executor type provided by this module.
    ///
    /// **Important**: The executor must implement `Deref` and `DerefMut` to the underlying Machine
    /// to allow FvmExecState to access machine methods like `state_tree()`, `context()`, etc.
    ///
    /// The Machine must also be Send to support async operations (ensured by trait bound).
    ///
    /// Note: FVM's DefaultExecutor does not implement these traits. Use RecallExecutor
    /// from storage-node or implement a custom executor wrapper.
    type Executor: Executor<Kernel = K>
        + std::ops::Deref<Target = <K::CallManager as CallManager>::Machine>
        + std::ops::DerefMut;

    /// Create an executor instance.
    ///
    /// # Arguments
    ///
    /// * `engine_pool` - Pool of FVM engines for message execution
    /// * `machine` - The FVM machine instance
    ///
    /// # Returns
    ///
    /// A new executor instance configured for this module.
    fn create_executor(
        engine_pool: EnginePool,
        machine: <K::CallManager as CallManager>::Machine,
    ) -> Result<Self::Executor>;
}

/// Default no-op executor module.
///
/// This uses RecallExecutor from storage-node, which properly implements
/// `Deref<Target = Machine>` as required by the `ExecutorModule` trait.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoOpExecutorModule;

impl<K> ExecutorModule<K> for NoOpExecutorModule
where
    K: Kernel,
    <K::CallManager as CallManager>::Machine: Send,
{
    type Executor = storage_node_executor::RecallExecutor<K>;

    fn create_executor(
        engine_pool: EnginePool,
        machine: <K::CallManager as CallManager>::Machine,
    ) -> Result<Self::Executor> {
        Ok(storage_node_executor::RecallExecutor::new(engine_pool, machine)?)
    }
}

/// A wrapper executor that provides `Deref` access to the machine.
///
/// This wraps FVM's DefaultExecutor and provides access to the underlying machine
/// through Deref/DerefMut, which is required by the ExecutorModule trait.
pub struct DelegatingExecutor<K: Kernel> {
    inner: fvm::executor::DefaultExecutor<K>,
}

impl<K: Kernel> DelegatingExecutor<K> {
    /// Create a new delegating executor
    pub fn new(inner: fvm::executor::DefaultExecutor<K>) -> Self {
        Self { inner }
    }

    /// Get the underlying executor
    pub fn inner(&self) -> &fvm::executor::DefaultExecutor<K> {
        &self.inner
    }

    /// Get the underlying executor mutably
    pub fn inner_mut(&mut self) -> &mut fvm::executor::DefaultExecutor<K> {
        &mut self.inner
    }
}

impl<K: Kernel> Executor for DelegatingExecutor<K> {
    type Kernel = K;

    fn execute_message(
        &mut self,
        msg: fvm_shared::message::Message,
        apply_kind: fvm::executor::ApplyKind,
        raw_length: usize,
    ) -> Result<fvm::executor::ApplyRet> {
        self.inner.execute_message(msg, apply_kind, raw_length)
    }

    fn flush(&mut self) -> Result<cid::Cid> {
        self.inner.flush()
    }
}

// Note: We cannot implement Deref for DelegatingExecutor<DefaultExecutor> because
// DefaultExecutor doesn't expose its machine. This means NoOpExecutorModule won't
// satisfy the ExecutorModule trait bounds. This is intentional - use RecallExecutor
// or another executor that properly exposes the machine.
//
// Commented out - cannot implement without machine access:
// impl<K: Kernel> std::ops::Deref for DelegatingExecutor<K> {
//     type Target = <K::CallManager as CallManager>::Machine;
//     fn deref(&self) -> &Self::Target {
//         // Cannot access - machine is private in DefaultExecutor
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_op_executor_module_default() {
        let _module = NoOpExecutorModule::default();
    }

    #[test]
    fn test_no_op_executor_module_clone() {
        let module1 = NoOpExecutorModule;
        let _module2 = module1;
        let _module3 = module1; // NoOpExecutorModule is Copy
    }
}
