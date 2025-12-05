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
use fvm::executor::{ApplyKind, ApplyRet, Executor};
use fvm::kernel::Kernel;
use fvm_shared::message::Message;

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
pub trait ExecutorModule<K: Kernel> {
    /// The executor type provided by this module.
    type Executor: Executor<Kernel = K>;

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

/// Default no-op executor module that uses FVM's standard executor.
///
/// This is used when no module-specific executor is needed.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoOpExecutorModule;

impl<K> ExecutorModule<K> for NoOpExecutorModule
where
    K: Kernel,
{
    type Executor = fvm::executor::DefaultExecutor<K>;

    fn create_executor(
        engine_pool: EnginePool,
        machine: <K::CallManager as CallManager>::Machine,
    ) -> Result<Self::Executor> {
        Ok(fvm::executor::DefaultExecutor::new(
            engine_pool,
            machine,
        )?)
    }
}

/// A wrapper executor that delegates to an inner executor.
///
/// This is useful for testing and for modules that want to wrap
/// the default executor with additional functionality.
pub struct DelegatingExecutor<E: Executor> {
    inner: E,
}

impl<E: Executor> DelegatingExecutor<E> {
    /// Create a new delegating executor wrapping the given executor.
    pub fn new(inner: E) -> Self {
        Self { inner }
    }

    /// Get a reference to the inner executor.
    pub fn inner(&self) -> &E {
        &self.inner
    }

    /// Get a mutable reference to the inner executor.
    pub fn inner_mut(&mut self) -> &mut E {
        &mut self.inner
    }

    /// Consume this wrapper and return the inner executor.
    pub fn into_inner(self) -> E {
        self.inner
    }
}

impl<E: Executor> Executor for DelegatingExecutor<E> {
    type Kernel = E::Kernel;

    fn execute_message(
        &mut self,
        msg: Message,
        apply_kind: ApplyKind,
        raw_length: usize,
    ) -> Result<ApplyRet> {
        self.inner.execute_message(msg, apply_kind, raw_length)
    }

    fn flush(&mut self) -> Result<cid::Cid> {
        self.inner.flush()
    }
}

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
