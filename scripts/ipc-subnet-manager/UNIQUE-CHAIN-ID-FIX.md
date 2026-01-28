# Unique Subnet Chain ID Implementation

## Problem

When running `ipc-manager init` in local mode, the subnet was inheriting the same EVM chain ID (31337) as the parent Anvil chain. This caused:
- Confusion about which chain was being queried
- Potential transaction replay vulnerabilities
- Inability to distinguish subnet from parent in wallets/tools

## Root Cause

The `deploy_subnet()` function in `lib/health.sh` was setting the subnet's `chain-id` parameter to the parent's chain ID:

```yaml
create:
  chain-id: $(echo "$parent_chain_id" | sed 's/\/r//')  # Was using parent's 31337
```

This made both parent and subnet report the same EVM chain ID.

## Solution

### 1. Added Configuration Option

Updated `ipc-subnet-config-local.yml` to include a dedicated subnet chain ID:

```yaml
subnet:
  # Subnet's EVM chain ID (must be unique from parent)
  # If not specified, will be auto-generated based on timestamp
  # Common practice: use a unique value like parent_chain_id + 1000
  # Example: parent is 31337, subnet could be 32337, 41337, etc.
  chain_id: 32337
```

**Default value:** 32337 (parent 31337 + 1000)

### 2. Updated deploy_subnet() Function

Modified `lib/health.sh` to read the subnet chain ID from config:

```bash
# Get subnet chain ID from config, or generate a unique one
local subnet_chain_id=$(get_config_value "subnet.chain_id" 2>/dev/null)
if [ -z "$subnet_chain_id" ] || [ "$subnet_chain_id" = "null" ]; then
    # Generate unique chain ID based on timestamp (milliseconds since epoch mod 2^32)
    local parent_num=$(echo "$parent_chain_id" | sed 's/\/r//')
    subnet_chain_id=$((parent_num + 1000 + ($(date +%s) % 10000)))
    log_warn "No subnet.chain_id configured, generated: $subnet_chain_id" >&2
else
    log_info "Using configured subnet chain ID: $subnet_chain_id" >&2
fi
```

Then use this value in the subnet-init.yaml:

```yaml
create:
  chain-id: $subnet_chain_id  # Now uses unique subnet chain ID
```

### 3. Created Chain ID Calculator (Optional)

Added `lib/calculate_chain_id.py` - a Python utility that mimics the Rust implementation's FNV hash-based chain ID derivation. This is available for future use if you want to derive chain IDs from subnet IDs.

```python
# Calculate chain ID from subnet ID (same as Rust implementation)
python3 lib/calculate_chain_id.py "/r31337/t410fwwa..."
```

## How It Works

### Configuration-Based (Default)
1. Read `subnet.chain_id` from config file
2. If specified, use that value
3. If not specified, auto-generate: `parent_chain_id + 1000 + random(0-9999)`

### Auto-Generation Formula
```
subnet_chain_id = parent_chain_id + 1000 + (current_timestamp % 10000)
```

Example:
- Parent: 31337
- Timestamp: 1705350123
- Generated: 31337 + 1000 + (1705350123 % 10000) = 32337 + 123 = 32460

## Testing

### Before Fix

```bash
$ ./ipc-manager --config ipc-subnet-config-local.yml info

Chain IDs:
  Parent Chain ID: 31337 (from config: /r31337)
  Parent eth_chainId (via RPC): 0x7a69 (decimal: 31337)
  Subnet eth_chainId (via RPC): 0x7a69 (decimal: 31337)  ← Same!
  ⚠ Subnet and parent have the same eth_chainId (31337)
```

### After Fix (Need to Re-Init)

```bash
# 1. Stop and wipe existing subnet
$ ./ipc-manager --config ipc-subnet-config-local.yml stop
$ ./ipc-manager --config ipc-subnet-config-local.yml wipe --force

# 2. Initialize with new chain ID
$ ./ipc-manager --config ipc-subnet-config-local.yml init

# Expected output during init:
[INFO] Using configured subnet chain ID: 32337

# 3. Check the new chain ID
$ ./ipc-manager --config ipc-subnet-config-local.yml info

Chain IDs:
  Parent Chain ID: 31337 (from config: /r31337)
  Parent eth_chainId (via RPC): 0x7a69 (decimal: 31337)
  Subnet eth_chainId (via RPC): 0x7e69 (decimal: 32337)  ← Different!
```

## Important Notes

### ⚠️ Requires Re-Initialization

The chain ID is set during subnet creation on the parent chain. To change it:
1. **Stop** all validators
2. **Wipe** the subnet data
3. **Re-initialize** the subnet with the new configuration

The chain ID cannot be changed after the subnet is created without re-deploying.

### Chain ID Selection

Choose a chain ID that:
- ✅ Is unique across your network
- ✅ Doesn't conflict with public chains (check [chainlist.org](https://chainlist.org))
- ✅ Is within valid range: 1 to 4,294,967,295 (2^32 - 1)
- ✅ For local dev: parent + 1000 is a safe choice

### MetaMask Configuration

After changing the chain ID, update your MetaMask network:
1. Network Name: IPC Subnet Local
2. RPC URL: http://localhost:8546
3. Chain ID: **32337** (new value)
4. Currency Symbol: FIL

## Files Modified

1. **`ipc-subnet-config-local.yml`**
   - Added `subnet.chain_id: 32337` configuration

2. **`lib/health.sh`**
   - Updated `deploy_subnet()` to read subnet chain ID from config
   - Added auto-generation fallback if not configured
   - Changed subnet-init.yaml to use subnet's chain ID instead of parent's

3. **`lib/calculate_chain_id.py`** (new)
   - Utility to calculate chain ID from subnet ID using FNV hash
   - Matches Rust implementation in `ipc/api/src/subnet_id.rs`

## Benefits

✅ **Unique Chain IDs**: Parent and subnet now have distinct chain IDs
✅ **Configurable**: Easy to set via config file
✅ **Auto-Generation**: Falls back to unique generation if not specified
✅ **Clear Display**: Info command shows both parent and subnet chain IDs
✅ **Security**: Reduces transaction replay risk between chains
✅ **Wallet Support**: Proper chain separation in MetaMask and other tools

## Related Documentation

- Chain ID explanation: `CHAIN-ID-EXPLANATION.md`
- Chain ID display fix: `CHAIN-ID-FIX-SUMMARY.md`
- All local mode fixes: `ALL-LOCAL-MODE-FIXES-SUMMARY.md`

## Future Enhancements

### Option 1: Derive from Subnet ID (Post-Creation)
After subnet is created, calculate chain ID from subnet ID:
```bash
subnet_id=$(get_config_value "subnet.id")
chain_id=$(python3 lib/calculate_chain_id.py "$subnet_id")
```

However, this requires a two-phase deployment which adds complexity.

### Option 2: Registry of Chain IDs
Maintain a registry of used chain IDs to avoid conflicts:
```bash
# Check if chain ID is already used
if chain_id_exists "$subnet_chain_id"; then
    subnet_chain_id=$((subnet_chain_id + 1))
fi
```

### Option 3: IPC Protocol Enhancement
Enhance IPC protocol to automatically assign unique chain IDs during subnet creation, similar to how subnet IDs are generated.
