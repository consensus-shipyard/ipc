# Systemd Installation Fix

## Issues Fixed

### 1. Installation Only on First Validator
**Problem:** `install-systemd` command only installed on validator-1, then exited.

**Root Cause:** The arithmetic expansion `((success_count++))` returns 0 when incrementing from 0 to 1. With `set -euo pipefail` in the main script, any command returning 0 (false) causes immediate exit.

**Fix:** Changed from `((success_count++))` to `success_count=$((success_count + 1))`, which always returns the new value (never 0).

### 2. Relayer Service Not Being Installed
**Problem:** Relayer service wasn't being installed even with `--with-relayer` flag.

**Root Cause:** Same arithmetic expansion issue prevented script from reaching the relayer installation step.

**Fix:** Same as above - the script now runs all installation steps successfully.

### 3. Missing SCRIPT_DIR in Template Generation
**Problem:** `generate_node_systemd_service()` and `generate_relayer_systemd_service()` functions couldn't find template files.

**Root Cause:** `SCRIPT_DIR` environment variable wasn't set when functions were called outside the main script context.

**Fix:** Added SCRIPT_DIR initialization in both functions:
```bash
if [ -z "$SCRIPT_DIR" ]; then
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fi
```

### 4. Service Masked on validator-2
**Problem:** Service was masked, preventing enablement.

**Fix:** Ran `sudo systemctl unmask ipc-node` on affected validators before installation.

## Changes Made

### File: `ipc-subnet-manager.sh`

```diff
for idx in "${!VALIDATORS[@]}"; do
    if install_systemd_services "$idx"; then
-       ((success_count++))
+       success_count=$((success_count + 1))
    else
-       ((fail_count++))
+       fail_count=$((fail_count + 1))
    fi
done

# Install relayer service on primary validator
if [ "$install_relayer" = true ]; then
    if ! install_relayer_systemd_service "$primary_idx"; then
-       ((fail_count++))
+       fail_count=$((fail_count + 1))
    else
-       ((success_count++))
+       success_count=$((success_count + 1))
    fi
fi
```

### File: `lib/health.sh`

**Added SCRIPT_DIR initialization in both functions:**

```bash
# Generate systemd service file for node
generate_node_systemd_service() {
    local validator_idx="$1"
    local output_file="$2"

    local ipc_user=$(get_config_value "validators[$validator_idx].ipc_user")
    local ipc_binary=$(get_config_value "paths.ipc_binary")
    local node_home=$(get_config_value "paths.node_home")

    # Ensure SCRIPT_DIR is set
    if [ -z "$SCRIPT_DIR" ]; then
        SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
    fi

    sed -e "s|__IPC_USER__|$ipc_user|g" \
        -e "s|__IPC_BINARY__|$ipc_binary|g" \
        -e "s|__NODE_HOME__|$node_home|g" \
        "${SCRIPT_DIR}/templates/ipc-node.service.template" > "$output_file"
}

# Generate systemd service file for relayer
generate_relayer_systemd_service() {
    local validator_idx="$1"
    local output_file="$2"

    # ... variable setup ...

    # Ensure SCRIPT_DIR is set
    if [ -z "$SCRIPT_DIR" ]; then
        SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
    fi

    sed -e "s|__IPC_USER__|$ipc_user|g" \
        # ... other replacements ...
        "${SCRIPT_DIR}/templates/ipc-relayer.service.template" > "$output_file"
}
```

## Why This Matters

### About `set -euo pipefail`

The main script uses `set -euo pipefail` for safety:
- `-e`: Exit if any command returns non-zero
- `-u`: Exit if using undefined variables
- `-o pipefail`: Exit if any command in a pipeline fails

### The Arithmetic Expansion Bug

In Bash, arithmetic expressions return their result value:
- `((0))` returns 0 (false) → causes `set -e` to exit
- `((1))` returns 1 (true) → continues
- `((2))` returns 2 (true) → continues

When we do `((success_count++))`:
- If `success_count` is 0, it increments to 1, then returns the OLD value (0)
- Return value 0 triggers `set -e` to exit the script

Using `success_count=$((success_count + 1))` instead:
- The expression returns the new value (1, 2, 3, etc.)
- Assignment always succeeds
- Never triggers `set -e`

## Testing

### Success Case

```bash
cd /Users/philip/github/ipc/scripts/ipc-subnet-manager
./ipc-manager install-systemd --with-relayer --yes
```

**Expected output:**
```
>>> Installing Node Services
[SUCCESS] ✓ Node service installed on validator-1
[SUCCESS] ✓ Node service installed on validator-2
[SUCCESS] ✓ Node service installed on validator-3

>>> Installing Relayer Service
[SUCCESS] ✓ Relayer service installed on validator-1

Installation Summary:
  ✓ Successful: 4
```

### Verification

1. **Check all services are installed:**
   ```bash
   for ip in 34.73.187.192 35.237.175.224 34.75.205.89; do
       echo "=== Checking $ip ==="
       ssh philip@$ip "systemctl list-unit-files | grep ipc"
   done
   ```

2. **Check relayer service on validator-1:**
   ```bash
   ssh philip@34.73.187.192 "ls -la /etc/systemd/system/ipc-*"
   # Should show both ipc-node.service and ipc-relayer.service
   ```

3. **View logs:**
   ```bash
   ssh philip@34.73.187.192 "sudo journalctl -u ipc-node -n 20"
   ssh philip@34.73.187.192 "sudo journalctl -u ipc-relayer -n 20"
   ```

## Files Modified

1. `ipc-subnet-manager.sh` - Fixed arithmetic expansions
2. `lib/health.sh` - Added SCRIPT_DIR initialization in template generation functions

## Related Documentation

- `SYSTEMD-LOGGING-FIX.md` - Logging improvements
- `SYSTEMD-SYSTEM-SERVICE-UPDATE.md` - System vs user services
- `SYSTEMD-TARGET-FIX.md` - Target configuration

## Success Criteria

After this fix:
- ✅ All 3 validators get node service installed
- ✅ Relayer service installs on validator-1
- ✅ Installation summary shows 4 successful installations
- ✅ No early script exit due to arithmetic expressions
- ✅ Template files are found and processed correctly

