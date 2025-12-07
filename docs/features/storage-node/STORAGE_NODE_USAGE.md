# Storage-Node Plugin - Usage Guide

## Overview

The storage-node functionality is now a **separate plugin** that provides a storage HTTP API service for managing objects/blobs. It runs as its own service, separate from the main Fendermint node.

## Building with Storage-Node Plugin

### 1. Build Fendermint with Plugin
```bash
# Build with storage-node plugin enabled
cargo build --release --features plugin-storage-node

# Or use make (but you need to add the feature flag)
# Note: Default make does NOT include plugins
```

### 2. Verify Plugin is Available
```bash
# Check if 'objects' command appears
./target/release/fendermint --help

# You should see:
#   objects       Subcommands related to the Objects/Blobs storage HTTP API
```

## Running the Storage Node

### Architecture
The storage-node plugin provides a **separate service** from the main Fendermint node:

```
┌─────────────────────┐
│  Tendermint Core    │
│                     │
└──────────┬──────────┘
           │ ABCI
           │
┌──────────▼──────────┐
│  Fendermint Run     │ ← Main consensus node (fendermint run)
│  (with plugin)      │
└─────────────────────┘

┌─────────────────────┐
│  Storage HTTP API   │ ← Storage service (fendermint objects run)
│  (Objects Service)  │
└──────────┬──────────┘
           │
           │ Queries Tendermint
           ▼
     [Iroh/Blobs]
```

### Starting the Services

#### 1. Start Main Fendermint Node
```bash
# This runs the ABCI application (consensus)
fendermint run

# The plugin is loaded automatically when built with --features plugin-storage-node
# It handles ReadRequest messages in the blockchain layer
```

#### 2. Start Storage HTTP API (Separate Service)
```bash
# This runs the storage HTTP API server
fendermint objects run \
  --tendermint-url http://127.0.0.1:26657 \
  --iroh-path /path/to/iroh/data \
  --iroh-resolver-rpc-addr 127.0.0.1:4444 \
  --iroh-v4-addr 0.0.0.0:11204 \
  --iroh-v6-addr [::]:11204
```

### Configuration Options

#### `fendermint objects run` Options:

| Option | Description | Default/Required |
|--------|-------------|------------------|
| `--tendermint-url` / `-t` | Tendermint RPC endpoint | `http://127.0.0.1:26657` |
| `--iroh-path` / `-i` | Path to Iroh data directory | Required (env: `IROH_PATH`) |
| `--iroh-resolver-rpc-addr` | Iroh RPC address | Required (env: `IROH_RESOLVER_RPC_ADDR`) |
| `--iroh-v4-addr` | IPv4 bind address for Iroh | Optional (env: `IROH_V4_ADDR`) |
| `--iroh-v6-addr` | IPv6 bind address for Iroh | Optional (env: `IROH_V6_ADDR`) |

### Configuration File

You can also configure the storage service via the config file at `~/.fendermint/config.toml`:

```toml
[objects]
# Storage service settings
...
```

## How It Works

### When Plugin is Enabled (`--features plugin-storage-node`)

1. **Blockchain Layer** (`fendermint run`)
   - The plugin is loaded automatically via `AppModule`
   - Implements `MessageHandlerModule` to process storage-related messages
   - Handles `ReadRequestPending` and `ReadRequestClosed` IPC messages
   - Uses `RecallExecutor` for FVM execution

2. **Storage HTTP API** (`fendermint objects run`)
   - Runs as a **separate HTTP service**
   - Provides REST API for uploading/downloading blobs
   - Connects to Tendermint to query blockchain state
   - Integrates with Iroh for content-addressed storage
   - Handles entanglement/erasure coding

### When Plugin is NOT Enabled (Default Build)

- `fendermint run` works normally but uses `NoOpModuleBundle`
- Storage-related IPC messages will fail with an error
- `fendermint objects` command does NOT exist
- Smaller binary, faster compilation

## Example: Full Storage-Node Deployment

### 1. Build with Plugin
```bash
cd /Users/philip/github/ipc
cargo build --release --features plugin-storage-node
```

### 2. Start Tendermint (Terminal 1)
```bash
tendermint start --home ~/.tendermint
```

### 3. Start Fendermint ABCI App (Terminal 2)
```bash
# This includes the storage plugin for message handling
./target/release/fendermint run \
  --home-dir ~/.fendermint \
  --network testnet
```

### 4. Start Storage HTTP API (Terminal 3)
```bash
# This provides the HTTP API for blob operations
./target/release/fendermint objects run \
  --tendermint-url http://127.0.0.1:26657 \
  --iroh-path ~/.fendermint/iroh \
  --iroh-resolver-rpc-addr 127.0.0.1:4444
```

### 5. Use Storage API
```bash
# Upload a blob
curl -X POST http://localhost:8080/upload \
  -F "file=@mydata.bin"

# Download a blob
curl http://localhost:8080/download/<blob-hash>
```

## Differences from Before

### Before (Monolithic)
- Storage code was **hardcoded** into fendermint core
- Always compiled, even if not used
- Couldn't build without storage dependencies

### After (Plugin Architecture) ✨

**Default Build (No Plugin):**
```bash
cargo build --release
# ✅ No storage code
# ✅ Smaller binary
# ✅ Faster compilation
# ✅ Works for basic IPC use cases
```

**With Storage Plugin:**
```bash
cargo build --release --features plugin-storage-node
# ✅ Full storage functionality
# ✅ Storage message handlers in blockchain
# ✅ Objects HTTP API available
# ✅ RecallExecutor for FVM
```

## Plugin Implementation Details

### What the Plugin Provides

1. **`ModuleBundle` Implementation** (`StorageNodeModule`)
   - Registers with fendermint module system
   - Provides custom executor, message handlers, etc.

2. **`ExecutorModule`**
   - Uses `RecallExecutor` for FVM execution
   - Handles storage-specific actor calls

3. **`MessageHandlerModule`**
   - Processes `ReadRequestPending` IPC messages
   - Processes `ReadRequestClosed` IPC messages
   - Integrates with storage actors

4. **`Objects` HTTP API** (via `fendermint objects run`)
   - Upload/download blobs
   - Query storage state
   - Entanglement operations

## Troubleshooting

### Objects Command Not Found
```bash
$ fendermint objects run
error: unexpected argument 'objects' found
```

**Solution:** You need to build with the plugin feature:
```bash
cargo build --release --features plugin-storage-node
```

### Storage Messages Fail
If you're running `fendermint run` without the plugin, storage-related IPC messages will fail:

```
Error: Storage message requires the plugin-storage-node feature
```

**Solution:** Rebuild with the plugin:
```bash
cargo build --release --features plugin-storage-node
```

### Configuration File Not Found
The objects service looks for configuration at `~/.fendermint/config/objects.toml`

**Solution:** Ensure config directory exists or use command-line flags

## Summary

**Key Points:**
- ✅ Storage-node is now a **plugin** (`--features plugin-storage-node`)
- ✅ **Two separate services**: `fendermint run` (consensus) + `fendermint objects run` (storage HTTP API)
- ✅ **Default build has no storage code** - opt-in only
- ✅ **No changes to main fendermint run** - plugin loads automatically when enabled
- ✅ **Objects command** only available when built with plugin feature

**Quick Commands:**
```bash
# Build with plugin
cargo build --release --features plugin-storage-node

# Run consensus node (includes plugin)
fendermint run

# Run storage HTTP API (separate service)
fendermint objects run --tendermint-url http://127.0.0.1:26657 --iroh-path ~/.iroh --iroh-resolver-rpc-addr 127.0.0.1:4444
```

---

**For more information:**
- `PLUGIN_USAGE.md` - General plugin architecture
- `QUICK_START_PLUGINS.md` - Quick reference
- `fendermint objects run --help` - Storage service options
