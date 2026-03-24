# Relayer and Resolver Configuration Fix

## Issues Found

### Issue 1: Relayer Missing Required Arguments
The relayer service is failing with:
```
error: the following required arguments were not provided:
  --fendermint-rpc-url <FENDERMINT_RPC_URL>
```

**Root Cause:** The systemd service template was missing the `--fendermint-rpc-url` parameter that the relayer command requires. This parameter specifies the child subnet's ETH API endpoint (http://localhost:8545).

**Solution:** Add the `--fendermint-rpc-url` parameter to the systemd service template and regenerate the service.

### Issue 2: Invalid Fendermint Configuration
The node init config includes invalid configuration sections:
```toml
[resolver.connection.parent]
http_endpoint = "..."

[resolver.subnet]
id = "..."

[resolver.subnet.parent_gateway]
address = "..."
```

**Root Cause:** These configuration paths don't exist in the current Fendermint settings structure. The parent gateway configuration should only be in `[ipc.topdown]`, not in `[resolver]`.

**Solution:** Remove the invalid configuration sections from the node-init.yml generation.

## Fixes Applied

### Fix 1: Update lib/config.sh

Removed invalid resolver configuration sections:

```diff
  [resolver.connection]
  listen_addr = "/ip4/0.0.0.0/tcp/$libp2p_port"

- [resolver.connection.parent]
- http_endpoint = "$parent_rpc"
-
- [resolver.subnet]
- id = "$subnet_id"
-
- [resolver.subnet.parent_gateway]
- address = "$parent_gateway"
-
  [resolver.network]
  local_key = "validator.sk"
```

The parent configuration is already correctly placed in `[ipc.topdown]`:
```toml
[ipc.topdown]
parent_http_endpoint = "$parent_rpc"
parent_registry = "$parent_registry"
parent_gateway = "$parent_gateway"
```

### Fix 2: Update Relayer Systemd Service Template

Added the missing `--fendermint-rpc-url` parameter:

**File: `templates/ipc-relayer.service.template`**
```diff
ExecStart=__IPC_BINARY__ checkpoint relayer \
    --subnet __SUBNET_ID__ \
+   --fendermint-rpc-url __FENDERMINT_RPC_URL__ \
    --checkpoint-interval-sec __CHECKPOINT_INTERVAL__ \
    --max-parallelism __MAX_PARALLELISM__ \
    --submitter __SUBMITTER_ADDRESS__
```

**File: `lib/health.sh` - `generate_relayer_systemd_service()`**
```diff
+   local eth_api_port=$(get_config_value "network.eth_api_port")
+
+   # Fendermint RPC URL is the local ETH API endpoint
+   local fendermint_rpc_url="http://localhost:${eth_api_port}"

    sed -e "s|__IPC_USER__|$ipc_user|g" \
        -e "s|__IPC_BINARY__|$ipc_binary|g" \
        -e "s|__NODE_HOME__|$node_home|g" \
        -e "s|__SUBNET_ID__|$subnet_id|g" \
+       -e "s|__FENDERMINT_RPC_URL__|$fendermint_rpc_url|g" \
        -e "s|__CHECKPOINT_INTERVAL__|$checkpoint_interval|g" \
        -e "s|__MAX_PARALLELISM__|$max_parallelism|g" \
        -e "s|__SUBMITTER_ADDRESS__|$submitter|g" \
        "${SCRIPT_DIR}/templates/ipc-relayer.service.template" > "$output_file"
```

## Steps to Fix

### 1. Reinstall Relayer Systemd Service

The fixes have been applied to the templates. Now reinstall the relayer service:

```bash
./ipc-manager install-systemd --with-relayer --yes
```

This will regenerate the service file with the corrected `--fendermint-rpc-url` parameter.

### 2. Restart the Relayer

```bash
# Stop the old relayer
./ipc-manager stop-relayer

# Start with new configuration
./ipc-manager start-relayer

# Verify it's running
./ipc-manager relayer-status
```

Or use systemd directly on the primary validator:
```bash
ssh philip@34.73.187.192 "sudo systemctl restart ipc-relayer"
./ipc-manager relayer-status
```

## Steps to Fix Node Configuration

### 1. Re-initialize Nodes

Since the fendermint-overrides section has been fixed in `lib/config.sh`, you need to re-run the init process:

```bash
./ipc-manager init --yes
```

This will:
1. Apply the corrected fendermint configuration
2. Re-create the default.toml files with valid settings
3. Restart all nodes with correct configuration

### 2. Verify Configuration

Check that the fendermint config is correct:

```bash
ssh philip@34.73.187.192 "cat /home/ipc/.ipc-node/fendermint/config/default.toml | grep -A 10 '\[ipc.topdown\]'"
```

Should show:
```toml
[ipc.topdown]
chain_head_delay = 10
proposal_delay = 10
max_proposal_range = 180
polling_interval = 30
exponential_back_off = 60
exponential_retry_limit = 5
parent_http_endpoint = "https://api.calibration.node.glif.io/rpc/v1"
parent_http_timeout = 120
parent_registry = "0x940f8cf09902b527e91105b6cfbaad7383216f4d"
parent_gateway = "0xd2d93eb6636b5268d9fbb8f71c4403c3415c139d"
```

And should NOT have any `[resolver.subnet.parent_gateway]` or `[resolver.connection.parent]` sections.

## Verification

### 1. Check Node Status
```bash
./ipc-manager status
```

All nodes should be running.

### 2. Check Relayer Status
```bash
./ipc-manager relayer-status
```

Should show the relayer running without errors.

### 3. Check Relayer Logs
```bash
ssh philip@34.73.187.192 "sudo journalctl -u ipc-relayer -n 50 --no-pager"
```

Should show checkpoint submissions without configuration errors.

## Summary

**Files Modified:**
- `scripts/ipc-subnet-manager/lib/config.sh` - Removed invalid resolver configuration paths

**Actions Required:**
1. ✅ Configuration fixed (already done)
2. ⚠️ Rebuild/redeploy `ipc-cli` binary to all validators
3. ⚠️ Re-run `./ipc-manager init --yes` to apply corrected config
4. ⚠️ Restart relayer with `./ipc-manager restart-relayer`

**Expected Result:**
- Nodes initialize without configuration errors
- Relayer starts successfully without missing argument errors
- Checkpoints are submitted to parent chain

