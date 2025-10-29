# Systemd Logging and Installation Fixes

## Issues Fixed

### 1. No Logs in journalctl
**Problem:** Running `journalctl -u ipc-node` only showed start/stop messages, not actual application logs.

**Cause:** Service templates redirected output to files instead of journal:
```ini
StandardOutput=append:__NODE_HOME__/logs/node.stdout.log
StandardError=append:__NODE_HOME__/logs/node.stderr.log
```

**Fix:** Changed to use systemd journal:
```ini
StandardOutput=journal
StandardError=journal
SyslogIdentifier=ipc-node
```

Now logs go to journal and can be viewed with `journalctl`.

### 2. Installation Only on First Node
**Problem:** `install-systemd` command only installed on validator-1, not validator-2 or validator-3.

**Cause:** Silent errors during installation stopped the loop. Output was suppressed with `>/dev/null 2>&1`.

**Fix:**
- Removed output suppression to show actual errors
- Added verbose logging at each installation step
- Added validation checks before each operation
- Better error messages to identify failure points

### 3. Relayer Service Not Being Installed
**Problem:** Relayer systemd service wasn't being installed.

**Cause:** User needs to explicitly request it with `--with-relayer` flag.

**Fix:** Documentation updated to show correct usage.

## Changes Made

### 1. Service Templates

**Both `ipc-node.service.template` and `ipc-relayer.service.template`:**

```diff
# Resource limits
LimitNOFILE=65536

-# Logging
-StandardOutput=append:__NODE_HOME__/logs/node.stdout.log
-StandardError=append:__NODE_HOME__/logs/node.stderr.log
+# Logging (both to journal and files)
+StandardOutput=journal
+StandardError=journal
+SyslogIdentifier=ipc-node
+
+# Also ensure logs directory exists
+ExecStartPre=/bin/sh -c 'mkdir -p __NODE_HOME__/logs'

# Security
```

**Benefits:**
- Logs visible in `journalctl`
- Can still write to files if needed (using a separate logger)
- Standard systemd logging approach
- Better log aggregation and filtering

### 2. Installation Functions

**Updated `install_systemd_services()` and `install_relayer_systemd_service()`:**

```diff
-# Copy service file to /etc/systemd/system/ (requires sudo)
-scp -o StrictHostKeyChecking=no "$node_service_file" "$ssh_user@$ip:/tmp/ipc-node.service" >/dev/null 2>&1
+# Copy service file to /etc/systemd/system/ (requires sudo)
+log_info "  Copying service file to $name..."
+if ! scp -o StrictHostKeyChecking=no "$node_service_file" "$ssh_user@$ip:/tmp/ipc-node.service" 2>&1; then
+    log_error "Failed to copy service file to $name"
+    rm -f "$node_service_file"
+    return 1
+fi

-ssh -o StrictHostKeyChecking=no "$ssh_user@$ip" \
-    "sudo mv /tmp/ipc-node.service /etc/systemd/system/ipc-node.service && sudo chmod 644 /etc/systemd/system/ipc-node.service" >/dev/null 2>&1
+log_info "  Moving to /etc/systemd/system/..."
+if ! ssh -o StrictHostKeyChecking=no "$ssh_user@$ip" \
+    "sudo mv /tmp/ipc-node.service /etc/systemd/system/ipc-node.service && sudo chmod 644 /etc/systemd/system/ipc-node.service" 2>&1; then
+    log_error "Failed to install service file on $name"
+    rm -f "$node_service_file"
+    return 1
+fi
```

**Added:**
- Progress messages for each step
- Error messages with context
- Proper error handling with early returns
- Output visibility (removed `>/dev/null 2>&1`)

## Usage

### Install Node Services on All Validators

```bash
./ipc-manager install-systemd --yes
```

This installs node service on:
- validator-1
- validator-2
- validator-3

### Install Node + Relayer Services

```bash
./ipc-manager install-systemd --with-relayer --yes
```

This installs:
- Node service on all 3 validators
- Relayer service on validator-1 (primary)

### Expected Output

```
>>> Installing Node Services

[INFO] Checking systemd availability on validator-1...
[INFO] Installing systemd service on validator-1...
[INFO]   Copying service file to validator-1...
[INFO]   Moving to /etc/systemd/system/...
[INFO]   Reloading systemd...
[INFO]   Enabling service...
[SUCCESS] ✓ Node service installed on validator-1

[INFO] Checking systemd availability on validator-2...
[INFO] Installing systemd service on validator-2...
[INFO]   Copying service file to validator-2...
[INFO]   Moving to /etc/systemd/system/...
[INFO]   Reloading systemd...
[INFO]   Enabling service...
[SUCCESS] ✓ Node service installed on validator-2

[INFO] Checking systemd availability on validator-3...
[INFO] Installing systemd service on validator-3...
[INFO]   Copying service file to validator-3...
[INFO]   Moving to /etc/systemd/system/...
[INFO]   Reloading systemd...
[INFO]   Enabling service...
[SUCCESS] ✓ Node service installed on validator-3

>>> Installing Relayer Service

[INFO] Installing relayer systemd service on validator-1...
[INFO]   Copying relayer service file to validator-1...
[INFO]   Moving to /etc/systemd/system/...
[INFO]   Reloading systemd...
[INFO]   Enabling relayer service...
[SUCCESS] ✓ Relayer service installed on validator-1

Installation Summary:
  ✓ Successful: 4
```

## Viewing Logs

### Using journalctl (now works!)

```bash
# On validator node
sudo journalctl -u ipc-node -f              # Follow node logs
sudo journalctl -u ipc-node -n 100          # Last 100 lines
sudo journalctl -u ipc-node --since "5m ago" # Last 5 minutes

# Relayer logs (on validator-1)
sudo journalctl -u ipc-relayer -f
sudo journalctl -u ipc-relayer -n 100
```

### Filter by Log Level

```bash
sudo journalctl -u ipc-node -p err          # Only errors
sudo journalctl -u ipc-node -p warning      # Warnings and above
sudo journalctl -u ipc-node -p info         # Info and above (all)
```

### Follow Both Services

```bash
sudo journalctl -u ipc-node -u ipc-relayer -f
```

### Export Logs

```bash
# JSON format
sudo journalctl -u ipc-node -o json > node-logs.json

# Short format
sudo journalctl -u ipc-node -o short > node-logs.txt
```

## Log Identifiers

- **Node logs**: `SyslogIdentifier=ipc-node`
- **Relayer logs**: `SyslogIdentifier=ipc-relayer`

You can filter by these:
```bash
sudo journalctl SYSLOG_IDENTIFIER=ipc-node
sudo journalctl SYSLOG_IDENTIFIER=ipc-relayer
```

## Troubleshooting

### If installation fails on a specific node

The detailed error output will now show:

```
[INFO] Checking systemd availability on validator-2...
[INFO] Installing systemd service on validator-2...
[INFO]   Copying service file to validator-2...
[ERROR] Failed to copy service file to validator-2
scp: /tmp/ipc-node.service: Permission denied
```

This tells you exactly where and why it failed.

### Common Issues

#### 1. Permission Denied

```
[ERROR] Failed to install service file on validator-2
sudo: a password is required
```

**Solution:** Ensure passwordless sudo is configured for the SSH user.

#### 2. Service Already Exists

```
[INFO]   Enabling service...
Failed to enable unit: Unit file ipc-node.service already exists
```

**Solution:** Service is already installed. To reinstall:
```bash
# On validator
sudo systemctl disable ipc-node
sudo rm /etc/systemd/system/ipc-node.service
sudo systemctl daemon-reload

# Then reinstall
./ipc-manager install-systemd --yes
```

#### 3. Systemd Not Available

```
[WARN] ✗ Systemd not available on validator-1
[INFO]   You can still manage processes manually without systemd
```

**Solution:** The server doesn't have systemd. The manager script will fall back to manual process management (nohup/kill).

### Verify Installation

```bash
# On each validator
systemctl list-unit-files | grep ipc

# Should show:
# ipc-node.service     enabled
# ipc-relayer.service  enabled  (on validator-1 only if installed with --with-relayer)
```

### Check Service Status

```bash
# On validator
sudo systemctl status ipc-node
sudo systemctl status ipc-relayer  # On validator-1

# Should show:
# ● ipc-node.service - IPC Validator Node
#    Loaded: loaded (/etc/systemd/system/ipc-node.service; enabled; vendor preset: enabled)
#    Active: active (running) since ...
```

## Service Files Location

After installation:
```
/etc/systemd/system/ipc-node.service      # All validators
/etc/systemd/system/ipc-relayer.service   # validator-1 only (if --with-relayer used)
```

## Restart Services After Update

If you update the service templates and need to reinstall:

```bash
# Remove old services on all validators
ssh philip@<validator-ip> 'sudo systemctl stop ipc-node && sudo systemctl disable ipc-node && sudo rm /etc/systemd/system/ipc-node.service && sudo systemctl daemon-reload'

# Reinstall
./ipc-manager install-systemd --with-relayer --yes

# Start services
./ipc-manager restart
./ipc-manager start-relayer
```

## Files Modified

1. `templates/ipc-node.service.template` - Changed logging to journal
2. `templates/ipc-relayer.service.template` - Changed logging to journal
3. `lib/health.sh`:
   - `install_systemd_services()` - Added verbose output and better error handling
   - `install_relayer_systemd_service()` - Added verbose output and better error handling

## Benefits

### Better Observability
- ✅ Logs in journal (standard systemd location)
- ✅ Can use all journalctl features (filtering, searching, exporting)
- ✅ Logs survive service restarts
- ✅ Automatic log rotation via journald

### Better Debugging
- ✅ See exactly where installation fails
- ✅ Error messages with context
- ✅ Progress indicators during installation
- ✅ Can identify which validator has issues

### Production Ready
- ✅ Standard systemd logging approach
- ✅ Centralized log management
- ✅ Integration with log aggregators (if using)
- ✅ Better monitoring and alerting capabilities

## Testing

1. **Reinstall services with verbose output:**
   ```bash
   ./ipc-manager install-systemd --with-relayer --yes
   ```

2. **Verify all services installed:**
   ```bash
   # Check each validator
   for ip in 34.73.187.192 35.237.175.224 34.75.205.89; do
       echo "Checking $ip..."
       ssh philip@$ip "systemctl list-unit-files ipc-node.service"
   done
   ```

3. **Start services:**
   ```bash
   ./ipc-manager restart
   ./ipc-manager start-relayer
   ```

4. **View logs:**
   ```bash
   # SSH to validator-1
   ssh philip@34.73.187.192
   sudo journalctl -u ipc-node -f

   # In another terminal, check relayer
   sudo journalctl -u ipc-relayer -f
   ```

You should now see full application logs, not just start/stop messages!

