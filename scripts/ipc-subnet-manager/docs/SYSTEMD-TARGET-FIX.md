# Systemd Target Fix

## Issues Fixed

### 1. Wrong Systemd Target
**Problem:** Service templates used `multi-user.target` which only exists for system services
**Error:** `Unit /home/ipc/.config/systemd/user/ipc-node.service is added as a dependency to a non-existent unit multi-user.target`

**Fix:** Changed both service templates to use `default.target` instead:
- `ipc-node.service.template`: `WantedBy=default.target`
- `ipc-relayer.service.template`: `WantedBy=default.target`

### 2. Incorrect User Directive
**Problem:** User services had `User=__IPC_USER__` which is redundant for user systemd services
**Fix:** Removed `User=` directive from both templates since user services already run as the owning user

### 3. Error Output Causing Loop Issues
**Problem:** Systemd warnings on stderr might have stopped the installation loop
**Fix:** Changed error handling from `|| { }` syntax to `if !` syntax with stderr redirected to prevent spurious failures

## What Changed

### Service Templates

**Both `ipc-node.service.template` and `ipc-relayer.service.template`:**

```diff
[Service]
Type=simple
-User=__IPC_USER__
WorkingDirectory=__NODE_HOME__
...

[Install]
-WantedBy=multi-user.target
+WantedBy=default.target
```

### Error Handling in Installation Functions

**Changed from:**
```bash
ssh_exec ... 2>&1 || {
    log_error "..."
    return 1
}
```

**To:**
```bash
if ! ssh_exec ... >/dev/null 2>&1; then
    log_error "..."
    return 1
fi
```

This prevents stderr output (even if exit code is 0) from causing issues with the loop.

## How to Test

1. **Remove existing services** (if any):
   ```bash
   # On each validator
   systemctl --user disable ipc-node.service 2>/dev/null || true
   systemctl --user disable ipc-relayer.service 2>/dev/null || true
   rm -f ~/.config/systemd/user/ipc-node.service
   rm -f ~/.config/systemd/user/ipc-relayer.service
   systemctl --user daemon-reload
   ```

2. **Reinstall services:**
   ```bash
   ./ipc-manager install-systemd --with-relayer --yes
   ```

3. **Verify installation on all validators:**
   ```bash
   # Should show installation messages for all 3 validators
   # Plus relayer installation on primary validator
   ```

4. **Check services are enabled:**
   ```bash
   # On each validator
   export XDG_RUNTIME_DIR=/run/user/$(id -u)
   systemctl --user list-unit-files | grep ipc
   # Should show:
   # ipc-node.service     enabled
   # ipc-relayer.service  enabled  (on primary only)
   ```

5. **Check symlinks are correct:**
   ```bash
   ls -la ~/.config/systemd/user/default.target.wants/
   # Should show symlinks to ipc-node.service (and ipc-relayer.service on primary)
   ```

## Expected Behavior After Fix

When running `./ipc-manager install-systemd --with-relayer --yes`:

1. **Checks systemd availability** on each validator
2. **Installs node service** on validator-1, validator-2, and validator-3
3. **Installs relayer service** on primary validator only
4. **Shows summary** with success/failure counts

Example output:
```
>>> Installing Node Services

[INFO] Checking systemd availability on validator-1...
[INFO] Installing systemd services on validator-1...
[SUCCESS] ✓ Node service installed on validator-1

[INFO] Checking systemd availability on validator-2...
[INFO] Installing systemd services on validator-2...
[SUCCESS] ✓ Node service installed on validator-2

[INFO] Checking systemd availability on validator-3...
[INFO] Installing systemd services on validator-3...
[SUCCESS] ✓ Node service installed on validator-3

>>> Installing Relayer Service

[INFO] Installing relayer systemd service on validator-1...
[SUCCESS] ✓ Relayer service installed on validator-1

Installation Summary:
  ✓ Successful: 4
```

## Service Location

**Correct location (user services):**
```
~/.config/systemd/user/ipc-node.service
~/.config/systemd/user/ipc-relayer.service
~/.config/systemd/user/default.target.wants/ipc-node.service -> ../ipc-node.service
~/.config/systemd/user/default.target.wants/ipc-relayer.service -> ../ipc-relayer.service
```

**NOT** `/etc/systemd/system/` (that's for system services run as root)

## Files Modified

1. `templates/ipc-node.service.template` - Fixed target and removed User directive
2. `templates/ipc-relayer.service.template` - Fixed target and removed User directive
3. `lib/health.sh` - Improved error handling in installation functions

## Notes

- User systemd services are installed in `~/.config/systemd/user/`
- They use `default.target` not `multi-user.target`
- They don't need a `User=` directive
- They run as the user who owns the systemd instance
- They require `loginctl enable-linger <user>` to run without an active login session

