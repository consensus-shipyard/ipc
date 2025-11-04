# Recall Storage Deployment Guide

Complete guide to deploying IPC validators with Recall blob storage functionality.

---

## 📦 Part 1: Build & Compile

### What You Need to Build

```bash
cd /path/to/ipc

# 1. Build the Fendermint binary (includes storage node components)
cargo build --release -p fendermint_app

# 2. Build Recall actors (for on-chain blob management)
cd fendermint/actors
cargo build --release --target wasm32-unknown-unknown \
  -p fendermint_actor_blobs \
  -p fendermint_actor_blob_reader \
  -p fendermint_actor_recall_config

# 3. Optional: Build IPC CLI (for network management)
cd ../../
cargo build --release -p ipc-cli
```

### Verify the Build

```bash
# Check fendermint binary exists
ls -lh target/release/fendermint

# Check it includes the objects command
target/release/fendermint --help | grep objects
# Should show: objects    Run the objects HTTP API server

# Check actors were compiled
ls -lh target/wasm32-unknown-unknown/release/fendermint_actor_*.wasm
```

---

## ⚙️ Part 2: Configuration

### A. Create Fendermint Configuration

Each validator needs a `fendermint` configuration file (typically `config.toml`):

```toml
# config.toml

# Base directories
data_dir = "data"
snapshots_dir = "snapshots"
contracts_dir = "contracts"

# CometBFT connection
tendermint_rpc_url = "http://127.0.0.1:26657"
tendermint_websocket_url = "ws://127.0.0.1:26657/websocket"

[abci]
listen = { host = "127.0.0.1", port = 26658 }

[eth]
listen = { host = "0.0.0.0", port = 8545 }

# ============================================
# STORAGE NODE CONFIGURATION (NEW!)
# ============================================

[objects]
# Maximum file size for uploads (100MB default)
max_object_size = 104857600
# HTTP API listen address for blob uploads/downloads
listen = { host = "0.0.0.0", port = 8080 }

[objects.metrics]
enabled = true
listen = { host = "127.0.0.1", port = 9186 }

# ============================================
# IROH RESOLVER CONFIGURATION (NEW!)
# ============================================

[resolver.iroh_resolver_config]
# IPv4 address for Iroh node (P2P blob transfer)
# Leave as None to bind to all interfaces with default port 11204
v4_addr = "0.0.0.0:11204"

# IPv6 address (optional)
# v6_addr = "[::]:11205"

# Directory where Iroh stores blobs
iroh_data_dir = "data/iroh_resolver"

# RPC address for Iroh client communication
rpc_addr = "127.0.0.1:4444"

# ============================================
# RESOLVER P2P SETTINGS
# ============================================

[resolver.network]
# Cryptographic key for P2P resolver network
local_key = "keys/network.sk"
network_name = "my-ipc-network"

[resolver.connection]
# Multiaddr to listen on for P2P connections
listen_addr = "/ip4/0.0.0.0/tcp/0"
external_addresses = []
max_incoming = 30

[resolver.membership]
# Subnets to track (empty = track all)
static_subnets = []
max_subnets = 100

[resolver.content]
# Rate limiting (0 = no limit)
rate_limit_bytes = 0
rate_limit_period = 0
```

### B. Directory Structure

Each validator node needs:

```
/path/to/validator/
├── config.toml                    # Main configuration
├── fendermint                     # Binary
├── data/                          # Blockchain data
│   ├── iroh_resolver/            # Iroh blob storage (NEW!)
│   │   ├── blobs/                # Actual blob data
│   │   └── iroh_key              # Iroh node identity
│   └── fendermint.db/            # State database
├── keys/
│   ├── validator.sk              # Validator key
│   └── network.sk                # P2P network key
└── cometbft/                     # CometBFT config/data
    └── config/
        └── config.toml
```

---

## 🚀 Part 3: Running the Nodes

### Option A: Integrated Mode (Validator + Storage in One Process)

This runs the validator node with built-in storage capabilities:

```bash
# Start the validator node with storage
./fendermint run \
  --home /path/to/validator \
  --config config.toml

# This automatically starts:
# 1. ABCI application (port 26658)
# 2. Ethereum API (port 8545)
# 3. IPLD Resolver with Iroh (port 11204/11205 for P2P)
# 4. Objects HTTP API (port 8080) - if enabled
```

**What's Running:**
- ✅ Validator/consensus via CometBFT
- ✅ FVM execution engine
- ✅ Iroh storage node (automatic, embedded)
- ✅ P2P blob resolution network
- ✅ Objects HTTP API (if configured)

### Option B: Separate Objects HTTP Server (Optional)

If you want to run the Objects HTTP API separately (e.g., on edge nodes):

```bash
# Terminal 1: Run validator node
./fendermint run --home /path/to/validator --config config.toml

# Terminal 2: Run standalone Objects HTTP API
./fendermint objects run \
  --tendermint-url http://localhost:26657 \
  --iroh-path /path/to/iroh_data \
  --iroh-resolver-rpc-addr 127.0.0.1:4444 \
  --iroh-v4-addr 0.0.0.0:11204
```

**Use Case**: Separate upload/download nodes from consensus validators.

---

## 🔧 Part 4: Port Configuration

### Ports You Need to Open

| Port | Protocol | Purpose | Firewall Rule |
|------|----------|---------|---------------|
| **26656** | TCP | CometBFT P2P | Allow from other validators |
| **26657** | TCP | CometBFT RPC | Internal only (or allow from trusted sources) |
| **26658** | TCP | ABCI Application | Internal only (localhost) |
| **8545** | TCP | Ethereum JSON-RPC | Allow from clients |
| **8080** | TCP | **Objects HTTP API (NEW!)** | Allow from clients uploading/downloading blobs |
| **11204** | UDP | **Iroh P2P IPv4 (NEW!)** | Allow from all validators |
| **11205** | UDP | **Iroh P2P IPv6 (NEW!)** | Allow from all validators (if using IPv6) |
| **4444** | TCP | **Iroh RPC (NEW!)** | Internal only (localhost) |

**Key Storage Ports:**
- **8080**: HTTP API for blob upload/download
- **11204/11205**: Iroh P2P for validator-to-validator blob transfer
- **4444**: Iroh RPC for local communication (keep internal)

---

## 🧪 Part 5: Testing Blob Upload

### Step 1: Verify Storage Node is Running

```bash
# Check Objects HTTP API is accessible
curl http://localhost:8080/health
# Expected: {"status":"ok"}

# Check Iroh node is running (look for logs)
tail -f /path/to/validator/logs/fendermint.log | grep -i iroh
# Expected: "creating persistent iroh node"
# Expected: "Iroh RPC listening on 127.0.0.1:4444"
```

### Step 2: Upload a Test File

```bash
# Create a test file
echo "Hello, Recall Storage!" > test.txt

# Upload via Objects HTTP API
curl -X POST http://localhost:8080/upload \
  -F "file=@test.txt" \
  -F "content_type=text/plain"

# Response includes:
# {
#   "blob_hash": "bafk...",
#   "seq_hash": "bafk...",
#   "upload_id": "uuid",
#   "size": 23,
#   "chunks": 1
# }

# Save the blob_hash for later!
BLOB_HASH="<blob_hash_from_response>"
```

### Step 3: Verify Blob Storage

```bash
# Check blob exists in Iroh storage
ls -lh /path/to/validator/data/iroh_resolver/blobs/

# Query blob metadata (if Blobs actor is deployed)
curl http://localhost:8545 \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "eth_call",
    "params": [{
      "to": "0xBlobsActorAddress",
      "data": "0x..."
    }, "latest"],
    "id": 1
  }'
```

### Step 4: Download the Blob

```bash
# Download from the same node
curl http://localhost:8080/download/$BLOB_HASH \
  -o downloaded.txt

# Verify it matches
diff test.txt downloaded.txt
# Should show no differences
```

### Step 5: Test Multi-Validator Resolution

```bash
# On Validator 2, download blob uploaded to Validator 1
# This tests P2P blob transfer via Iroh

# First, get Validator 1's Iroh node ID
curl http://validator1:8080/node_info
# Response: { "node_id": "...", "addrs": [...] }

# On Validator 2, download the blob
curl -X POST http://validator2:8080/download \
  -H "Content-Type: application/json" \
  -d '{
    "blob_hash": "'$BLOB_HASH'",
    "source_node": "<validator1_node_id>",
    "source_addrs": ["<validator1_addr>"]
  }'

# This triggers:
# 1. Validator 2 connects to Validator 1 via Iroh P2P
# 2. Downloads blob chunks
# 3. Reconstructs file
# 4. Submits resolution vote to vote tally
```

---

## 📊 Part 6: Monitoring

### Check Storage Node Health

```bash
# Objects API metrics
curl http://localhost:9186/metrics | grep object

# Iroh stats (from logs)
tail -f /path/to/validator/logs/fendermint.log | grep -i "blob\|iroh"

# Check storage usage
du -sh /path/to/validator/data/iroh_resolver/blobs/
```

### Monitor Blob Resolution

```bash
# Watch for blob events in logs
tail -f /path/to/validator/logs/fendermint.log | grep -i "blob.*resolved\|vote"

# Check vote tally (requires RPC call to chain)
# This shows which blobs reached consensus
```

### Prometheus Metrics (if enabled)

```bash
# Objects API metrics
curl http://localhost:9186/metrics

# Key metrics:
# - fendermint_objects_upload_total
# - fendermint_objects_upload_bytes
# - fendermint_objects_download_total
# - fendermint_objects_download_bytes
```

---

## 🔐 Part 7: Security Considerations

### Firewall Configuration

```bash
# Allow CometBFT P2P from other validators
ufw allow from <validator_ip> to any port 26656 proto tcp

# Allow Iroh P2P from other validators
ufw allow from <validator_ip> to any port 11204 proto udp

# Allow Objects API from clients (public or restricted)
ufw allow from <client_subnet> to any port 8080 proto tcp

# Allow Ethereum RPC from clients
ufw allow from <client_subnet> to any port 8545 proto tcp

# Keep internal ports closed
ufw deny 26657  # CometBFT RPC
ufw deny 26658  # ABCI
ufw deny 4444   # Iroh RPC
```

### Authentication (Future Enhancement)

Currently, the Objects HTTP API has no authentication. For production:

1. **Use a reverse proxy** (nginx, Traefik) with auth
2. **Network segmentation** - Only allow from trusted sources
3. **Rate limiting** - Prevent abuse

---

## 🐛 Troubleshooting

### Blob Upload Fails

```bash
# Check Objects API is running
curl http://localhost:8080/health

# Check disk space
df -h /path/to/validator/data/

# Check logs for errors
tail -f /path/to/validator/logs/fendermint.log | grep -i error
```

### Iroh Node Won't Start

```bash
# Check port 11204/11205 are available
netstat -tuln | grep 11204

# Check Iroh data directory permissions
ls -ld /path/to/validator/data/iroh_resolver/

# Check for error logs
tail -f /path/to/validator/logs/fendermint.log | grep -i iroh
```

### Blob Not Replicating to Other Validators

```bash
# Check Iroh P2P connectivity
# Look for "connected to peer" in logs
tail -f /path/to/validator/logs/fendermint.log | grep -i "peer\|connect"

# Check firewall allows UDP 11204
# On validator 1:
nc -u -l 11204

# On validator 2:
nc -u validator1 11204
# Type something and press Enter
```

### Vote Tally Not Working

```bash
# Check vote submissions in logs
tail -f /path/to/validator/logs/fendermint.log | grep -i "vote.*blob"

# Verify validator keys are configured
ls -l /path/to/validator/keys/validator.sk

# Check validators are active
curl http://localhost:26657/validators
```

---

## 📝 Complete Example: 3-Validator Network

### Validator 1 Config

```toml
# validator1/config.toml
[objects]
listen = { host = "0.0.0.0", port = 8080 }
max_object_size = 104857600

[resolver.iroh_resolver_config]
v4_addr = "0.0.0.0:11204"
iroh_data_dir = "data/iroh_resolver"
rpc_addr = "127.0.0.1:4444"

[resolver.connection]
listen_addr = "/ip4/0.0.0.0/tcp/7001"
external_addresses = ["/ip4/192.168.1.101/tcp/7001"]
```

### Validator 2 Config

```toml
# validator2/config.toml
[objects]
listen = { host = "0.0.0.0", port = 8080 }
max_object_size = 104857600

[resolver.iroh_resolver_config]
v4_addr = "0.0.0.0:11204"
iroh_data_dir = "data/iroh_resolver"
rpc_addr = "127.0.0.1:4444"

[resolver.connection]
listen_addr = "/ip4/0.0.0.0/tcp/7001"
external_addresses = ["/ip4/192.168.1.102/tcp/7001"]
```

### Validator 3 Config

```toml
# validator3/config.toml
[objects]
listen = { host = "0.0.0.0", port = 8080 }
max_object_size = 104857600

[resolver.iroh_resolver_config]
v4_addr = "0.0.0.0:11204"
iroh_data_dir = "data/iroh_resolver"
rpc_addr = "127.0.0.1:4444"

[resolver.connection]
listen_addr = "/ip4/0.0.0.0/tcp/7001"
external_addresses = ["/ip4/192.168.1.103/tcp/7001"]
```

### Start All Validators

```bash
# Terminal 1 (Validator 1)
./fendermint run --home validator1 --config validator1/config.toml

# Terminal 2 (Validator 2)
./fendermint run --home validator2 --config validator2/config.toml

# Terminal 3 (Validator 3)
./fendermint run --home validator3 --config validator3/config.toml
```

### Test Cross-Validator Resolution

```bash
# Upload to Validator 1
curl -X POST http://validator1:8080/upload -F "file=@bigfile.dat"
# Returns blob_hash

# Download from Validator 2 (triggers P2P transfer)
curl http://validator2:8080/download/<blob_hash> -o downloaded.dat

# Verify Validator 3 also has it (after resolution)
curl http://validator3:8080/download/<blob_hash> -o downloaded3.dat

# All files should match
md5sum bigfile.dat downloaded.dat downloaded3.dat
```

---

## 🎯 Quick Start Checklist

- [ ] Build `fendermint` binary
- [ ] Build Recall actors (blobs, blob_reader, recall_config)
- [ ] Create `config.toml` with `[objects]` and `[resolver.iroh_resolver_config]`
- [ ] Create directory structure (data/iroh_resolver/, keys/, etc.)
- [ ] Open firewall ports (8080, 11204 UDP)
- [ ] Start fendermint: `./fendermint run --config config.toml`
- [ ] Test upload: `curl -X POST http://localhost:8080/upload -F "file=@test.txt"`
- [ ] Test download: `curl http://localhost:8080/download/<blob_hash>`
- [ ] Monitor logs: `tail -f logs/fendermint.log | grep -i "blob\|iroh"`

---

## 📚 Additional Resources

- **Architecture**: See `RECALL_MIGRATION_SUMMARY.md`
- **API Reference**: See `fendermint/app/src/cmd/objects.rs`
- **Configuration**: See `fendermint/app/settings/src/`
- **Troubleshooting**: See `INTERPRETER_FILES_ANALYSIS.md`

---

**Ready to deploy? Start with a single validator test, then scale to your full network!**

