# Watch Blocks Feature

## Overview

Added a new `watch-blocks` command to the IPC subnet manager that provides real-time monitoring of block production timing and performance.

## Usage

### Basic Monitoring (Continuous)
```bash
./ipc-manager watch-blocks
```

Monitors block production indefinitely until Ctrl+C is pressed. Useful for observing subnet performance.

### Monitor Until Target Height
```bash
./ipc-manager watch-blocks --target-height=1000
```

Monitors until the specified block height is reached, then automatically exits.

### Custom Refresh Interval
```bash
./ipc-manager watch-blocks --interval=5
```

Changes the refresh interval (default: 2 seconds). Useful for reducing overhead or getting more frequent updates.

### Combined Example
```bash
./ipc-manager watch-blocks --target-height=1000 --interval=1
```

## Output

The command displays a table with the following metrics:

- **Time**: Current time of the measurement
- **Iter**: Iteration count
- **Height**: Current block height
- **Δ Blocks**: Number of blocks produced since last check
- **Block Time**: Time taken to produce the recent blocks (seconds)
- **Blocks/s**: Block production rate
- **Avg Time**: Average block time over entire monitoring period
- **Status**: Production status or progress toward target

### Example Output

**Continuous Mode:**
```
========================================
  Block Production Monitor
========================================

Monitoring block production (Ctrl+C to stop)
Refresh interval: 2s
Source: validator-1

Time      | Iter | Height  | Δ Blocks | Block Time | Blocks/s | Avg Time | Status
----------|------|---------|----------|------------|----------|----------|--------
11:09:59 | 1    | 755     | 0        | N/As       | 0.00     | N/As     | stalled
11:10:01 | 2    | 755     | 0        | N/As       | 0.00     | N/As     | stalled
11:10:04 | 3    | 756     | 1        | 2.00s      | .50      | 2.00s    | producing
11:10:06 | 4    | 756     | 0        | N/As       | 0.00     | 2.00s    | stalled
11:10:09 | 5    | 757     | 1        | 2.00s      | .50      | 2.00s    | producing
11:10:12 | 6    | 757     | 0        | N/As       | 0.00     | 2.00s    | stalled
11:10:14 | 7    | 758     | 1        | 3.00s      | .33      | 2.33s    | producing
```

**Target Height Mode:**
```
========================================
  Block Production Monitor
========================================

Monitoring until block height: 770
Refresh interval: 2s
Source: validator-1

Time      | Iter | Height  | Δ Blocks | Block Time | Blocks/s | Avg Time | Status
----------|------|---------|----------|------------|----------|----------|--------
11:10:38 | 1    | 762     | 0        | N/As       | 0.00     | N/As     | 8 left
11:10:41 | 2    | 763     | 1        | 2.00s      | .50      | 2.00s    | 7 left
11:10:44 | 3    | 763     | 0        | N/As       | 0.00     | 2.00s    | 7 left
11:10:46 | 4    | 764     | 1        | 3.00s      | .33      | 2.50s    | 6 left
...
11:11:20 | 20   | 770     | 1        | 2.00s      | .50      | 2.50s    | ✓ REACHED

✓ Target height 770 reached!
  Current height: 770
  Total blocks produced: 8
  Average block time: 2.50s
  Total elapsed time: 40s
```

## Metrics Explained

### Δ Blocks (Delta Blocks)
Number of new blocks since the last measurement. In a healthy subnet:
- **0**: No new blocks (might be normal if refresh interval is faster than block time)
- **1-3**: Normal range for 2-second intervals
- **>5**: Catching up after a delay

### Block Time
Time taken to produce the Δ blocks:
- **1-2s**: Fast block production
- **2-5s**: Normal range
- **>5s**: Slower than expected (might indicate issues)
- **N/A**: No blocks produced in this interval

### Blocks/s (Blocks per Second)
Instantaneous block production rate:
- **0.00**: No blocks this interval
- **0.33-0.50**: Normal range (2-3 second block times)
- **>1.00**: Very fast production (catching up or very fast consensus)

### Avg Time (Average Block Time)
Running average of all block times during the monitoring session:
- This smooths out variations and gives you the actual subnet performance
- Should converge to a stable value after 10-20 blocks
- Typical healthy range: 1-3 seconds

### Status
- **stalled**: No blocks produced in this interval (not necessarily a problem)
- **producing**: Actively producing blocks
- **reorg?**: Block height decreased (potential chain reorganization - rare)
- **X left**: When monitoring to target, shows blocks remaining
- **✓ REACHED**: Target height achieved

## Use Cases

### 1. Verifying Subnet Performance

Check if your subnet is producing blocks at the expected rate:

```bash
# Watch for 1 minute
timeout 60 ./ipc-manager watch-blocks

# Look at "Avg Time" after 30+ seconds
# Expected: 1-3 seconds per block
```

### 2. Detecting Block Production Issues

Monitor to see if block production stalls:

```bash
./ipc-manager watch-blocks --interval=5

# Watch the "Status" column
# If you see "stalled" for >3-4 consecutive iterations, investigate:
# - Check validator connectivity (./ipc-manager check)
# - Check validator voting power
# - Look for errors in logs
```

### 3. Measuring Performance After Config Changes

Before and after making configuration changes:

```bash
# Before change
./ipc-manager watch-blocks --interval=3
# Note the "Avg Time"

# Make configuration change and restart
./ipc-manager update-config
./ipc-manager restart

# After change
./ipc-manager watch-blocks --interval=3
# Compare "Avg Time" to see if performance improved
```

### 4. Waiting for Blocks Before Testing

Ensure subnet has produced some blocks before running tests:

```bash
# Current height: 100
# Wait for 20 more blocks
./ipc-manager watch-blocks --target-height=120

# Then run your tests
```

### 5. Estimating Time to Reach Height

Use the average block time to estimate when a target will be reached:

```bash
# Current: 500, Target: 1000
# Gap: 500 blocks
# If avg block time is 2.5s:
# Estimated time: 500 × 2.5s = 1,250s ≈ 21 minutes

./ipc-manager watch-blocks --target-height=1000
```

## Interpreting Results

### Healthy Subnet
```
Time      | Iter | Height  | Δ Blocks | Block Time | Blocks/s | Avg Time | Status
----------|------|---------|----------|------------|----------|----------|--------
11:00:00 | 1    | 100     | 1        | 2.00s      | .50      | 2.00s    | producing
11:00:02 | 2    | 101     | 1        | 2.00s      | .50      | 2.00s    | producing
11:00:04 | 3    | 102     | 1        | 2.00s      | .50      | 2.00s    | producing
```
**Signs**: Consistent Δ blocks, stable avg time, "producing" status

### Slow but Steady
```
Time      | Iter | Height  | Δ Blocks | Block Time | Blocks/s | Avg Time | Status
----------|------|---------|----------|------------|----------|----------|--------
11:00:00 | 1    | 100     | 0        | N/As       | 0.00     | N/As     | stalled
11:00:02 | 2    | 100     | 0        | N/As       | 0.00     | N/As     | stalled
11:00:04 | 3    | 101     | 1        | 4.00s      | .25      | 4.00s    | producing
```
**Signs**: Alternating stalled/producing, higher avg time (4s+)
**Action**: May be normal if validators are geographically distributed

### Completely Stalled
```
Time      | Iter | Height  | Δ Blocks | Block Time | Blocks/s | Avg Time | Status
----------|------|---------|----------|------------|----------|----------|--------
11:00:00 | 1    | 100     | 0        | N/As       | 0.00     | N/As     | stalled
11:00:02 | 2    | 100     | 0        | N/As       | 0.00     | N/As     | stalled
11:00:04 | 3    | 100     | 0        | N/As       | 0.00     | N/As     | stalled
11:00:06 | 4    | 100     | 0        | N/As       | 0.00     | N/As     | stalled
```
**Signs**: No blocks for extended period (>30 seconds)
**Action**: Immediate investigation needed!
```bash
./ipc-manager check  # Check validator health
./ipc-manager info   # Check voting power and quorum
```

### Catching Up After Delay
```
Time      | Iter | Height  | Δ Blocks | Block Time | Blocks/s | Avg Time | Status
----------|------|---------|----------|------------|----------|----------|--------
11:00:00 | 1    | 100     | 3        | 2.00s      | 1.50     | 0.67s    | producing
11:00:02 | 2    | 103     | 3        | 2.00s      | 1.50     | 0.67s    | producing
11:00:04 | 3    | 105     | 2        | 2.00s      | 1.00     | 0.75s    | producing
```
**Signs**: Multiple blocks per interval (Δ > 1), high blocks/s, low avg time
**Interpretation**: Node catching up after being behind or restart

## Performance Benchmarks

Based on typical IPC subnet configurations:

### CometBFT with 3 Validators
- **Expected avg block time**: 1-3 seconds
- **Blocks per minute**: 20-60
- **Normal variation**: ±30%

### Factors Affecting Block Time
1. **Network latency** between validators
2. **Validator count** (more validators = slightly slower consensus)
3. **Transaction volume** in blocks
4. **Hardware performance** of validator nodes
5. **CometBFT configuration** (`timeout_commit` setting)

## Troubleshooting

### Command shows "0" for all values

**Issue**: Cannot connect to validator

**Solution**:
```bash
# Test connectivity
./ipc-manager check

# Verify first validator is running
ssh validator-1 "curl -s http://localhost:26657/status | jq '.result.sync_info.latest_block_height'"
```

### "stalled" status persists

**Issue**: No blocks being produced

**Causes**:
1. Insufficient voting power / no quorum
2. Validators not connected
3. Validators stopped or crashed

**Diagnosis**:
```bash
# Check overall health
./ipc-manager info

# Check validator status
./ipc-manager check

# Check logs for errors
./ipc-manager logs validator-1 | grep -i error
```

### Highly variable block times

**Issue**: Avg time keeps changing significantly

**Normal**: Some variation is expected (±1 second)

**If excessive** (varying by >3 seconds):
- Check network connectivity between validators
- Check for resource constraints (CPU, memory)
- Look for validators going offline/online

### Negative Δ Blocks

**Issue**: Shows reorg?

**Interpretation**: Chain reorganization occurred

**Actions**:
```bash
# Check all validators for consistency
for v in validator-1 validator-2 validator-3; do
  ssh $v "curl -s http://localhost:26657/status | jq '.result.sync_info.latest_block_height'"
done

# Check logs for reorg evidence
./ipc-manager logs validator-1 | grep -i reorg
```

## Comparison with `block-time` Command

The subnet manager has two block-related commands:

### `block-time` (One-time Measurement)
```bash
./ipc-manager block-time --duration=10
```
- Takes a single measurement over X seconds
- Gives average block time for that period
- Exits after measurement
- Good for quick checks

### `watch-blocks` (Continuous Monitoring)
```bash
./ipc-manager watch-blocks
```
- Continuous real-time updates
- Shows each interval's metrics
- Tracks trends over time
- Shows instantaneous and average performance
- Can monitor to specific target
- Good for ongoing observation and diagnostics

## Related Commands

- **`./ipc-manager block-time`** - One-time block time measurement
- **`./ipc-manager info`** - Snapshot of subnet status
- **`./ipc-manager check`** - Comprehensive health check
- **`./ipc-manager watch-finality`** - Monitor parent finality progress

## Tips

1. **Use shorter intervals** (1-2s) for detailed observation
2. **Use longer intervals** (5-10s) to reduce SSH overhead
3. **Let it run for 30+ seconds** before judging avg block time
4. **Monitor during peak usage** to see performance under load
5. **Compare before/after changes** to measure impact

## Future Enhancements

### Planned Features

1. **Multi-validator comparison**
   ```bash
   ./ipc-manager watch-blocks --all-validators
   ```
   Show block production from all validators' perspectives

2. **Transaction throughput**
   ```bash
   ./ipc-manager watch-blocks --show-tx
   ```
   Include transaction count per block

3. **Alert on stalls**
   ```bash
   ./ipc-manager watch-blocks --alert-stall=30
   ```
   Alert if no blocks for X seconds

4. **Export mode**
   ```bash
   ./ipc-manager watch-blocks --export=csv > blocks.csv
   ```
   Export data for analysis

5. **Historical comparison**
   ```bash
   ./ipc-manager watch-blocks --compare=yesterday
   ```
   Compare current performance to previous measurements

---

**Feature Added**: October 18, 2025
**Version**: 1.0
**Status**: Production Ready

