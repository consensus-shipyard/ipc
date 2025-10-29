# Fix: Config Update Quoting Issues

## Problem
The `ipc-subnet-manager` script's `update-config` command was failing to properly update validator node configurations. Specifically:

1. **CometBFT `persistent_peers`** - Not being set
2. **Fendermint `static_addresses`** - Being set but without quotes around multiaddrs
3. **Fendermint `external_addresses`** - Being set correctly

## Root Causes

### 1. Quote Escaping Through SSH
The main issue was improper quote escaping when passing sed commands through `ssh_exec()`, which wraps commands in `sudo su - ipc_user -c '$cmd'`.

**Problem Code:**
```bash
ssh_exec "$ip" "$ssh_user" "$ipc_user" \
    "sed -i.bak 's|^persistent_peers = .*|persistent_peers = \"$comet_peers\"|' $node_home/..."
```

When passed through `ssh_exec`, this becomes:
```bash
sudo su - ipc -c 'sed -i.bak 's|...|...|' /path/...'
```

The nested single quotes break the quoting, causing syntax errors.

### 2. Missing Variable Definition
The `$name` variable was not defined in `update_validator_config()`, causing the function to fail silently after the first log message.

### 3. Arithmetic Operation Exit
The `((peer_count++))` arithmetic operation was causing script exit when `set -e` was enabled and the operation returned non-zero.

## Solutions

### 1. Fixed Quote Escaping for CometBFT
Changed from single quotes to double quotes with escaped inner quotes:

```bash
# Before (BROKEN):
"sed -i.bak 's|^persistent_peers = .*|persistent_peers = \"$comet_peers\"|' ..."

# After (FIXED):
"sed -i.bak \"s|^persistent_peers = .*|persistent_peers = \\\"$comet_peers\\\"|\" ..."
```

### 2. Fixed Quote Escaping for Fendermint static_addresses
This required a multi-step approach:

1. Build peer list WITHOUT quotes: `/ip4/.../p2p/..., /ip4/.../p2p/...`
2. Add quotes locally using sed: `"/ip4/.../p2p/...", "/ip4/.../p2p/..."`
3. Escape quotes for ssh transmission: `\"/ip4/...\", \"/ip4/...\"`

```bash
# Build list without quotes
libp2p_static_addrs+="${LIBP2P_PEERS[$peer_idx]}, "

# Add quotes around each multiaddr
local quoted_addrs=$(echo "$libp2p_static_addrs" | sed 's|/ip4/|"/ip4/|g' | sed 's|, |", |g')
quoted_addrs="${quoted_addrs}\""  # Add trailing quote

# Escape quotes for ssh_exec
local escaped_addrs="${quoted_addrs//\"/\\\"}"

# Pass to remote sed
ssh_exec ... "sed ... s|^static_addresses = .*|static_addresses = [$escaped_addrs]|"
```

### 3. Fixed Missing Variable
Added `local name="${VALIDATORS[$validator_idx]}"` at the start of `update_validator_config()`.

### 4. Fixed Arithmetic Operation
Changed from `((peer_count++))` to `peer_count=$((peer_count + 1))` which doesn't cause exit on error.

## Files Modified

- `lib/config.sh`:
  - `update_validator_config()` - Fixed quote escaping in all sed commands
  - `update_all_configs()` - Fixed arithmetic operation
  - `collect_all_peer_info()` - Used `jq` for JSON parsing instead of `sed`/`grep`

- `lib/health.sh`:
  - `start_validator_node()` - Added missing `--home` parameter
  - `check_validator()` - Fixed quote escaping in grep patterns

- `lib/ssh.sh`:
  - `ssh_check_process()` - Fixed pgrep command to use if/then/else instead of &&/||
  - `ssh_kill_process()` - Made more robust with proper error handling

## Verification

After fixes, all three validators now have:

✅ **CometBFT persistent_peers**: Correctly set with comma-separated peer list
```
persistent_peers = "node_id1@ip1:port1,node_id2@ip2:port2"
```

✅ **Fendermint static_addresses**: Correctly set with quoted multiaddrs
```
static_addresses = ["/ip4/ip1/tcp/port1/p2p/peer_id1", "/ip4/ip2/tcp/port2/p2p/peer_id2"]
```

✅ **Fendermint external_addresses**: Correctly set with quoted multiaddr
```
external_addresses = ["/ip4/own_ip/tcp/own_port/p2p/own_peer_id"]
```

## Testing

Run the full update-config command:
```bash
./ipc-manager update-config
```

Verify configs on each validator:
```bash
# CometBFT
grep "^persistent_peers" ~/.ipc-node/cometbft/config/config.toml

# Fendermint
grep "static_addresses\|external_addresses" ~/.ipc-node/fendermint/config/default.toml
```

## Lessons Learned

1. **Quote Escaping is Tricky**: When passing commands through multiple layers (bash → ssh → sudo → bash), quote escaping requires careful attention to how each layer interprets quotes.

2. **Use jq for JSON**: Parsing JSON with `sed`/`grep` is error-prone. Using `jq` is more reliable, even through SSH.

3. **Test with Debug Output**: Adding debug output helped identify where the script was failing and what values variables contained at each step.

4. **Avoid Nested Single Quotes**: When using `ssh_exec` which wraps commands in single quotes, use double quotes in the command string and escape inner quotes with backslashes.

5. **Process Substitution**: For complex string transformations, it's often easier to do them locally before passing to remote commands rather than trying to do everything in one remote sed command.

---

**Date**: October 17, 2025
**Status**: ✅ Fixed and verified

