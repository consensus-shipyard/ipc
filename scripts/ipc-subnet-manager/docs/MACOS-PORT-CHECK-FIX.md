# macOS Port Check Fix

## Problem
Health checks were reporting "Ports not listening (0/3)" even though the ports were actually listening and the node was working correctly.

```bash
[✓] Process running
[✗] Ports not listening (       0/3)  # ❌ FALSE NEGATIVE
[✓] CometBFT peers: 0/0
[✓] Block height: 58
```

## Root Cause
The port check in `check_validator_health()` was using a Linux-style `netstat` pattern that doesn't work on macOS:

### Linux Format
```bash
$ netstat -tuln | grep LISTEN
tcp        0      0 *:8546                  *:*                     LISTEN
tcp        0      0 *:26657                 *:*                     LISTEN
```
Ports shown with `:` separator (e.g., `*:8546`)

### macOS Format
```bash
$ netstat -an | grep LISTEN
tcp4       0      0  *.8546                 *.*                    LISTEN
tcp46      0      0  *.26657                *.*                    LISTEN
```
Ports shown with `.` separator (e.g., `*.8546`)

## The Fix

Changed the port detection pattern to work on both Linux and macOS:

### Before (Linux-only)
```bash
netstat -tuln 2>/dev/null | grep -E \":($cometbft_port|$libp2p_port|$eth_api_port)\" | wc -l
```

### After (Cross-platform)
```bash
netstat -an 2>/dev/null | grep LISTEN | grep -E \"[\.:]$cometbft_port|[\.:]$libp2p_port|[\.:]$eth_api_port\" | wc -l
```

### Key Changes
1. **`-an` instead of `-tuln`**: Works on both macOS and Linux
2. **`grep LISTEN`**: Explicitly filter for listening ports
3. **`[\.:]`**: Matches both `.` (macOS) and `:` (Linux) separators
4. **Separate alternations**: `[\.:]port1|[\.:]port2` instead of `[\.:]( port1|port2)`

## Verification

### Test on macOS
```bash
$ netstat -an 2>/dev/null | grep LISTEN | grep -E "[\.:]26657|[\.:]26655|[\.:]8546" | wc -l
       3
```
✅ Correctly detects 3 listening ports

### Test Health Check
```bash
$ ./ipc-manager --config ipc-subnet-config-local.yml check
  -- Checking validator-0
[✓] Process running
[✓] Ports listening (       3/3)  # ✅ NOW WORKS!
[✓] CometBFT peers: 0/0
[✓] Block height: 32156
[✓] No recent errors
```

## Files Modified
- `/Users/philip/github/ipc/scripts/ipc-subnet-manager/lib/health.sh`
  - Function: `check_validator_health()`
  - Line: ~447

## Testing on Linux
This fix maintains compatibility with Linux systems:

```bash
# Linux netstat output
$ netstat -an | grep LISTEN | grep -E "[\.:]8546"
tcp        0      0 0.0.0.0:8546            0.0.0.0:*               LISTEN
```

The pattern `[\.:]` matches the `:` in Linux output just as it matches `.` in macOS output.

## Related Issues
This fix ensures the health check works correctly on:
- ✅ macOS (Darwin) - Uses `.` separator
- ✅ Linux - Uses `:` separator
- ✅ Local mode deployments
- ✅ Remote mode deployments

## Impact
- Health checks now correctly report port status on macOS
- No false negatives about ports not listening
- Better developer experience on macOS for local development
