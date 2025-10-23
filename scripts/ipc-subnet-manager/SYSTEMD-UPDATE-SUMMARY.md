# IPC Subnet Manager - Systemd Integration

## Summary

This update adds full systemd integration for managing both IPC validator nodes and the checkpoint relayer, replacing the previous nohup-based process management. This prevents issues like the relayer stop accidentally killing the node process.

## What's New

### 1. Systemd Service Templates

Created two systemd service templates that are customized per validator:

#### `templates/ipc-node.service.template`
- Manages the IPC validator node
- Automatic restart on failure
- Proper logging to `~/.ipc-node/logs/`
- Resource limits configured
- Security hardening enabled

#### `templates/ipc-relayer.service.template`
- Manages the checkpoint relayer
- Depends on ipc-node service (starts after node is running)
- Automatic restart on failure
- Logs to `~/.ipc-node/logs/relayer.log` and systemd journal

### 2. New Command: `install-systemd`

```bash
# Install node services on all validators
./ipc-manager install-systemd

# Install node + relayer services
./ipc-manager install-systemd --with-relayer

# Skip confirmation
./ipc-manager install-systemd --yes
```

**What it does:**
- Generates customized systemd service files for each validator
- Installs services to `~/.config/systemd/user/`
- Enables user lingering (services run without login)
- Enables services for auto-start
- Configures proper permissions and paths

### 3. Updated Start/Stop Logic

All start/stop commands now intelligently detect and use systemd:

**Start/Stop Nodes:**
- Checks if systemd service exists
- If yes: uses `systemctl --user start/stop ipc-node`
- If no: falls back to nohup/kill

**Start/Stop Relayer:**
- Checks if systemd service exists
- If yes: uses `systemctl --user start/stop ipc-relayer`
- If no: falls back to nohup/kill

This provides backward compatibility while enabling modern service management.

### 4. Improved Status Checking

The `relayer-status` command now:
- Detects if using systemd or manual process management
- For systemd: shows service status and journal logs
- For manual: shows PID and log file contents

## Usage

### Initial Setup (One-Time)

After initializing your subnet, install systemd services:

```bash
# Install node services on all validators
./ipc-manager install-systemd

# Or install with relayer (on primary validator)
./ipc-manager install-systemd --with-relayer --yes
```

### Managing Services

Once systemd is installed, all existing commands work automatically:

```bash
# Start/stop/restart nodes (uses systemd automatically)
./ipc-manager restart
./ipc-manager check

# Start/stop relayer (uses systemd automatically)
./ipc-manager start-relayer
./ipc-manager stop-relayer
./ipc-manager relayer-status
```

### Direct Systemd Commands

You can also use systemd directly on any validator:

```bash
# Node management
systemctl --user status ipc-node
systemctl --user start ipc-node
systemctl --user stop ipc-node
systemctl --user restart ipc-node
journal ctl --user -u ipc-node -f

# Relayer management (on primary validator)
systemctl --user status ipc-relayer
systemctl --user start ipc-relayer
systemctl --user stop ipc-relayer
journalctl --user -u ipc-relayer -f
```

### View Logs

**Using systemd journal:**
```bash
# Node logs
journalctl --user -u ipc-node -f

# Relayer logs
journalctl --user -u ipc-relayer -f

# Show last 100 lines
journalctl --user -u ipc-node -n 100
```

**Using log files:**
```bash
# Node logs
tail -f ~/.ipc-node/logs/node.stdout.log
tail -f ~/.ipc-node/logs/node.stderr.log

# Relayer logs
tail -f ~/.ipc-node/logs/relayer.log
```

## Benefits

### 1. **Process Isolation**
- Node and relayer run as separate services
- Stopping one doesn't affect the other
- No more accidental process kills

### 2. **Automatic Restart**
- Services restart automatically on failure
- Configurable restart policies
- Better reliability

### 3. **Better Logging**
- Logs go to both files and systemd journal
- Structured logging with timestamps
- Easy log rotation and management

### 4. **Resource Management**
- File descriptor limits configured
- Process limits set
- Memory and CPU can be limited if needed

### 5. **Security**
- NoNewPrivileges prevents privilege escalation
- PrivateTmp provides isolated /tmp
- Services run as unprivileged user

### 6. **Ease of Management**
- Standard systemd commands
- Integration with system monitoring
- Service dependencies properly configured

## Service Configuration

### Node Service Details

- **Type:** simple
- **User:** Configured ipc_user
- **WorkingDirectory:** Node home directory
- **Restart:** on-failure (5s delay, max 5 attempts in 5 minutes)
- **Logs:** Both stdout and stderr to separate files
- **Limits:** 65536 file descriptors, 32768 processes

### Relayer Service Details

- **Type:** simple
- **User:** Configured ipc_user
- **Depends On:** ipc-node.service (won't start without node)
- **Restart:** on-failure (10s delay, max 5 attempts in 5 minutes)
- **Logs:** Combined stdout/stderr to relayer.log
- **Limits:** 65536 file descriptors

## Troubleshooting

### Service Won't Start

```bash
# Check service status
systemctl --user status ipc-node

# View full logs
journalctl --user -u ipc-node -n 50

# Check configuration
systemctl --user cat ipc-node
```

### Relayer Not Starting

```bash
# Check if node is running first
systemctl --user status ipc-node

# Check relayer status
systemctl --user status ipc-relayer

# View logs
journalctl --user -u ipc-relayer -n 50
```

### Reinstall Services

```bash
# Stop services first
./ipc-manager stop-relayer
./ipc-manager restart  # This stops nodes

# Reinstall
./ipc-manager install-systemd --with-relayer --yes

# Start again
./ipc-manager restart
./ipc-manager start-relayer
```

### Check Lingering

User lingering must be enabled for services to run without login:

```bash
# Check if enabled
loginctl show-user $USER | grep Linger

# Enable manually if needed
sudo loginctl enable-linger $USER
```

## Files Modified

1. **templates/ipc-node.service.template** - New systemd service template for nodes
2. **templates/ipc-relayer.service.template** - New systemd service template for relayer
3. **lib/health.sh** - Added systemd generation and management functions
4. **ipc-subnet-manager.sh** - Added `install-systemd` command and integration

## Migration Path

### For Existing Deployments

If you already have nodes running with nohup:

1. **Stop everything cleanly:**
   ```bash
   ./ipc-manager stop-relayer
   # Manually kill any remaining processes if needed
   ```

2. **Install systemd services:**
   ```bash
   ./ipc-manager install-systemd --with-relayer --yes
   ```

3. **Start with systemd:**
   ```bash
   ./ipc-manager restart
   ./ipc-manager start-relayer
   ```

4. **Verify:**
   ```bash
   ./ipc-manager check
   ./ipc-manager relayer-status
   ```

### For New Deployments

After running `./ipc-manager init`, immediately install systemd:

```bash
./ipc-manager init
./ipc-manager install-systemd --with-relayer --yes
./ipc-manager restart
./ipc-manager start-relayer
```

## Notes

- Systemd services are installed per-user (`--user` flag)
- Services persist across reboots (with lingering enabled)
- Log files are still written for compatibility
- Falls back to nohup if systemd not available
- All existing commands work with or without systemd

