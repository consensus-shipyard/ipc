# Verification Guide for Local Mode Fix

## Quick Test

To verify the fix works, run these commands:

```bash
cd /Users/philip/github/ipc/scripts/ipc-subnet-manager

# Test the info command in local mode
./ipc-manager info
```

## What to Expect

### Before the Fix
- The command would attempt to SSH to localhost
- You'd see connection attempts or hangs
- Commands might timeout or fail with SSH errors

### After the Fix
- The command executes immediately without SSH
- All information is fetched from local processes
- No SSH connection attempts or errors

## Debugging

If you encounter issues, check:

1. **Verify local mode is set:**
```bash
grep "deployment_mode" ipc-subnet-config-local.yml
# Should show: deployment_mode: local
```

2. **Check if nodes are running:**
```bash
pgrep -f "ipc-cli node start"
# Should return process IDs if nodes are running
```

3. **Test exec_on_host function:**
```bash
# Add this test command temporarily
./ipc-manager info 2>&1 | head -20
# Look for any SSH-related errors
```

## Other Commands That May Need Similar Fixes

The following commands in `health.sh` also use SSH directly and may need similar fixes for full local mode support:

- `check` - Uses `check_validator_health()` which calls `ssh_exec`
- `block-time` - Uses `measure_block_time()` which calls `ssh_exec`
- `watch-finality` - Uses `watch_parent_finality()` which calls `ssh_exec`
- `watch-blocks` - Uses `watch_block_production()` which calls `ssh_exec`
- `consensus-status` - Uses `show_consensus_status()` which calls `ssh_exec`
- `voting-status` - Uses `show_voting_status()` which calls `ssh_exec`

If you use these commands in local mode and encounter SSH issues, they will need similar fixes.

## Implementation Pattern

The fix follows this pattern:

**Old pattern (remote-only):**
```bash
local ip=$(get_config_value "validators[$idx].ip")
local ssh_user=$(get_config_value "validators[$idx].ssh_user")
local ipc_user=$(get_config_value "validators[$idx].ipc_user")
local result=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" "command")
```

**New pattern (local + remote):**
```bash
local result=$(exec_on_host "$idx" "command")
```

The `exec_on_host` function (in `lib/exec.sh`) automatically:
- Checks `is_local_mode()`
- Calls `local_exec()` if local
- Calls `ssh_exec()` if remote
