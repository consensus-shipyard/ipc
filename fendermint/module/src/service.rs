// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Service module trait for initializing background services.
//!
//! This trait allows modules to start background tasks and provide
//! resources that other components can use.

use anyhow::Result;
use async_trait::async_trait;
use std::any::Any;
use std::fmt;
use std::sync::Arc;
use tokio::task::JoinHandle;

/// Context provided to service modules during initialization.
///
/// This contains all the resources a module needs to start its services,
/// including settings, keys, and access to the database.
pub struct ServiceContext {
    /// Module-specific settings (opaque to the framework)
    pub settings: Box<dyn Any + Send + Sync>,
    /// Optional validator keypair for signing operations
    pub validator_keypair: Option<Vec<u8>>,
    /// Additional context data (can be populated by other modules)
    pub extra: Arc<dyn Any + Send + Sync>,
}

impl ServiceContext {
    /// Create a new service context with minimal configuration
    pub fn new(settings: Box<dyn Any + Send + Sync>) -> Self {
        Self {
            settings,
            validator_keypair: None,
            extra: Arc::new(()),
        }
    }

    /// Set the validator keypair
    pub fn with_validator_keypair(mut self, keypair: Vec<u8>) -> Self {
        self.validator_keypair = Some(keypair);
        self
    }

    /// Set extra context data
    pub fn with_extra(mut self, extra: Arc<dyn Any + Send + Sync>) -> Self {
        self.extra = extra;
        self
    }

    /// Try to downcast the settings to a specific type
    pub fn settings_as<T: 'static>(&self) -> Option<&T> {
        self.settings.downcast_ref::<T>()
    }

    /// Try to downcast the extra context to a specific type
    pub fn extra_as<T: 'static>(&self) -> Option<&T> {
        (*self.extra).downcast_ref::<T>()
    }
}

impl fmt::Debug for ServiceContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ServiceContext")
            .field("has_validator_keypair", &self.validator_keypair.is_some())
            .finish()
    }
}

/// Resources provided by a module to other components.
///
/// Modules can use this to share resources like connection pools,
/// caches, or other shared state with the rest of the system.
pub struct ModuleResources {
    resources: Arc<dyn Any + Send + Sync>,
}

impl ModuleResources {
    /// Create a new module resources container
    pub fn new<T: Any + Send + Sync>(resources: T) -> Self {
        Self {
            resources: Arc::new(resources),
        }
    }

    /// Create an empty resources container
    pub fn empty() -> Self {
        Self {
            resources: Arc::new(()),
        }
    }

    /// Try to get resources as a specific type
    pub fn get<T: 'static>(&self) -> Option<&T> {
        (*self.resources).downcast_ref::<T>()
    }

    /// Get the underlying Arc
    pub fn as_arc(&self) -> Arc<dyn Any + Send + Sync> {
        self.resources.clone()
    }
}

impl fmt::Debug for ModuleResources {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ModuleResources").finish()
    }
}

impl Clone for ModuleResources {
    fn clone(&self) -> Self {
        Self {
            resources: self.resources.clone(),
        }
    }
}

/// Module trait for initializing background services.
///
/// Modules can implement this trait to start background tasks that
/// run for the lifetime of the application. These tasks might handle
/// things like:
/// - Network communication
/// - Background data processing
/// - Cache management
/// - Resource resolution
///
/// # Example
///
/// ```ignore
/// struct MyModule;
///
/// #[async_trait]
/// impl ServiceModule for MyModule {
///     async fn initialize_services(
///         &self,
///         ctx: &ServiceContext,
///     ) -> Result<Vec<JoinHandle<()>>> {
///         let mut handles = vec![];
///
///         // Start a background task
///         handles.push(tokio::spawn(async move {
///             loop {
///                 // Do background work
///                 tokio::time::sleep(Duration::from_secs(1)).await;
///             }
///         }));
///
///         Ok(handles)
///     }
///
///     fn resources(&self) -> ModuleResources {
///         ModuleResources::new(MyModuleResources {
///             // ... shared resources ...
///         })
///     }
/// }
/// ```
#[async_trait]
pub trait ServiceModule: Send + Sync {
    /// Initialize background services.
    ///
    /// This is called during application startup. The module should spawn
    /// any background tasks it needs and return their join handles.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Context containing settings and other initialization data
    ///
    /// # Returns
    ///
    /// A vector of join handles for the spawned tasks
    async fn initialize_services(
        &self,
        ctx: &ServiceContext,
    ) -> Result<Vec<JoinHandle<()>>>;

    /// Provide resources to other components.
    ///
    /// This is called after `initialize_services` completes. The resources
    /// can be used by other parts of the system to interact with this module.
    ///
    /// # Returns
    ///
    /// A container with module-specific resources
    fn resources(&self) -> ModuleResources;

    /// Optional: Perform cleanup when shutting down.
    ///
    /// This is called when the application is shutting down gracefully.
    /// Modules can use this to clean up resources or save state.
    async fn shutdown(&self) -> Result<()> {
        Ok(()) // Default: no cleanup needed
    }

    /// Optional: Health check for the module's services.
    ///
    /// This can be used to monitor the health of background services.
    ///
    /// # Returns
    ///
    /// * `Ok(true)` if all services are healthy
    /// * `Ok(false)` if services are degraded but operational
    /// * `Err(e)` if services have failed
    async fn health_check(&self) -> Result<bool> {
        Ok(true) // Default: always healthy
    }
}

/// Default no-op service module that doesn't start any services.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoOpServiceModule;

#[async_trait]
impl ServiceModule for NoOpServiceModule {
    async fn initialize_services(
        &self,
        _ctx: &ServiceContext,
    ) -> Result<Vec<JoinHandle<()>>> {
        Ok(vec![]) // No services to start
    }

    fn resources(&self) -> ModuleResources {
        ModuleResources::empty()
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(()) // Nothing to clean up
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(true) // Always healthy
    }
}

impl fmt::Display for NoOpServiceModule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NoOpServiceModule")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_no_op_service_module_initialize() {
        let module = NoOpServiceModule::default();
        let ctx = ServiceContext::new(Box::new(()));

        let handles = module.initialize_services(&ctx).await;
        assert!(handles.is_ok());
        assert_eq!(handles.unwrap().len(), 0);
    }

    #[test]
    fn test_no_op_service_module_resources() {
        let module = NoOpServiceModule;
        let resources = module.resources();
        // Empty resources contain unit type as placeholder
        assert!(resources.get::<()>().is_some());
    }

    #[tokio::test]
    async fn test_no_op_service_module_shutdown() {
        let module = NoOpServiceModule;
        let result = module.shutdown().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_no_op_service_module_health_check() {
        let module = NoOpServiceModule;
        let result = module.health_check().await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_service_context_creation() {
        let ctx = ServiceContext::new(Box::new("test"));
        assert!(ctx.validator_keypair.is_none());
    }

    #[test]
    fn test_service_context_with_keypair() {
        let ctx = ServiceContext::new(Box::new("test"))
            .with_validator_keypair(vec![1, 2, 3]);
        assert!(ctx.validator_keypair.is_some());
        assert_eq!(ctx.validator_keypair.unwrap(), vec![1, 2, 3]);
    }

    #[test]
    fn test_module_resources_get() {
        struct TestData {
            value: i32,
        }

        let resources = ModuleResources::new(TestData { value: 42 });
        let data = resources.get::<TestData>();
        assert!(data.is_some());
        assert_eq!(data.unwrap().value, 42);
    }

    #[test]
    fn test_module_resources_clone() {
        let resources1 = ModuleResources::new(42);
        let resources2 = resources1.clone();
        assert_eq!(resources1.get::<i32>(), resources2.get::<i32>());
    }
}
