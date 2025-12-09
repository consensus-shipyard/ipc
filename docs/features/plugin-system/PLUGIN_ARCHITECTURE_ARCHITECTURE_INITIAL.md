# IPC Modular Architecture Specification

## Overview

This document specifies the refactoring of IPC into a modular architecture, separating the core library from the node and CLI implementations, and introducing a plugin system for extensible modules (starting with storage).

### Goals

1. **Separation of concerns**: Core consensus/state logic independent from node runtime
2. **Modularity**: Pluggable backends for storage, telemetry, and future subsystems
3. **Developer experience**: Clear interfaces, good documentation, easy module development
4. **Operator experience**: Simple configuration, helpful CLI, validation tooling
5. **Incremental adoption**: Implement in stages without breaking existing functionality

### Architecture Overview

```
ipc/
├── crates/
│   ├── ipc-core/              # Core library (consensus, state, types)
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── consensus/
│   │   │   ├── state/
│   │   │   ├── types/
│   │   │   └── modules/       # Module trait definitions
│   │   │       ├── mod.rs
│   │   │       ├── registry.rs
│   │   │       ├── storage.rs
│   │   │       └── testing.rs
│   │   └── Cargo.toml
│   │
│   ├── ipc-node/              # Node implementation
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── config.rs
│   │   │   └── runtime.rs
│   │   └── Cargo.toml
│   │
│   ├── ipc-cli/               # CLI tooling
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   └── commands/
│   │   └── Cargo.toml
│   │
│   └── ipc-modules/           # First-party module implementations
│       ├── storage-basin/
│       ├── storage-actor/
│       └── storage-local/
│
└── Cargo.toml                 # Workspace root
```

---

## Stage 1: Core Library Extraction

### Objective

Extract the core IPC logic into `ipc-core` crate that can be imported independently.

### Tasks

#### 1.1 Create workspace structure

```toml
# Root Cargo.toml
[workspace]
resolver = "2"
members = [
    "crates/ipc-core",
    "crates/ipc-node",
    "crates/ipc-cli",
    "crates/ipc-modules/*",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"
repository = "https://github.com/consensus-shipyard/ipc"

[workspace.dependencies]
# Shared dependencies with versions pinned at workspace level
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
thiserror = "1.0"
async-trait = "0.1"
tracing = "0.1"
```

#### 1.2 Define ipc-core public API

The core library should expose:

```rust
// ipc-core/src/lib.rs

// Re-export core types
pub mod types;
pub mod state;
pub mod consensus;
pub mod modules;

// Prelude for common imports
pub mod prelude {
    pub use crate::types::*;
    pub use crate::modules::{ModuleRegistry, ModuleRegistryBuilder};
    pub use crate::modules::storage::StorageBackend;
}
```

#### 1.3 Identify and move core components

Review existing codebase and categorize:

| Component | Destination | Notes |
|-----------|-------------|-------|
| Subnet types/structs | `ipc-core/types` | Foundation types |
| State management | `ipc-core/state` | State machine logic |
| Consensus interfaces | `ipc-core/consensus` | CometBFT/F3 abstractions |
| Cryptographic primitives | `ipc-core/crypto` | Signing, verification |
| Actor definitions | `ipc-core/actors` | Core actor interfaces |
| Node runtime | `ipc-node` | Stays in node |
| CLI commands | `ipc-cli` | Stays in CLI |
| RPC server | `ipc-node` | Node-specific |

#### 1.4 Establish dependency direction

```
ipc-cli ──────┐
              ├──► ipc-core
ipc-node ─────┘
              │
ipc-modules/* ─┘
```

**Rule**: `ipc-core` MUST NOT depend on `ipc-node`, `ipc-cli`, or any specific module implementation.

### Acceptance Criteria - Stage 1

- [ ] Workspace compiles with new structure
- [ ] `ipc-core` can be imported independently
- [ ] `ipc-node` builds and runs using `ipc-core` as dependency
- [ ] `ipc-cli` builds and runs using `ipc-core` as dependency
- [ ] All existing tests pass
- [ ] No circular dependencies

---

## Stage 2: Module System Foundation

### Objective

Implement the module trait system and registry in `ipc-core`.

### Tasks

#### 2.1 Define module traits

```rust
// ipc-core/src/modules/mod.rs

pub mod storage;
pub mod registry;
pub mod config;
pub mod testing;

pub use registry::{ModuleRegistry, ModuleRegistryBuilder};
pub use config::{ConfigSchema, ConfigField, ConfigValue};
```

```rust
// ipc-core/src/modules/config.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Schema definition for module configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSchema {
    pub fields: Vec<ConfigField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigField {
    pub name: String,
    pub description: String,
    pub field_type: ConfigFieldType,
    pub required: bool,
    pub default: Option<ConfigValue>,
    pub env_var: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigFieldType {
    String,
    Integer,
    Float,
    Boolean,
    Duration,
    Url,
    Path,
    Array(Box<ConfigFieldType>),
    Object(ConfigSchema),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<ConfigValue>),
    Object(HashMap<String, ConfigValue>),
    Null,
}

impl ConfigSchema {
    pub fn builder() -> ConfigSchemaBuilder {
        ConfigSchemaBuilder::default()
    }

    /// Validate a TOML value against this schema
    pub fn validate(&self, value: &toml::Value) -> Result<(), ConfigValidationError> {
        // Implementation validates all required fields present,
        // types match, etc.
        todo!()
    }

    /// Generate example TOML configuration
    pub fn example_toml(&self) -> String {
        todo!()
    }
}

#[derive(Default)]
pub struct ConfigSchemaBuilder {
    fields: Vec<ConfigField>,
}

impl ConfigSchemaBuilder {
    pub fn field(
        mut self,
        name: impl Into<String>,
        field_type: ConfigFieldType,
        required: bool,
    ) -> Self {
        self.fields.push(ConfigField {
            name: name.into(),
            description: String::new(),
            field_type,
            required,
            default: None,
            env_var: None,
        });
        self
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        if let Some(field) = self.fields.last_mut() {
            field.description = desc.into();
        }
        self
    }

    pub fn default_value(mut self, value: ConfigValue) -> Self {
        if let Some(field) = self.fields.last_mut() {
            field.default = Some(value);
        }
        self
    }

    pub fn env_var(mut self, var: impl Into<String>) -> Self {
        if let Some(field) = self.fields.last_mut() {
            field.env_var = Some(var.into());
        }
        self
    }

    pub fn build(self) -> ConfigSchema {
        ConfigSchema { fields: self.fields }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigValidationError {
    #[error("missing required field: {0}")]
    MissingRequired(String),
    #[error("invalid type for field {field}: expected {expected}, got {actual}")]
    TypeMismatch {
        field: String,
        expected: String,
        actual: String,
    },
    #[error("validation error for field {field}: {message}")]
    ValidationFailed { field: String, message: String },
}
```

#### 2.2 Define storage module trait

```rust
// ipc-core/src/modules/storage.rs

use async_trait::async_trait;
use crate::modules::config::ConfigSchema;
use std::fmt::Debug;

/// Metadata about a storage module
#[derive(Debug, Clone)]
pub struct StorageModuleInfo {
    /// Unique identifier for this storage backend
    pub name: &'static str,
    /// Human-readable description
    pub description: &'static str,
    /// Version of this module
    pub version: &'static str,
}

/// Result type for storage operations
pub type StorageResult<T> = Result<T, StorageError>;

/// Errors that can occur during storage operations
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("key not found: {0}")]
    NotFound(String),

    #[error("connection error: {0}")]
    Connection(String),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("configuration error: {0}")]
    Configuration(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("storage backend error: {0}")]
    Backend(#[from] Box<dyn std::error::Error + Send + Sync>),
}

/// Options for store operations
#[derive(Debug, Clone, Default)]
pub struct StoreOptions {
    /// Time-to-live for the stored value
    pub ttl: Option<std::time::Duration>,
    /// Whether to overwrite existing values
    pub overwrite: bool,
    /// Optional metadata to store with the value
    pub metadata: Option<Vec<(String, String)>>,
}

/// Options for retrieve operations
#[derive(Debug, Clone, Default)]
pub struct RetrieveOptions {
    /// Whether to include metadata in response
    pub include_metadata: bool,
}

/// Response from a retrieve operation
#[derive(Debug, Clone)]
pub struct RetrieveResponse {
    pub value: Vec<u8>,
    pub metadata: Option<Vec<(String, String)>>,
}

/// Health check result for a storage backend
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    pub healthy: bool,
    pub message: Option<String>,
    pub latency: Option<std::time::Duration>,
}

/// Core trait that all storage backends must implement
#[async_trait]
pub trait StorageBackend: Send + Sync + Debug {
    /// Store a value at the given key
    async fn store(
        &self,
        key: &[u8],
        value: &[u8],
        options: StoreOptions,
    ) -> StorageResult<()>;

    /// Retrieve a value by key
    async fn retrieve(
        &self,
        key: &[u8],
        options: RetrieveOptions,
    ) -> StorageResult<Option<RetrieveResponse>>;

    /// Delete a value by key
    async fn delete(&self, key: &[u8]) -> StorageResult<bool>;

    /// Check if a key exists
    async fn exists(&self, key: &[u8]) -> StorageResult<bool>;

    /// List keys with optional prefix
    async fn list_keys(&self, prefix: Option<&[u8]>) -> StorageResult<Vec<Vec<u8>>>;

    /// Perform a health check
    async fn health_check(&self) -> HealthCheckResult;

    /// Graceful shutdown
    async fn shutdown(&self) -> StorageResult<()>;
}

/// Factory trait for creating storage backends from configuration
pub trait StorageModule: Send + Sync {
    /// The backend type this module creates
    type Backend: StorageBackend;

    /// Module information
    fn info() -> StorageModuleInfo;

    /// Configuration schema for this module
    fn config_schema() -> ConfigSchema;

    /// Create a new backend instance from configuration
    fn from_config(config: &toml::Value) -> Result<Self::Backend, StorageError>;
}

/// Type-erased storage backend for runtime flexibility
pub type DynStorageBackend = Box<dyn StorageBackend>;

/// Factory function type for creating storage backends
pub type StorageFactory = fn(&toml::Value) -> Result<DynStorageBackend, StorageError>;
```

#### 2.3 Implement module registry

```rust
// ipc-core/src/modules/registry.rs

use crate::modules::storage::{DynStorageBackend, StorageFactory, StorageModuleInfo, ConfigSchema};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

/// Registry entry for a storage module
#[derive(Clone)]
pub struct StorageModuleEntry {
    pub info: StorageModuleInfo,
    pub config_schema: ConfigSchema,
    pub factory: StorageFactory,
}

/// Global registry for available modules
/// This allows compile-time registration of modules via inventory or ctor
static STORAGE_MODULES: RwLock<HashMap<&'static str, StorageModuleEntry>> =
    RwLock::new(HashMap::new());

/// Register a storage module at runtime
pub fn register_storage_module(entry: StorageModuleEntry) {
    let mut modules = STORAGE_MODULES.write();
    modules.insert(entry.info.name, entry);
}

/// Get all registered storage modules
pub fn available_storage_modules() -> Vec<StorageModuleEntry> {
    STORAGE_MODULES.read().values().cloned().collect()
}

/// Get a specific storage module by name
pub fn get_storage_module(name: &str) -> Option<StorageModuleEntry> {
    STORAGE_MODULES.read().get(name).cloned()
}

/// Active module instances for a running node
pub struct ModuleRegistry {
    storage: Option<Arc<DynStorageBackend>>,
    // Future: Add other module types
    // telemetry: Option<Arc<DynTelemetryBackend>>,
    // networking: Option<Arc<DynNetworkingBackend>>,
}

impl ModuleRegistry {
    /// Create a new builder for constructing a registry
    pub fn builder() -> ModuleRegistryBuilder {
        ModuleRegistryBuilder::default()
    }

    /// Get the storage backend, if configured
    pub fn storage(&self) -> Option<Arc<DynStorageBackend>> {
        self.storage.clone()
    }

    /// Check if storage is available
    pub fn has_storage(&self) -> bool {
        self.storage.is_some()
    }

    /// Shutdown all modules gracefully
    pub async fn shutdown(&self) -> Result<(), ModuleShutdownError> {
        if let Some(storage) = &self.storage {
            storage.shutdown().await.map_err(|e| {
                ModuleShutdownError::Storage(e.to_string())
            })?;
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ModuleShutdownError {
    #[error("storage shutdown error: {0}")]
    Storage(String),
}

#[derive(Default)]
pub struct ModuleRegistryBuilder {
    storage: Option<DynStorageBackend>,
}

impl ModuleRegistryBuilder {
    /// Configure storage backend directly
    pub fn with_storage(mut self, backend: impl Into<DynStorageBackend>) -> Self {
        self.storage = Some(backend.into());
        self
    }

    /// Configure storage backend from module name and config
    pub fn with_storage_module(
        mut self,
        module_name: &str,
        config: &toml::Value,
    ) -> Result<Self, ModuleBuildError> {
        let module = get_storage_module(module_name)
            .ok_or_else(|| ModuleBuildError::ModuleNotFound(module_name.to_string()))?;

        // Validate configuration
        module.config_schema.validate(config)
            .map_err(|e| ModuleBuildError::ConfigValidation(e.to_string()))?;

        // Create backend
        let backend = (module.factory)(config)
            .map_err(|e| ModuleBuildError::Initialization(e.to_string()))?;

        self.storage = Some(backend);
        Ok(self)
    }

    /// Build the registry
    pub fn build(self) -> ModuleRegistry {
        ModuleRegistry {
            storage: self.storage.map(Arc::new),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ModuleBuildError {
    #[error("module not found: {0}")]
    ModuleNotFound(String),
    #[error("configuration validation failed: {0}")]
    ConfigValidation(String),
    #[error("module initialization failed: {0}")]
    Initialization(String),
}

/// Macro for registering storage modules at compile time
#[macro_export]
macro_rules! register_storage_module {
    ($module:ty) => {
        // Uses inventory crate or ctor for static registration
        $crate::modules::registry::register_storage_module(
            $crate::modules::registry::StorageModuleEntry {
                info: <$module as $crate::modules::storage::StorageModule>::info(),
                config_schema: <$module as $crate::modules::storage::StorageModule>::config_schema(),
                factory: |config| {
                    let backend = <$module as $crate::modules::storage::StorageModule>::from_config(config)?;
                    Ok(Box::new(backend))
                },
            }
        );
    };
}
```

### Acceptance Criteria - Stage 2

- [ ] Module traits compile and are well-documented
- [ ] ConfigSchema can validate TOML configurations
- [ ] ModuleRegistry can be built with storage backend
- [ ] Registration macro works for storage modules
- [ ] Unit tests for config validation

---

## Stage 3: Storage Module Implementations

### Objective

Implement the first storage backends: local (for development), Basin, and custom-actor.

### Tasks

#### 3.1 Local storage module (development/testing)

```rust
// ipc-modules/storage-local/src/lib.rs

use ipc_core::modules::storage::*;
use ipc_core::modules::config::*;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use parking_lot::RwLock;
use tokio::fs;

/// Local filesystem storage backend for development and testing
#[derive(Debug)]
pub struct LocalStorage {
    base_path: PathBuf,
    // In-memory cache for faster access
    cache: RwLock<HashMap<Vec<u8>, Vec<u8>>>,
    use_cache: bool,
}

impl LocalStorage {
    pub fn new(base_path: PathBuf, use_cache: bool) -> Self {
        Self {
            base_path,
            cache: RwLock::new(HashMap::new()),
            use_cache,
        }
    }

    fn key_to_path(&self, key: &[u8]) -> PathBuf {
        let hex_key = hex::encode(key);
        // Create subdirectories based on first 4 chars to avoid too many files in one dir
        let (prefix, rest) = hex_key.split_at(4.min(hex_key.len()));
        self.base_path.join(prefix).join(rest)
    }
}

#[async_trait]
impl StorageBackend for LocalStorage {
    async fn store(
        &self,
        key: &[u8],
        value: &[u8],
        options: StoreOptions,
    ) -> StorageResult<()> {
        let path = self.key_to_path(key);

        // Create parent directories
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await
                .map_err(|e| StorageError::Backend(Box::new(e)))?;
        }

        // Check overwrite setting
        if !options.overwrite && path.exists() {
            return Err(StorageError::Backend(
                "key already exists and overwrite=false".into()
            ));
        }

        // Write to file
        fs::write(&path, value).await
            .map_err(|e| StorageError::Backend(Box::new(e)))?;

        // Update cache
        if self.use_cache {
            self.cache.write().insert(key.to_vec(), value.to_vec());
        }

        Ok(())
    }

    async fn retrieve(
        &self,
        key: &[u8],
        _options: RetrieveOptions,
    ) -> StorageResult<Option<RetrieveResponse>> {
        // Check cache first
        if self.use_cache {
            if let Some(value) = self.cache.read().get(key) {
                return Ok(Some(RetrieveResponse {
                    value: value.clone(),
                    metadata: None,
                }));
            }
        }

        let path = self.key_to_path(key);

        match fs::read(&path).await {
            Ok(value) => {
                if self.use_cache {
                    self.cache.write().insert(key.to_vec(), value.clone());
                }
                Ok(Some(RetrieveResponse {
                    value,
                    metadata: None,
                }))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(StorageError::Backend(Box::new(e))),
        }
    }

    async fn delete(&self, key: &[u8]) -> StorageResult<bool> {
        let path = self.key_to_path(key);

        if self.use_cache {
            self.cache.write().remove(key);
        }

        match fs::remove_file(&path).await {
            Ok(()) => Ok(true),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(StorageError::Backend(Box::new(e))),
        }
    }

    async fn exists(&self, key: &[u8]) -> StorageResult<bool> {
        if self.use_cache && self.cache.read().contains_key(key) {
            return Ok(true);
        }
        Ok(self.key_to_path(key).exists())
    }

    async fn list_keys(&self, prefix: Option<&[u8]>) -> StorageResult<Vec<Vec<u8>>> {
        // Implementation walks directory structure
        todo!("implement directory walking with prefix filter")
    }

    async fn health_check(&self) -> HealthCheckResult {
        // Check if base path is writable
        let test_path = self.base_path.join(".health_check");
        let start = std::time::Instant::now();

        match fs::write(&test_path, b"ok").await {
            Ok(()) => {
                let _ = fs::remove_file(&test_path).await;
                HealthCheckResult {
                    healthy: true,
                    message: None,
                    latency: Some(start.elapsed()),
                }
            }
            Err(e) => HealthCheckResult {
                healthy: false,
                message: Some(e.to_string()),
                latency: Some(start.elapsed()),
            },
        }
    }

    async fn shutdown(&self) -> StorageResult<()> {
        // Flush cache if needed, cleanup
        Ok(())
    }
}

impl StorageModule for LocalStorage {
    type Backend = LocalStorage;

    fn info() -> StorageModuleInfo {
        StorageModuleInfo {
            name: "local",
            description: "Local filesystem storage for development and testing",
            version: env!("CARGO_PKG_VERSION"),
        }
    }

    fn config_schema() -> ConfigSchema {
        ConfigSchema::builder()
            .field("path", ConfigFieldType::Path, true)
            .description("Base directory for storing data")
            .env_var("IPC_STORAGE_LOCAL_PATH")
            .field("cache", ConfigFieldType::Boolean, false)
            .description("Enable in-memory caching")
            .default_value(ConfigValue::Boolean(true))
            .build()
    }

    fn from_config(config: &toml::Value) -> Result<Self::Backend, StorageError> {
        let path = config.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| StorageError::Configuration("missing 'path' field".into()))?;

        let use_cache = config.get("cache")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        Ok(LocalStorage::new(PathBuf::from(path), use_cache))
    }
}

// Register the module
ipc_core::register_storage_module!(LocalStorage);
```

#### 3.2 Basin storage module

```rust
// ipc-modules/storage-basin/src/lib.rs

use ipc_core::modules::storage::*;
use ipc_core::modules::config::*;
use async_trait::async_trait;
use reqwest::Client;
use url::Url;

/// Basin hot storage backend
#[derive(Debug)]
pub struct BasinStorage {
    client: Client,
    endpoint: Url,
    bucket: String,
    auth_token: Option<String>,
}

impl BasinStorage {
    pub fn new(endpoint: Url, bucket: String, auth_token: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("failed to create HTTP client");

        Self {
            client,
            endpoint,
            bucket,
            auth_token,
        }
    }
}

#[async_trait]
impl StorageBackend for BasinStorage {
    async fn store(
        &self,
        key: &[u8],
        value: &[u8],
        _options: StoreOptions,
    ) -> StorageResult<()> {
        let url = self.endpoint
            .join(&format!("/buckets/{}/objects/{}", self.bucket, hex::encode(key)))
            .map_err(|e| StorageError::Configuration(e.to_string()))?;

        let mut request = self.client.put(url).body(value.to_vec());

        if let Some(token) = &self.auth_token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await
            .map_err(|e| StorageError::Connection(e.to_string()))?;

        if !response.status().is_success() {
            return Err(StorageError::Backend(
                format!("Basin returned status {}", response.status()).into()
            ));
        }

        Ok(())
    }

    async fn retrieve(
        &self,
        key: &[u8],
        _options: RetrieveOptions,
    ) -> StorageResult<Option<RetrieveResponse>> {
        let url = self.endpoint
            .join(&format!("/buckets/{}/objects/{}", self.bucket, hex::encode(key)))
            .map_err(|e| StorageError::Configuration(e.to_string()))?;

        let mut request = self.client.get(url);

        if let Some(token) = &self.auth_token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await
            .map_err(|e| StorageError::Connection(e.to_string()))?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        if !response.status().is_success() {
            return Err(StorageError::Backend(
                format!("Basin returned status {}", response.status()).into()
            ));
        }

        let value = response.bytes().await
            .map_err(|e| StorageError::Connection(e.to_string()))?;

        Ok(Some(RetrieveResponse {
            value: value.to_vec(),
            metadata: None,
        }))
    }

    async fn delete(&self, key: &[u8]) -> StorageResult<bool> {
        let url = self.endpoint
            .join(&format!("/buckets/{}/objects/{}", self.bucket, hex::encode(key)))
            .map_err(|e| StorageError::Configuration(e.to_string()))?;

        let mut request = self.client.delete(url);

        if let Some(token) = &self.auth_token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await
            .map_err(|e| StorageError::Connection(e.to_string()))?;

        Ok(response.status().is_success())
    }

    async fn exists(&self, key: &[u8]) -> StorageResult<bool> {
        let url = self.endpoint
            .join(&format!("/buckets/{}/objects/{}", self.bucket, hex::encode(key)))
            .map_err(|e| StorageError::Configuration(e.to_string()))?;

        let mut request = self.client.head(url);

        if let Some(token) = &self.auth_token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await
            .map_err(|e| StorageError::Connection(e.to_string()))?;

        Ok(response.status().is_success())
    }

    async fn list_keys(&self, prefix: Option<&[u8]>) -> StorageResult<Vec<Vec<u8>>> {
        // Basin-specific listing implementation
        todo!("implement Basin list API")
    }

    async fn health_check(&self) -> HealthCheckResult {
        let start = std::time::Instant::now();

        let url = match self.endpoint.join("/health") {
            Ok(u) => u,
            Err(e) => return HealthCheckResult {
                healthy: false,
                message: Some(e.to_string()),
                latency: None,
            },
        };

        match self.client.get(url).send().await {
            Ok(resp) if resp.status().is_success() => HealthCheckResult {
                healthy: true,
                message: None,
                latency: Some(start.elapsed()),
            },
            Ok(resp) => HealthCheckResult {
                healthy: false,
                message: Some(format!("status: {}", resp.status())),
                latency: Some(start.elapsed()),
            },
            Err(e) => HealthCheckResult {
                healthy: false,
                message: Some(e.to_string()),
                latency: Some(start.elapsed()),
            },
        }
    }

    async fn shutdown(&self) -> StorageResult<()> {
        Ok(())
    }
}

impl StorageModule for BasinStorage {
    type Backend = BasinStorage;

    fn info() -> StorageModuleInfo {
        StorageModuleInfo {
            name: "basin",
            description: "Hot storage via Textile Basin",
            version: env!("CARGO_PKG_VERSION"),
        }
    }

    fn config_schema() -> ConfigSchema {
        ConfigSchema::builder()
            .field("endpoint", ConfigFieldType::Url, true)
            .description("Basin API endpoint URL")
            .field("bucket", ConfigFieldType::String, true)
            .description("Bucket name for this subnet's data")
            .field("auth_token", ConfigFieldType::String, false)
            .description("Authentication token (can also use IPC_BASIN_TOKEN env var)")
            .env_var("IPC_BASIN_TOKEN")
            .build()
    }

    fn from_config(config: &toml::Value) -> Result<Self::Backend, StorageError> {
        let endpoint = config.get("endpoint")
            .and_then(|v| v.as_str())
            .ok_or_else(|| StorageError::Configuration("missing 'endpoint' field".into()))?;

        let endpoint = Url::parse(endpoint)
            .map_err(|e| StorageError::Configuration(format!("invalid endpoint URL: {}", e)))?;

        let bucket = config.get("bucket")
            .and_then(|v| v.as_str())
            .ok_or_else(|| StorageError::Configuration("missing 'bucket' field".into()))?
            .to_string();

        let auth_token = config.get("auth_token")
            .and_then(|v| v.as_str())
            .map(String::from)
            .or_else(|| std::env::var("IPC_BASIN_TOKEN").ok());

        Ok(BasinStorage::new(endpoint, bucket, auth_token))
    }
}

ipc_core::register_storage_module!(BasinStorage);
```

#### 3.3 Custom actor storage module (stub)

```rust
// ipc-modules/storage-actor/src/lib.rs

use ipc_core::modules::storage::*;
use ipc_core::modules::config::*;
use async_trait::async_trait;

/// On-chain storage via custom IPC actors
#[derive(Debug)]
pub struct ActorStorage {
    // Connection to IPC node for actor invocation
    rpc_endpoint: String,
    actor_address: String,
}

#[async_trait]
impl StorageBackend for ActorStorage {
    // Implementation sends messages to custom storage actor
    // This integrates with IPC's actor system

    async fn store(&self, key: &[u8], value: &[u8], options: StoreOptions) -> StorageResult<()> {
        todo!("implement actor-based storage")
    }

    async fn retrieve(&self, key: &[u8], options: RetrieveOptions) -> StorageResult<Option<RetrieveResponse>> {
        todo!("implement actor-based retrieval")
    }

    async fn delete(&self, key: &[u8]) -> StorageResult<bool> {
        todo!("implement actor-based deletion")
    }

    async fn exists(&self, key: &[u8]) -> StorageResult<bool> {
        todo!("implement actor-based existence check")
    }

    async fn list_keys(&self, prefix: Option<&[u8]>) -> StorageResult<Vec<Vec<u8>>> {
        todo!("implement actor-based key listing")
    }

    async fn health_check(&self) -> HealthCheckResult {
        todo!("implement actor health check")
    }

    async fn shutdown(&self) -> StorageResult<()> {
        Ok(())
    }
}

impl StorageModule for ActorStorage {
    type Backend = ActorStorage;

    fn info() -> StorageModuleInfo {
        StorageModuleInfo {
            name: "actor",
            description: "On-chain storage via custom IPC actors",
            version: env!("CARGO_PKG_VERSION"),
        }
    }

    fn config_schema() -> ConfigSchema {
        ConfigSchema::builder()
            .field("rpc_endpoint", ConfigFieldType::Url, true)
            .description("IPC node RPC endpoint")
            .field("actor_address", ConfigFieldType::String, true)
            .description("Address of the storage actor")
            .build()
    }

    fn from_config(config: &toml::Value) -> Result<Self::Backend, StorageError> {
        let rpc_endpoint = config.get("rpc_endpoint")
            .and_then(|v| v.as_str())
            .ok_or_else(|| StorageError::Configuration("missing 'rpc_endpoint'".into()))?
            .to_string();

        let actor_address = config.get("actor_address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| StorageError::Configuration("missing 'actor_address'".into()))?
            .to_string();

        Ok(ActorStorage {
            rpc_endpoint,
            actor_address,
        })
    }
}

ipc_core::register_storage_module!(ActorStorage);
```

### Acceptance Criteria - Stage 3

- [ ] Local storage module passes all trait compliance tests
- [ ] Basin storage module connects and operates with Basin API
- [ ] Actor storage module compiles (full implementation can be later)
- [ ] All modules register correctly via macro
- [ ] Integration tests for each module

---

## Stage 4: Node and CLI Integration

### Objective

Update `ipc-node` and `ipc-cli` to use the module system.

### Tasks

#### 4.1 Node configuration with modules

```toml
# Example node.toml configuration

[node]
name = "my-subnet-node"
listen_addr = "0.0.0.0:26656"

[consensus]
# Existing consensus configuration
engine = "cometbft"

[modules]
# Module configuration section

[modules.storage]
# Which storage backend to use
backend = "basin"

# Backend-specific configuration
[modules.storage.basin]
endpoint = "https://basin.tableland.xyz"
bucket = "my-subnet-data"
# auth_token loaded from IPC_BASIN_TOKEN env var

# Alternative: local storage for development
# [modules.storage]
# backend = "local"
# [modules.storage.local]
# path = "/var/lib/ipc/storage"
# cache = true
```

```rust
// ipc-node/src/config.rs

use ipc_core::modules::registry::{ModuleRegistry, ModuleRegistryBuilder};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct NodeConfig {
    pub node: NodeSettings,
    pub consensus: ConsensusConfig,
    #[serde(default)]
    pub modules: ModulesConfig,
}

#[derive(Debug, Deserialize, Default)]
pub struct ModulesConfig {
    pub storage: Option<StorageModuleConfig>,
    // Future: pub telemetry: Option<TelemetryModuleConfig>,
}

#[derive(Debug, Deserialize)]
pub struct StorageModuleConfig {
    pub backend: String,
    #[serde(flatten)]
    pub backends: toml::Value, // Contains backend-specific configs
}

impl NodeConfig {
    pub fn build_module_registry(&self) -> Result<ModuleRegistry, ConfigError> {
        let mut builder = ModuleRegistry::builder();

        if let Some(storage_config) = &self.modules.storage {
            let backend_name = &storage_config.backend;
            let backend_config = storage_config.backends
                .get(backend_name)
                .ok_or_else(|| ConfigError::MissingModuleConfig(backend_name.clone()))?;

            builder = builder.with_storage_module(backend_name, backend_config)?;
        }

        Ok(builder.build())
    }
}
```

#### 4.2 Node runtime integration

```rust
// ipc-node/src/runtime.rs

use ipc_core::modules::registry::ModuleRegistry;
use std::sync::Arc;

pub struct NodeRuntime {
    config: NodeConfig,
    modules: Arc<ModuleRegistry>,
    // ... other runtime components
}

impl NodeRuntime {
    pub async fn new(config: NodeConfig) -> Result<Self, RuntimeError> {
        // Build module registry
        let modules = Arc::new(config.build_module_registry()?);

        // Perform health checks on all modules
        if let Some(storage) = modules.storage() {
            let health = storage.health_check().await;
            if !health.healthy {
                return Err(RuntimeError::ModuleHealthCheck(
                    "storage".into(),
                    health.message.unwrap_or_default(),
                ));
            }
            tracing::info!(
                "Storage module healthy, latency: {:?}",
                health.latency
            );
        }

        Ok(Self {
            config,
            modules,
        })
    }

    pub fn modules(&self) -> &ModuleRegistry {
        &self.modules
    }

    pub async fn shutdown(&self) -> Result<(), RuntimeError> {
        self.modules.shutdown().await?;
        Ok(())
    }
}
```

#### 4.3 CLI module commands

```rust
// ipc-cli/src/commands/modules.rs

use clap::{Parser, Subcommand};
use ipc_core::modules::registry::{available_storage_modules, get_storage_module};

#[derive(Parser)]
pub struct ModulesCommand {
    #[command(subcommand)]
    command: ModulesSubcommand,
}

#[derive(Subcommand)]
enum ModulesSubcommand {
    /// List all available modules
    List {
        /// Filter by category (storage, telemetry, etc.)
        #[arg(short, long)]
        category: Option<String>,
    },
    /// Show detailed information about a module
    Info {
        /// Module name
        name: String,
    },
    /// Validate module configuration
    Validate {
        /// Path to configuration file
        #[arg(short, long)]
        config: String,
    },
}

impl ModulesCommand {
    pub fn execute(&self) -> Result<(), CliError> {
        match &self.command {
            ModulesSubcommand::List { category } => {
                self.list_modules(category.as_deref())
            }
            ModulesSubcommand::Info { name } => {
                self.show_module_info(name)
            }
            ModulesSubcommand::Validate { config } => {
                self.validate_config(config)
            }
        }
    }

    fn list_modules(&self, category: Option<&str>) -> Result<(), CliError> {
        println!("Available modules:\n");

        if category.is_none() || category == Some("storage") {
            println!("STORAGE");
            for module in available_storage_modules() {
                println!(
                    "  {:<15} {} [v{}]",
                    module.info.name,
                    module.info.description,
                    module.info.version
                );
            }
            println!();
        }

        // Future: list other module categories

        println!("Run `ipc modules info <name>` for configuration options.");
        Ok(())
    }

    fn show_module_info(&self, name: &str) -> Result<(), CliError> {
        // Try storage modules
        if let Some(module) = get_storage_module(name) {
            println!("Module: {}", module.info.name);
            println!("Category: storage");
            println!("Version: {}", module.info.version);
            println!("Description: {}", module.info.description);
            println!();
            println!("Configuration:");

            for field in &module.config_schema.fields {
                let required = if field.required { "(required)" } else { "(optional)" };
                println!(
                    "  {:<15} {}  {}",
                    field.name,
                    required,
                    field.description
                );
                if let Some(env_var) = &field.env_var {
                    println!("                 env: {}", env_var);
                }
                if let Some(default) = &field.default {
                    println!("                 default: {:?}", default);
                }
            }

            println!();
            println!("Example configuration:");
            println!("{}", module.config_schema.example_toml());

            return Ok(());
        }

        Err(CliError::ModuleNotFound(name.to_string()))
    }

    fn validate_config(&self, config_path: &str) -> Result<(), CliError> {
        let config_str = std::fs::read_to_string(config_path)?;
        let config: toml::Value = toml::from_str(&config_str)?;

        // Validate storage module config
        if let Some(modules) = config.get("modules") {
            if let Some(storage) = modules.get("storage") {
                let backend = storage.get("backend")
                    .and_then(|v| v.as_str())
                    .ok_or(CliError::InvalidConfig("missing storage.backend".into()))?;

                if let Some(module) = get_storage_module(backend) {
                    let backend_config = storage.get(backend)
                        .ok_or(CliError::InvalidConfig(
                            format!("missing storage.{} configuration", backend)
                        ))?;

                    module.config_schema.validate(backend_config)?;
                    println!("✓ Storage module [{}] configuration valid", backend);

                    // Optionally test connectivity
                    // ...
                } else {
                    return Err(CliError::ModuleNotFound(backend.to_string()));
                }
            }
        }

        println!("✓ Configuration valid");
        Ok(())
    }
}
```

### Acceptance Criteria - Stage 4

- [ ] Node loads configuration with module settings
- [ ] Node initializes modules from configuration
- [ ] Module health checks run on startup
- [ ] CLI `modules list` shows available modules
- [ ] CLI `modules info <name>` shows configuration schema
- [ ] CLI `modules validate` validates configuration files
- [ ] Graceful shutdown properly closes modules

---

## Stage 5: Testing Infrastructure

### Objective

Build comprehensive testing utilities for modules.

### Tasks

#### 5.1 Module test suite

```rust
// ipc-core/src/modules/testing.rs

use crate::modules::storage::*;
use std::time::Duration;

/// Standard test suite for storage backends
pub struct StorageTestSuite;

impl StorageTestSuite {
    /// Run all compliance tests against a storage backend
    pub async fn run<B: StorageBackend>(backend: &B) {
        Self::test_store_retrieve(backend).await;
        Self::test_delete(backend).await;
        Self::test_exists(backend).await;
        Self::test_overwrite_behavior(backend).await;
        Self::test_nonexistent_key(backend).await;
        Self::test_health_check(backend).await;
        Self::test_concurrent_access(backend).await;
    }

    async fn test_store_retrieve<B: StorageBackend>(backend: &B) {
        let key = b"test_key_1";
        let value = b"test_value_1";

        // Store
        backend.store(key, value, StoreOptions::default()).await
            .expect("store should succeed");

        // Retrieve
        let result = backend.retrieve(key, RetrieveOptions::default()).await
            .expect("retrieve should succeed")
            .expect("value should exist");

        assert_eq!(result.value, value.to_vec(), "retrieved value should match stored value");
    }

    async fn test_delete<B: StorageBackend>(backend: &B) {
        let key = b"test_key_delete";
        let value = b"test_value_delete";

        // Store then delete
        backend.store(key, value, StoreOptions::default()).await.unwrap();
        let deleted = backend.delete(key).await.expect("delete should succeed");
        assert!(deleted, "delete should return true for existing key");

        // Verify deleted
        let result = backend.retrieve(key, RetrieveOptions::default()).await.unwrap();
        assert!(result.is_none(), "deleted key should not exist");

        // Delete non-existent
        let deleted_again = backend.delete(key).await.expect("delete should succeed");
        assert!(!deleted_again, "delete should return false for non-existent key");
    }

    async fn test_exists<B: StorageBackend>(backend: &B) {
        let key = b"test_key_exists";
        let value = b"test_value_exists";

        assert!(!backend.exists(key).await.unwrap(), "key should not exist initially");

        backend.store(key, value, StoreOptions::default()).await.unwrap();
        assert!(backend.exists(key).await.unwrap(), "key should exist after store");

        backend.delete(key).await.unwrap();
        assert!(!backend.exists(key).await.unwrap(), "key should not exist after delete");
    }

    async fn test_overwrite_behavior<B: StorageBackend>(backend: &B) {
        let key = b"test_key_overwrite";
        let value1 = b"value_1";
        let value2 = b"value_2";

        // Initial store
        backend.store(key, value1, StoreOptions::default()).await.unwrap();

        // Overwrite with default options (should succeed)
        backend.store(key, value2, StoreOptions::default()).await.unwrap();

        let result = backend.retrieve(key, RetrieveOptions::default()).await.unwrap().unwrap();
        assert_eq!(result.value, value2.to_vec());

        // Cleanup
        backend.delete(key).await.unwrap();
    }

    async fn test_nonexistent_key<B: StorageBackend>(backend: &B) {
        let key = b"definitely_does_not_exist_12345";

        let result = backend.retrieve(key, RetrieveOptions::default()).await
            .expect("retrieve should not error for non-existent key");

        assert!(result.is_none(), "non-existent key should return None");
    }

    async fn test_health_check<B: StorageBackend>(backend: &B) {
        let health = backend.health_check().await;
        assert!(health.healthy, "health check should pass: {:?}", health.message);
    }

    async fn test_concurrent_access<B: StorageBackend>(backend: &B) {
        use tokio::task::JoinSet;

        let mut tasks = JoinSet::new();

        // Spawn concurrent store operations
        for i in 0..10 {
            let key = format!("concurrent_key_{}", i).into_bytes();
            let value = format!("concurrent_value_{}", i).into_bytes();

            // Note: In real impl, backend would need to be Arc<B>
            tasks.spawn(async move {
                // This is a simplified example - real test would use Arc
                (i, key, value)
            });
        }

        // In actual test, verify all operations completed
    }
}

/// Mock storage backend for testing code that uses storage
#[derive(Debug, Default)]
pub struct MockStorage {
    data: std::sync::RwLock<std::collections::HashMap<Vec<u8>, Vec<u8>>>,
    fail_next: std::sync::atomic::AtomicBool,
}

impl MockStorage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fail_next_operation(&self) {
        self.fail_next.store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

#[async_trait::async_trait]
impl StorageBackend for MockStorage {
    async fn store(&self, key: &[u8], value: &[u8], _: StoreOptions) -> StorageResult<()> {
        if self.fail_next.swap(false, std::sync::atomic::Ordering::SeqCst) {
            return Err(StorageError::Backend("simulated failure".into()));
        }
        self.data.write().unwrap().insert(key.to_vec(), value.to_vec());
        Ok(())
    }

    async fn retrieve(&self, key: &[u8], _: RetrieveOptions) -> StorageResult<Option<RetrieveResponse>> {
        if self.fail_next.swap(false, std::sync::atomic::Ordering::SeqCst) {
            return Err(StorageError::Backend("simulated failure".into()));
        }
        Ok(self.data.read().unwrap().get(key).map(|v| RetrieveResponse {
            value: v.clone(),
            metadata: None,
        }))
    }

    async fn delete(&self, key: &[u8]) -> StorageResult<bool> {
        Ok(self.data.write().unwrap().remove(key).is_some())
    }

    async fn exists(&self, key: &[u8]) -> StorageResult<bool> {
        Ok(self.data.read().unwrap().contains_key(key))
    }

    async fn list_keys(&self, prefix: Option<&[u8]>) -> StorageResult<Vec<Vec<u8>>> {
        let data = self.data.read().unwrap();
        Ok(data.keys()
            .filter(|k| prefix.map(|p| k.starts_with(p)).unwrap_or(true))
            .cloned()
            .collect())
    }

    async fn health_check(&self) -> HealthCheckResult {
        HealthCheckResult {
            healthy: true,
            message: None,
            latency: Some(Duration::from_micros(1)),
        }
    }

    async fn shutdown(&self) -> StorageResult<()> {
        Ok(())
    }
}
```

### Acceptance Criteria - Stage 5

- [ ] StorageTestSuite runs against all storage implementations
- [ ] MockStorage available for unit testing
- [ ] All tests pass for local, basin modules
- [ ] CI integration for module tests

---

## Future Stages (Roadmap)

### Stage 6: Additional Module Types

- Telemetry modules (Prometheus, OpenTelemetry)
- Networking modules (transport configurations)
- Execution modules (FVM variants)

### Stage 7: Dynamic Plugin Loading (Optional)

- Define stable ABI for plugins
- Implement plugin discovery and loading
- Security considerations for third-party plugins

### Stage 8: Module Marketplace

- Documentation generation from ConfigSchema
- Module versioning and compatibility matrix
- Community module contributions

---

## Implementation Notes

### Cargo Features

Use feature flags for optional module inclusion:

```toml
# ipc-node/Cargo.toml
[features]
default = ["storage-local"]
storage-local = ["ipc-modules-storage-local"]
storage-basin = ["ipc-modules-storage-basin"]
storage-actor = ["ipc-modules-storage-actor"]
all-storage = ["storage-local", "storage-basin", "storage-actor"]
```

### Error Handling

All module errors should:
1. Be convertible to a common error type
2. Include context about which module failed
3. Be actionable (suggest fixes where possible)

### Logging

Modules should use `tracing` with structured fields:

```rust
tracing::info!(
    module = "storage",
    backend = "basin",
    operation = "store",
    key_size = key.len(),
    value_size = value.len(),
    "storing value"
);
```

### Configuration Precedence

1. CLI arguments (highest)
2. Environment variables
3. Configuration file
4. Default values (lowest)

---

## References

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Tokio Best Practices](https://tokio.rs/tokio/topics/bridging)
- [Plugin Architecture Patterns](https://nullderef.com/blog/plugin-tech/)