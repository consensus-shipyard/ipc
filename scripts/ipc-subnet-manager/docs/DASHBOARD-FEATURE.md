# Live Monitoring Dashboard

## Overview

The dashboard command provides a comprehensive, real-time monitoring interface for your IPC subnet. It combines multiple metrics into a single, continuously updating display similar to tools like `htop` or `docker stats`.

## Features

### 📊 Real-Time Metrics

1. **Block Production**
   - Current block height
   - Blocks produced per minute
   - Average block time
   - Production status

2. **Parent Finality**
   - Subnet's parent finality height
   - Parent chain's actual height
   - Lag between subnet and parent
   - Last commit timestamp

3. **Network Health**
   - CometBFT peer count
   - Libp2p peer connections
   - RPC responsiveness

4. **Mempool Status**
   - Current transaction count
   - Capacity utilization percentage
   - Memory size usage
   - Health status

5. **Checkpoint Activity**
   - Signature broadcasts
   - Success rate
   - Last activity timestamp

6. **Error Tracking**
   - Categorized error counts
   - Error rate per minute
   - Sample error messages
   - Categories:
     - Bottom-up Checkpoint errors
     - Parent Finality errors
     - Network/P2P errors
     - Consensus errors
     - RPC/API errors
     - Other errors

7. **Recent Events**
   - Last 5 significant events
   - Timestamped activity log

## Usage

### Basic Usage

```bash
./ipc-manager dashboard
```

This starts the dashboard monitoring the first validator (`validator-1`) with a 3-second refresh interval.

### Monitor Specific Validator

```bash
./ipc-manager dashboard --validator=validator-2
```

### Adjust Refresh Interval

```bash
./ipc-manager dashboard --interval=5
```

### Combined Options

```bash
./ipc-manager dashboard --validator=validator-3 --interval=10
```

## Display Format

```
╔═══════════════════════════════════════════════════════════════════════╗
║               IPC SUBNET LIVE MONITOR - validator-1                   ║
║  Subnet: /r314159/t410fa...    Refresh: 3s    Uptime: 2h 34m         ║
╚═══════════════════════════════════════════════════════════════════════╝

┌─ BLOCK PRODUCTION ────────────────────────────────────────────────────┐
│ Height: 18,453  (+127 in 1m)    Avg Block Time: 0.71s    Rate: 1.4/s │
│ Status: ●●●●● PRODUCING        Last Block: 2s ago                     │
└───────────────────────────────────────────────────────────────────────┘

┌─ PARENT FINALITY ─────────────────────────────────────────────────────┐
│ Subnet: 3,116,450  Parent Chain: 3,116,465  Lag: 15 blocks (12s)     │
│ Status: ✓ SYNCING              Last Commit: 18s ago                   │
└───────────────────────────────────────────────────────────────────────┘

┌─ NETWORK HEALTH ──────────────────────────────────────────────────────┐
│ CometBFT Peers: 2/2 ✓    Libp2p Peers: 2/2 ✓    RPC: ✓ RESPONSIVE    │
└───────────────────────────────────────────────────────────────────────┘

┌─ MEMPOOL STATUS ──────────────────────────────────────────────────────┐
│ Transactions: 94/10000 (0.9%)  Size: 48KB/1GB    Status: ✓ HEALTHY   │
└───────────────────────────────────────────────────────────────────────┘

┌─ CHECKPOINT ACTIVITY (Last 5 min) ────────────────────────────────────┐
│ Signatures: 12 broadcast, 10 success, 2 mempool collision            │
│ Success Rate: 83%  Last: 23s ago                                      │
└───────────────────────────────────────────────────────────────────────┘

┌─ ERROR SUMMARY (Last 5 min) ──────────────────────────────────────────┐
│ ⚠ Bottom-up Checkpoint:  2  (mempool full)                            │
│ ● Parent Finality:       0                                            │
│ ● Network/P2P:           0                                            │
│ ● Consensus:             0                                            │
│ ● RPC/API:               1  (timeout)                                 │
│ ● Other:                 0                                            │
│ Total Errors: 3          Error Rate: 0.6/min                          │
└───────────────────────────────────────────────────────────────────────┘

┌─ RECENT EVENTS ───────────────────────────────────────────────────────┐
│ 18:42:15  ✓ Checkpoint signature broadcast (tx: 9268473A...)         │
│ 18:42:03  ✓ Parent finality committed (height: 3116450)              │
│ 18:41:58  ⚠ Mempool full error (recovered)                           │
│ 18:41:45  ✓ Block 18453 produced (0.68s)                             │
│ 18:41:30  ✓ Checkpoint signature broadcast (tx: D43F97EF...)         │
└───────────────────────────────────────────────────────────────────────┘

Press 'q' to quit, 'r' to reset counters, 'h' for help
```

## Status Indicators

### Color Coding

- **Green (✓)**: Normal operation
- **Yellow (⚠)**: Warning condition
- **Red (✗)**: Error condition
- **Blue (●)**: No issues detected

### Thresholds

**Block Production:**
- ✓ Green: 30+ blocks/minute
- ⚠ Yellow: 10-29 blocks/minute
- ✗ Red: <10 blocks/minute

**Parent Finality Lag:**
- ✓ Green: ≤30 blocks behind
- ⚠ Yellow: 31-100 blocks behind
- ✗ Red: >100 blocks behind

**Mempool Utilization:**
- ✓ Green: <50% full
- ⚠ Yellow: 50-80% full
- ✗ Red: >80% full

**Network Peers:**
- ✓ Green: All expected peers connected
- ⚠ Yellow: Some peers missing
- ✗ Red: No peers connected

## Interactive Controls

### Keyboard Commands

- **`q` or `Q`**: Quit the dashboard
- **`r` or `R`**: Reset error counters and recent events
- **`Ctrl+C`**: Exit immediately

## Error Categories

### Bottom-up Checkpoint Errors
Issues related to checkpoint signature creation and broadcasting:
- Mempool full
- Broadcast failures
- Signature creation errors

### Parent Finality Errors
Problems with syncing parent chain state:
- Vote gossip failures
- Proposal errors
- Sync issues

### Network/P2P Errors
Peer-to-peer communication problems:
- Peer connection failures
- Gossip protocol issues
- Libp2p errors

### Consensus Errors
CometBFT consensus issues:
- Round timeout
- Proposal failures
- Voting errors

### RPC/API Errors
Remote procedure call failures:
- Connection timeouts
- HTTP errors
- JSON-RPC failures

## Metrics Explained

### Blocks Per Minute
Number of blocks produced in the last 60 seconds. This metric updates every minute.

### Mempool Size
Number of pending transactions waiting to be included in blocks. Should stay well below the maximum (10,000).

### Finality Lag
Difference between parent chain height and the height the subnet has finalized. Lower is better; high lag indicates parent finality sync issues.

### Checkpoint Signatures
Count of bottom-up checkpoint signatures broadcast in recent log samples. Active checkpointing will show regular activity here.

### Error Rate
Average errors per minute over the last 5 minutes. A low, stable rate is normal; spikes indicate issues.

## Tips

### Troubleshooting

1. **High Error Rate**
   - Check the error categories to identify the source
   - Use the `info` command for detailed diagnostics
   - Review full logs with `./ipc-manager logs validator-1`

2. **High Finality Lag**
   - Verify parent RPC connectivity
   - Check for parent finality errors
   - Use `watch-finality` for detailed tracking

3. **Low Block Production**
   - Check validator connectivity
   - Verify consensus health
   - Use `watch-blocks` for detailed block timing

4. **Mempool Full**
   - Increase mempool size if persistent
   - Check for checkpoint spam
   - Verify transactions are being processed

### Performance

The dashboard executes multiple SSH commands and API calls every refresh interval. Consider:
- Using a longer refresh interval (5-10s) to reduce load
- Running it on a management machine, not production nodes
- Monitoring only during active troubleshooting

## Comparison with Other Commands

### vs. `info` Command
- **`info`**: One-time snapshot with detailed diagnostics
- **`dashboard`**: Continuous real-time monitoring

### vs. `watch-blocks`
- **`watch-blocks`**: Focused on block production only
- **`dashboard`**: Comprehensive multi-metric view

### vs. `watch-finality`
- **`watch-finality`**: Detailed parent finality tracking
- **`dashboard`**: Broader overview including finality

### Use Cases

Use **`dashboard`** when you want:
- General health monitoring
- Quick at-a-glance status
- Real-time error tracking
- Comprehensive system overview

Use **`info`** when you want:
- Detailed diagnostics
- Configuration verification
- Setup validation

Use **`watch-blocks`** when you need:
- Precise block timing data
- Performance tuning metrics
- Block production debugging

Use **`watch-finality`** when tracking:
- Specific parent epoch targets
- Parent finality sync progress
- Cross-chain message processing

## Technical Details

### Data Sources

1. **CometBFT RPC**
   - `/status` - Block height, catching up status
   - `/net_info` - Peer connections
   - `/num_unconfirmed_txs` - Mempool status

2. **Parent Chain RPC**
   - `eth_blockNumber` - Current parent chain height

3. **Node Logs**
   - `~/.ipc-node/logs/*.log` - Error tracking, events

4. **SSH Execution**
   - Process status checks
   - Port listening verification

### Refresh Cycle

Each refresh cycle:
1. Fetches metrics from validator node
2. Queries parent chain RPC
3. Parses recent log entries
4. Categorizes and counts errors
5. Calculates derived metrics
6. Redraws the entire display

Default cycle time: 3 seconds

### Resource Usage

- **Network**: Multiple SSH connections per cycle
- **CPU**: Minimal (log parsing, JSON processing)
- **Memory**: <10MB for dashboard process

## Alias Command

The dashboard is also available as `monitor`:

```bash
./ipc-manager monitor
```

Both commands are identical and can be used interchangeably.

