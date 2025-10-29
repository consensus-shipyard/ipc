# Watch Finality Feature

## Overview

Added a new `watch-finality` command to the IPC subnet manager that provides real-time monitoring of parent finality progress.

## Usage

### Basic Monitoring (Continuous)
```bash
./ipc-manager watch-finality
```

Monitors parent finality indefinitely until Ctrl+C is pressed. Useful for general observation.

### Monitor Until Target Epoch
```bash
./ipc-manager watch-finality --target-epoch=3115755
```

Monitors until the specified parent epoch is reached, then automatically exits. Perfect for tracking when a specific cross-msg transaction will be processed.

### Custom Refresh Interval
```bash
./ipc-manager watch-finality --interval=10
```

Changes the refresh interval (default: 5 seconds). Useful for reducing SSH overhead.

### Combined Example
```bash
./ipc-manager watch-finality --target-epoch=3115755 --interval=3
```

## Output

The command displays:
- **Real-time progress**: Current parent finality height and subnet block height
- **Elapsed time**: Time since monitoring started
- **Iteration count**: Number of refresh cycles
- **Progress tracking**: When a target is set, shows epochs remaining
- **Periodic updates**: Every 10 iterations, displays detailed status with timestamp

### Example Output

**Continuous Mode:**
```
========================================
  Parent Finality Monitor
========================================

Monitoring parent finality progress (Ctrl+C to stop)
Refresh interval: 5s
Source: validator-1

[10:56:42] Iteration: 1 | Elapsed: 0s | Parent: 3115746 | Subnet: 607
[10:56:49] Iteration: 2 | Elapsed: 7s | Parent: 3115746 | Subnet: 608
...
[10:57:44] Iteration: 10 | Elapsed: 62s | Parent: 3115748 | Subnet: 618
Status update (#10):
  Parent finality height: 3115748
  Subnet block height: 618
  Last parent finality: 2025-10-18T14:57:39
```

**Target Epoch Mode:**
```
========================================
  Parent Finality Monitor
========================================

Monitoring until parent epoch: 3115755
Refresh interval: 5s
Source: validator-1

[10:59:16] Iteration: 1 | Elapsed: 0s | Parent: 3115751 | Subnet: 635 | 4 epochs remaining
[10:59:22] Iteration: 2 | Elapsed: 7s | Parent: 3115751 | Subnet: 637 | 4 epochs remaining
[10:59:42] Iteration: 5 | Elapsed: 27s | Parent: 3115752 | Subnet: 640 | 3 epochs remaining
...

✓ Target epoch 3115755 reached!
  Current parent height: 3115755
  Current subnet height: 650
  Last finality: 2025-10-18T15:02:15
```

## Use Cases

### 1. Tracking Cross-Msg Fund Transactions

After submitting a `cross-msg fund`, you can watch for when it will be processed:

```bash
# Submit transaction (returns epoch in output)
ipc-cli cross-msg fund --from 0x... --to 0x... --subnet /r314159/... 10

# Watch until that epoch
./ipc-manager watch-finality --target-epoch=3115719
```

### 2. Monitoring Parent Finality Health

Check if parent finality is progressing normally:

```bash
# Watch for 1 minute to see progress rate
timeout 60 ./ipc-manager watch-finality
```

Expected: Parent height should advance ~1-2 epochs per minute (depending on parent chain block time).

### 3. Debugging Parent Finality Issues

If parent finality appears stuck:

```bash
# Watch and observe if height is advancing
./ipc-manager watch-finality --interval=10
```

If parent height doesn't change for >5 minutes, check:
- Parent RPC connectivity
- Validator voting power and quorum
- Parent finality configuration

### 4. Estimating Transaction Processing Time

Use current lag to estimate when a transaction will execute:

```bash
# Current parent finality: 3115700
# Transaction epoch: 3115750
# Lag: 50 epochs
# Parent block time: ~30 seconds
# Estimated time: 50 * 30s = 25 minutes

./ipc-manager watch-finality --target-epoch=3115750
```

## Implementation Details

### Files Modified

1. **`ipc-subnet-manager.sh`**
   - Added `cmd_watch_finality()` function
   - Added command parser case for `watch-finality`
   - Updated usage documentation

2. **`lib/health.sh`**
   - Added `watch_parent_finality()` function
   - Implements real-time monitoring logic
   - Fetches data via SSH from first validator

### Technical Approach

The monitor:
1. Queries the first validator's logs for `ParentFinalityCommitted` events
2. Extracts the latest parent finality height
3. Queries CometBFT's `/status` endpoint for subnet height
4. Updates display every refresh interval
5. Automatically exits when target reached (if specified)

### Performance Considerations

- **SSH overhead**: Each iteration makes 2-3 SSH calls
- **Log parsing**: Greps through potentially large log files
- **Recommended interval**: 5-15 seconds balances responsiveness vs overhead
- **Network usage**: ~1-2KB per iteration

### Limitations

1. **Single validator monitoring**: Uses only the first validator
   - Pro: Reduces network overhead
   - Con: If first validator is down, command fails

2. **Log-based tracking**: Relies on log file grep
   - Pro: Works without custom APIs
   - Con: Slower than direct state queries

3. **No alert mechanism**: Just displays progress
   - Future enhancement: Add webhook/notification support

## Future Enhancements

### Planned Features

1. **Balance tracking integration**
   ```bash
   ./ipc-manager watch-finality --target-epoch=3115719 --check-balance=0x...
   ```
   Automatically check if balance updated when epoch reached.

2. **Multi-validator monitoring**
   ```bash
   ./ipc-manager watch-finality --all-validators
   ```
   Show parent finality height from all validators (detect inconsistencies).

3. **Export mode**
   ```bash
   ./ipc-manager watch-finality --export=csv > finality-log.csv
   ```
   Export monitoring data for analysis.

4. **Notification support**
   ```bash
   ./ipc-manager watch-finality --target-epoch=3115719 --notify=email@example.com
   ```
   Send alert when target reached.

5. **Comparison mode**
   ```bash
   ./ipc-manager watch-finality --compare-validators
   ```
   Show how parent finality differs across validators (detect sync issues).

## Related Commands

- **`./ipc-manager info`** - One-time snapshot of subnet status including parent finality
- **`./ipc-manager check`** - Health check including parent finality validation
- **`./ipc-manager block-time`** - Measure subnet block production rate

## Troubleshooting

### Command hangs at startup

**Issue**: SSH connection problems

**Solution**:
```bash
# Test SSH connectivity first
./ipc-manager check
```

### Parent height shows 0

**Issue**: Validator logs don't contain `ParentFinalityCommitted` events

**Causes**:
- Parent finality not working (check with `./ipc-manager info`)
- Logs rotated (check log file dates)
- Wrong validator name in config

**Solution**:
```bash
# Check if parent finality is working
./ipc-manager info | grep -A10 "Parent Finality"
```

### Height advances very slowly

**Normal**: Parent finality follows parent chain block time (~30 seconds per epoch on Calibration)

**If stuck**: Parent finality may have issues:
```bash
# Check for errors
ssh validator-1 "grep -i error ~/.ipc-node/logs/*.log | grep -i parent | tail -20"
```

## Example Session

```bash
$ ./ipc-manager watch-finality --target-epoch=3115800

========================================
  Parent Finality Monitor
========================================

Monitoring until parent epoch: 3115800
Refresh interval: 5s
Source: validator-1

[14:00:00] Iteration: 1 | Elapsed: 0s | Parent: 3115750 | Subnet: 500 | 50 epochs remaining
[14:00:05] Iteration: 2 | Elapsed: 5s | Parent: 3115750 | Subnet: 501 | 50 epochs remaining
[14:00:10] Iteration: 3 | Elapsed: 10s | Parent: 3115751 | Subnet: 502 | 49 epochs remaining
...
[14:25:00] Iteration: 300 | Elapsed: 1500s | Parent: 3115800 | Subnet: 800 | ✓ TARGET REACHED

✓ Target epoch 3115800 reached!
  Current parent height: 3115800
  Current subnet height: 800
  Last finality: 2025-10-18T14:25:00
```

---

**Feature Added**: October 18, 2025
**Version**: 1.0
**Status**: Production Ready

