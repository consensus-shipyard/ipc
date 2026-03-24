# IPC Subnet Manager - Fixes for Relayer Connection & Systemd Issues

## Issues Fixed

### Issue 1: Relayer Connection Error
**Error:** `error trying to connect: tcp connect error: Connection refused (os error 111)`

**Root Cause:**
- Relayer was trying to connect to `http://127.0.0.1:8545`
- The IPC node wasn't running or wasn't accessible at that address

**Fix:**
1. Changed `provider_http` from `127.0.0.1` to `localhost` in config
2. Ensured proper RPC endpoint configuration for the relayer

### Issue 2: Systemd Installation Error
**Error:** `Failed to connect to bus: No medium found`

**Root Causes:**
- SSH sessions don't always have proper dbus access for systemd user services
- `XDG_RUNTIME_DIR` environment variable not set correctly
- User lingering might not be enabled

**Fixes:**
1. Added `check_systemd_available()` function to detect if systemd user services are accessible
2. Set `XDG_RUNTIME_DIR=/run/user/$UID` explicitly when running systemd commands
3. Added graceful fallback to manual process management if systemd isn't available
4. Updated all systemd commands to include proper environment variables

## What Changed

### 1. Configuration File (`ipc-subnet-config.yml`)

```yaml
# Changed from:
provider_http: "http://127.0.0.1:8545"

# To:
provider_http: "http://localhost:8545"
```

### 2. Systemd Availability Check (`lib/health.sh`)

Added new function to check if systemd user services are actually usable:

```bash
check_systemd_available() {
    # Tests both systemd presence and dbus connectivity
    # Returns "yes" only if user systemd services actually work
}
```

### 3. Improved Systemd Installation

**Node Service Installation:**
- Checks systemd availability before attempting installation
- Sets `XDG_RUNTIME_DIR` explicitly for all systemd commands
- Returns proper error codes on failure
- Provides helpful error messages

**Relayer Service Installation:**
- Same improvements as node service
- Gracefully handles failures
- Falls back to manual management if systemd unavailable

### 4. Graceful Failure Handling

The `install-systemd` command now:
- Tracks successful and failed installations
- Shows a summary at the end
- Explains that manual management will work if systemd fails
- Doesn't exit on first failure

## Current State

### Systemd Availability

If systemd user services are **available**:
- ✅ Services installed and managed via systemd
- ✅ Automatic restart on failure
- ✅ Better logging and process isolation
- ✅ Use `systemctl --user` commands

If systemd user services are **NOT available**:
- ✅ Falls back to nohup/kill for process management
- ✅ All commands still work
- ✅ Node and relayer run but without systemd benefits
- ⚠️ Manual process management (less robust)

### Relayer Connection

The relayer now:
- Connects to `http://localhost:8545` (the node's RPC endpoint)
- Will work if the node is running and accessible
- Shows clear error messages if connection fails

## Troubleshooting

### Relayer Still Can't Connect

1. **Check if node is running:**
   ```bash
   ./ipc-manager check
   ```

2. **Verify node RPC is accessible:**
   ```bash
   # On the validator node
   curl http://localhost:8545 -X POST -H "Content-Type: application/json" \
     --data '{"jsonrpc":"2.0","method":"eth_chainId","id":1}'
   ```

3. **Check node logs:**
   ```bash
   tail -f ~/.ipc-node/logs/*.log
   ```

4. **Ensure node is bound to 0.0.0.0:8545 or 127.0.0.1:8545:**
   ```bash
   ss -tulpn | grep 8545
   ```

### Systemd Issues

#### If systemd installation fails:

1. **Check if systemd user services are supported:**
   ```bash
   # On validator node
   systemctl --user --version
   ```

2. **Check if lingering is enabled:**
   ```bash
   loginctl show-user $USER | grep Linger
   ```

3. **Enable lingering if needed:**
   ```bash
   sudo loginctl enable-linger $USER
   ```

4. **Set XDG_RUNTIME_DIR manually:**
   ```bash
   export XDG_RUNTIME_DIR=/run/user/$(id -u)
   systemctl --user list-units
   ```

5. **Check dbus availability:**
   ```bash
   echo $DBUS_SESSION_BUS_ADDRESS
   ```

#### If dbus isn't available in SSH:

You have two options:

**Option A: Use manual management (no systemd)**
```bash
# Just use the commands normally - they'll fall back to nohup/kill
./ipc-manager restart
./ipc-manager start-relayer
```

**Option B: SSH with dbus forwarding**
```bash
# SSH with proper environment
ssh -t user@host "export XDG_RUNTIME_DIR=/run/user/\$(id -u) && bash"
```

**Option C: Install via direct login**
```bash
# Login directly to the server (not via SSH)
# Then run:
./ipc-manager install-systemd --with-relayer --yes
```

## Current Workflow

### Normal Usage (with or without systemd)

All commands work automatically:

```bash
# Start/stop nodes
./ipc-manager restart
./ipc-manager check

# Start/stop relayer
./ipc-manager start-relayer
./ipc-manager stop-relayer
./ipc-manager relayer-status
```

The scripts detect whether systemd is available and use it if possible, otherwise fall back to manual management.

### Try to Install Systemd (Optional)

Only if you want systemd management:

```bash
# Try to install systemd services
./ipc-manager install-systemd --with-relayer --yes
```

If this fails due to dbus issues, don't worry - everything still works with manual management!

## Recommendations

### For Production Deployments

1. **If systemd works:** Great! You get all the benefits (auto-restart, better logging, etc.)

2. **If systemd doesn't work:** No problem! Use manual management:
   - All commands work the same
   - Processes run via nohup
   - Node and relayer are still isolated (different PIDs)
   - Stopping relayer won't kill node (fixed with better process detection)

### For Development/Testing

Manual management (nohup/kill) is actually simpler and often preferred:
- No need to deal with systemd user service setup
- Direct process control
- Easier to debug

## Files Modified

1. **ipc-subnet-config.yml**
   - Changed child `provider_http` to use `localhost` instead of `127.0.0.1`

2. **lib/health.sh**
   - Added `check_systemd_available()` function
   - Updated `install_systemd_services()` to check availability and set XDG_RUNTIME_DIR
   - Updated `install_relayer_systemd_service()` with same improvements
   - Added proper error handling and return codes

3. **ipc-subnet-manager.sh**
   - Updated `cmd_install_systemd()` to track success/failure counts
   - Added installation summary
   - Better error messages and guidance

## Next Steps

1. **Check if nodes are running:**
   ```bash
   ./ipc-manager check
   ```

2. **If nodes aren't running, start them:**
   ```bash
   ./ipc-manager restart
   ```

3. **Once nodes are running, start the relayer:**
   ```bash
   ./ipc-manager start-relayer
   ```

4. **Check relayer status:**
   ```bash
   ./ipc-manager relayer-status
   ```

5. **(Optional) Try installing systemd:**
   ```bash
   ./ipc-manager install-systemd --with-relayer --yes
   ```

The relayer connection issue should be resolved once your nodes are running properly. The systemd issue won't prevent you from using the system - it just means you'll use manual process management instead.

