# Systemd System Service Update

## What Changed

Converted from **user systemd services** to **system systemd services** for better reliability and easier management.

### Before (User Services)
- **Location**: `~/.config/systemd/user/`
- **Commands**: `systemctl --user start ipc-node`
- **Issues**:
  - Required `XDG_RUNTIME_DIR` environment variable
  - SSH sessions often couldn't access dbus
  - Needed user lingering enabled
  - "Failed to connect to bus: No medium found" errors

### After (System Services)
- **Location**: `/etc/systemd/system/`
- **Commands**: `sudo systemctl start ipc-node`
- **Benefits**:
  - Works reliably via SSH
  - No dbus or environment variable issues
  - Standard system service management
  - Services run as specified `User=ipc` in the service file

## Changes Made

### 1. Service Templates

**Both `ipc-node.service.template` and `ipc-relayer.service.template`:**

```diff
[Service]
Type=simple
+User=__IPC_USER__
WorkingDirectory=__NODE_HOME__
...

[Install]
-WantedBy=default.target
+WantedBy=multi-user.target
```

- **Added back** `User=__IPC_USER__` directive (required for system services to run as non-root)
- **Changed** `WantedBy=multi-user.target` (correct for system services)

### 2. Installation Functions

**`install_systemd_services()` and `install_relayer_systemd_service()`:**

```diff
-# Create systemd user directory
-ssh_exec "$ip" "$ssh_user" "$ipc_user" "mkdir -p ~/.config/systemd/user"
-
-# Copy service file
-scp_to_host "$ip" "$ssh_user" "$ipc_user" \
-    "$service_file" \
-    "/home/$ipc_user/.config/systemd/user/ipc-node.service"
+# Copy service file to /etc/systemd/system/ (requires sudo)
+scp "$service_file" "$ssh_user@$ip:/tmp/ipc-node.service"
+ssh "$ssh_user@$ip" "sudo mv /tmp/ipc-node.service /etc/systemd/system/"

-# Reload systemd
-ssh_exec "$ip" "$ssh_user" "$ipc_user" \
-    "export XDG_RUNTIME_DIR=/run/user/$uid && systemctl --user daemon-reload"
+# Reload systemd
+ssh "$ssh_user@$ip" "sudo systemctl daemon-reload"

-# Enable service
-ssh_exec "$ip" "$ssh_user" "$ipc_user" \
-    "export XDG_RUNTIME_DIR=/run/user/$uid && systemctl --user enable ipc-node.service"
+# Enable service
+ssh "$ssh_user@$ip" "sudo systemctl enable ipc-node.service"
```

- Copy to `/etc/systemd/system/` instead of `~/.config/systemd/user/`
- Use `sudo systemctl` instead of `systemctl --user`
- No need for `XDG_RUNTIME_DIR` or user lingering
- Simplified systemd availability check

### 3. Service Management Functions

**Updated `start_validator_node()`, `stop_all_nodes()`, `start_relayer()`, `stop_relayer()`, `check_relayer_status()`:**

```diff
-# Check if service exists
-local has_systemd=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
-    "systemctl --user list-unit-files ipc-node.service ..." )
+# Check if service exists
+local has_systemd=$(ssh "$ssh_user@$ip" \
+    "systemctl list-unit-files ipc-node.service ..." )

-# Start service
-ssh_exec "$ip" "$ssh_user" "$ipc_user" "systemctl --user start ipc-node"
+# Start service
+ssh "$ssh_user@$ip" "sudo systemctl start ipc-node"

-# Check status
-systemctl --user is-active ipc-node
+# Check status
+systemctl is-active ipc-node

-# View logs
-journalctl --user -u ipc-relayer -f
+# View logs
+sudo journalctl -u ipc-relayer -f
```

All systemd commands now use `sudo systemctl` instead of `systemctl --user`.

## Installation

### Prerequisites

The `ssh_user` must have passwordless sudo access for systemctl commands. Add to `/etc/sudoers` or `/etc/sudoers.d/ipc`:

```bash
# Allow ssh_user to manage IPC services without password
root    ALL=(ALL) NOPASSWD: /bin/systemctl start ipc-node, /bin/systemctl stop ipc-node, /bin/systemctl restart ipc-node, /bin/systemctl status ipc-node
root    ALL=(ALL) NOPASSWD: /bin/systemctl start ipc-relayer, /bin/systemctl stop ipc-relayer, /bin/systemctl restart ipc-relayer, /bin/systemctl status ipc-relayer
root    ALL=(ALL) NOPASSWD: /bin/systemctl daemon-reload, /bin/systemctl enable ipc-node, /bin/systemctl enable ipc-relayer
root    ALL=(ALL) NOPASSWD: /bin/journalctl
```

Or for full systemctl access:
```bash
root    ALL=(ALL) NOPASSWD: /bin/systemctl
```

### Install Services

```bash
# Install node services on all validators + relayer on primary
./ipc-manager install-systemd --with-relayer --yes
```

This will:
1. Check systemd availability on each validator
2. Generate service files from templates
3. Copy to `/etc/systemd/system/` on each validator
4. Reload systemd and enable services
5. Report success/failure for each validator

## Usage

### Direct Systemd Commands (on validator hosts)

```bash
# Node service
sudo systemctl start ipc-node
sudo systemctl stop ipc-node
sudo systemctl restart ipc-node
sudo systemctl status ipc-node

# Relayer service (primary validator only)
sudo systemctl start ipc-relayer
sudo systemctl stop ipc-relayer
sudo systemctl status ipc-relayer

# View logs
sudo journalctl -u ipc-node -f
sudo journalctl -u ipc-relayer -f

# Enable/disable auto-start
sudo systemctl enable ipc-node
sudo systemctl disable ipc-node
```

### Manager Commands (from management machine)

The manager script auto-detects systemd and uses it if available:

```bash
# Start all nodes
./ipc-manager restart

# Start relayer
./ipc-manager start-relayer

# Stop relayer
./ipc-manager stop-relayer

# Check relayer status
./ipc-manager relayer-status

# Check overall health
./ipc-manager check
```

## Service Files Location

```
/etc/systemd/system/ipc-node.service      # Node service
/etc/systemd/system/ipc-relayer.service   # Relayer service (primary only)
```

## Logs Location

Logs are written to both:
1. **Systemd journal**: `sudo journalctl -u ipc-node -f`
2. **Log files**:
   - `~/.ipc-node/logs/node.stdout.log`
   - `~/.ipc-node/logs/node.stderr.log`
   - `~/.ipc-node/logs/relayer.log`

## Troubleshooting

### Service won't start
```bash
# Check status and errors
sudo systemctl status ipc-node
sudo journalctl -u ipc-node -n 50 --no-pager

# Check service file syntax
sudo systemd-analyze verify /etc/systemd/system/ipc-node.service
```

### Permission errors
```bash
# Ensure ipc user owns the files
sudo chown -R ipc:ipc /home/ipc/.ipc-node

# Check service user
sudo systemctl show ipc-node | grep ^User
```

### Manager script not detecting systemd
The script checks for service existence:
```bash
# Verify service is installed
systemctl list-unit-files ipc-node.service
```

## Uninstall

To remove systemd services:

```bash
# On each validator
sudo systemctl stop ipc-node
sudo systemctl disable ipc-node
sudo rm /etc/systemd/system/ipc-node.service

# On primary validator only
sudo systemctl stop ipc-relayer
sudo systemctl disable ipc-relayer
sudo rm /etc/systemd/system/ipc-relayer.service

# Reload
sudo systemctl daemon-reload
```

The manager script will fall back to manual process management (nohup/kill) if systemd services are not found.

## Benefits Over User Services

1. **Reliability**: No dbus or environment variable issues
2. **SSH Compatibility**: Works perfectly via SSH
3. **Standard Management**: Uses familiar system service patterns
4. **Better Logging**: Integrated with system journal
5. **Production Ready**: Standard approach for production services
6. **Auto-restart**: Systemd automatically restarts failed services
7. **Resource Limits**: Can set limits via service file

## Files Modified

1. `templates/ipc-node.service.template` - Added `User=`, changed target
2. `templates/ipc-relayer.service.template` - Added `User=`, changed target
3. `lib/health.sh`:
   - `check_systemd_available()` - Simplified to check system systemd
   - `install_systemd_services()` - Install to /etc/systemd/system
   - `install_relayer_systemd_service()` - Install to /etc/systemd/system
   - `start_validator_node()` - Use `sudo systemctl`
   - `stop_all_nodes()` - Use `sudo systemctl`
   - `start_relayer()` - Use `sudo systemctl`
   - `stop_relayer()` - Use `sudo systemctl`
   - `check_relayer_status()` - Use `sudo systemctl` and `sudo journalctl`
4. `ipc-subnet-manager.sh`:
   - `cmd_install_systemd()` - Updated documentation messages

## Testing

1. **Install services:**
   ```bash
   ./ipc-manager install-systemd --with-relayer --yes
   ```

2. **Verify installation:**
   ```bash
   # On each validator
   systemctl list-unit-files | grep ipc
   ls -la /etc/systemd/system/ipc-*
   ```

3. **Start nodes:**
   ```bash
   ./ipc-manager restart
   ```

4. **Start relayer:**
   ```bash
   ./ipc-manager start-relayer
   ```

5. **Check status:**
   ```bash
   ./ipc-manager relayer-status
   sudo systemctl status ipc-node
   sudo systemctl status ipc-relayer
   ```

6. **View logs:**
   ```bash
   sudo journalctl -u ipc-node -f
   sudo journalctl -u ipc-relayer -f
   ```

