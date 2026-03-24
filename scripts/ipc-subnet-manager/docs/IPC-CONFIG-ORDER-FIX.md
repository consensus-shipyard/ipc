# IPC Config Order Fix

## Problem

When running `./ipc-manager init`, the following error occurred:

```
Error: parent subnet /r314159 not found in config store
```

This happened during `ipc-cli node init` execution.

## Root Cause

The IPC CLI configuration file (`~/.ipc/config.toml`) was being deployed **after** node initialization, but `ipc node init` requires the parent subnet configuration to already exist in the config store.

### Broken Order (Before)

```
1. Stop nodes
2. Backup data
3. Wipe node data
4. Initialize primary node        ← Runs `ipc node init` (needs parent config)
5. Extract peer info
6. Initialize secondary nodes
7. Collect peer info
8. Fix listen addresses
9. Update node configurations
10. Update IPC CLI configs         ← Creates ~/.ipc/config.toml (TOO LATE!)
11. Set federated power
12. Start nodes
```

**Problem:** Step 4 needs the config created in step 10!

### Fixed Order (After)

```
1. Stop nodes
2. Backup data
3. Wipe node data
4. Deploy IPC CLI Configuration    ← Creates ~/.ipc/config.toml FIRST
5. Initialize primary node         ← Now has parent config available
6. Extract peer info
7. Initialize secondary nodes
8. Collect peer info
9. Fix listen addresses
10. Update node configurations
11. Set federated power
12. Start nodes
```

**Solution:** Deploy IPC CLI config before any node initialization.

## Changes Made

### File: `ipc-subnet-manager.sh`

Moved the IPC CLI config deployment step to happen before node initialization:

```diff
# Wipe node data
log_section "Wiping Node Data"
wipe_all_nodes

+# Update IPC CLI configs (must be done BEFORE node init)
+log_section "Deploying IPC CLI Configuration"
+log_info "Creating ~/.ipc/config.toml with parent subnet configuration..."
+update_ipc_cli_configs
+
# Initialize primary node
log_section "Initializing Primary Node"
local primary_validator=$(get_primary_validator)
initialize_primary_node "$primary_validator"

...

# Update all configs with full mesh
log_section "Updating Node Configurations"
update_all_configs

-# Update IPC CLI configs
-log_section "Updating IPC CLI Configuration"
-update_ipc_cli_configs
-
# Set federated power
```

## Why This Fix Works

### What `ipc node init` Does

When you run `ipc-cli node init --config node-init.yml`, it:

1. Reads the node initialization config (`node-init.yml`)
2. **Looks up the parent subnet in `~/.ipc/config.toml`** to get:
   - Parent RPC endpoint
   - Parent registry contract address
   - Parent gateway contract address
3. Creates genesis from parent chain
4. Sets up the node directory structure

### What `~/.ipc/config.toml` Contains

The IPC CLI config file contains both parent and child subnet configurations:

```toml
keystore_path = "~/.ipc"

[[subnets]]
id = "/r314159"

[subnets.config]
network_type = "fevm"
provider_http = "https://api.calibration.node.glif.io/rpc/v1"
registry_addr = "0x51b66fb4f4b26c9cff772f3492ff6c2b205d1d46"
gateway_addr = "0x9a6740a1e23de7b9ebdf160b744546d2affc9e6e"

[[subnets]]
id = "/r314159/t410fgxd7f5t3up6ho5l6po7bfthuiaxib2olfoxeafq"

[subnets.config]
network_type = "fevm"
provider_http = "http://localhost:8545"
registry_addr = "0x74539671a1d2f1c8f200826baba665179f53a1b7"
gateway_addr = "0x77aa40b105843728088c0132e43fc44348881da8"
```

The first `[[subnets]]` entry is the **parent** subnet (`/r314159`), which is what `ipc node init` needs to look up.

## Configuration Requirements

For this to work, ensure your `ipc-subnet-config.yml` has:

### 1. Parent Subnet Configuration

```yaml
ipc_cli:
  parent:
    id: "/r314159"
    network_type: "fevm"
    provider_http: "https://api.calibration.node.glif.io/rpc/v1"
    registry_addr: "0x51b66fb4f4b26c9cff772f3492ff6c2b205d1d46"
    gateway_addr: "0x9a6740a1e23de7b9ebdf160b744546d2affc9e6e"
```

### 2. Child Subnet Configuration

```yaml
  child:
    network_type: "fevm"
    provider_http: "http://localhost:8545"
    gateway_addr: "0x77aa40b105843728088c0132e43fc44348881da8"
    registry_addr: "0x74539671a1d2f1c8f200826baba665179f53a1b7"
```

### 3. Subnet ID

```yaml
subnet:
  id: "/r314159/t410fgxd7f5t3up6ho5l6po7bfthuiaxib2olfoxeafq"
```

**Important:** All these addresses must match your actual deployed subnet on Calibration testnet.

## Testing

### 1. Clean slate initialization

```bash
./ipc-manager init --yes
```

You should see:

```
>>> Deploying IPC CLI Configuration
[INFO] Creating ~/.ipc/config.toml with parent subnet configuration...
[INFO] Updating IPC CLI configuration on all validators...
[SUCCESS] IPC CLI config updated for validator-1
[SUCCESS] IPC CLI config updated for validator-2
[SUCCESS] IPC CLI config updated for validator-3

>>> Initializing Primary Node
[INFO] Initializing validator-1 (primary)...
[INFO] Testing parent chain connectivity from validator-1...
[SUCCESS] Parent chain connectivity OK
[INFO] Running ipc-cli node init with verbose logging...
[INFO] Configuration validation completed
[INFO] Creating node directories under /home/ipc/.ipc-node
...
```

**No more "parent subnet not found" errors!**

### 2. Verify config on validator

```bash
# SSH to a validator
ssh philip@34.73.187.192
sudo su - ipc

# Check the config exists
cat ~/.ipc/config.toml

# Should show both parent and child subnets
```

### 3. Test IPC CLI commands

```bash
# On validator, test that parent subnet is accessible
ipc-cli subnet list --subnet /r314159

# Should work now!
```

## Related Files

- `ipc-subnet-manager.sh` - Main script with initialization flow
- `lib/config.sh` - Contains `generate_ipc_cli_config()` and `update_ipc_cli_configs()`
- `ipc-subnet-config.yml` - Configuration with parent and child subnet details

## Additional Notes

### Why Both Parent and Child in Config?

- **Parent**: Required by `ipc node init` to fetch genesis from parent chain
- **Child**: Used by IPC CLI commands to interact with the subnet itself

### When Config Is Used

1. **During init**: Parent config is read to create genesis
2. **After init**: Both configs are used by `ipc-cli` commands
3. **By relayer**: Parent and child configs are used for checkpoint submission

### Config Updates

If you need to update the IPC CLI config after initialization:

```bash
./ipc-manager update-config
```

This will regenerate and redeploy the config to all validators without reinitializing nodes.

## Troubleshooting

### If you still get "parent subnet not found"

1. **Check config file exists:**
   ```bash
   ssh philip@<validator-ip> sudo su - ipc -c "cat ~/.ipc/config.toml"
   ```

2. **Verify parent subnet entry:**
   Should contain `id = "/r314159"` (or your parent subnet ID)

3. **Check addresses match:**
   ```bash
   # Compare config.yml with deployed addresses on Calibration
   # Parent registry: 0x51b66fb4f4b26c9cff772f3492ff6c2b205d1d46
   # Parent gateway: 0x9a6740a1e23de7b9ebdf160b744546d2affc9e6e
   ```

4. **Test parent chain connectivity:**
   ```bash
   curl -X POST -H 'Content-Type: application/json' \
     --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
     https://api.calibration.node.glif.io/rpc/v1
   ```

### If parent addresses are wrong

Update `ipc-subnet-config.yml` with correct addresses from:
- Calibration testnet docs
- Your subnet deployment output
- Block explorer: https://calibration.filfox.info/

Then run `./ipc-manager init --yes` again.

## Success Criteria

After this fix, initialization should:

- ✅ Deploy IPC CLI config before node init
- ✅ Node init finds parent subnet in config store
- ✅ Genesis is created from parent chain
- ✅ All validators initialize successfully
- ✅ IPC CLI commands work on validators

## Files Modified

1. `ipc-subnet-manager.sh` - Reordered initialization steps

That's it! Single file change, big impact.

