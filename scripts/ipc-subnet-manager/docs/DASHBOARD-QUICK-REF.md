# Dashboard Quick Reference

## Launch Dashboard

```bash
# Basic usage (monitor validator-1, 3s refresh)
./ipc-manager dashboard

# Specific validator
./ipc-manager dashboard --validator=validator-2

# Custom refresh rate
./ipc-manager dashboard --interval=5

# Combined
./ipc-manager dashboard --validator=validator-3 --interval=10

# Alias command
./ipc-manager monitor  # Same as dashboard
```

## Keyboard Controls

| Key | Action |
|-----|--------|
| `q` or `Q` | Quit dashboard |
| `r` or `R` | Reset error counters |
| `Ctrl+C` | Force quit |

## Dashboard Panels

### 1. Block Production
- **Height**: Current blockchain height
- **+N in 1m**: Blocks produced in last minute
- **Status**: Production health (⚠ if <30 blocks/min)

### 2. Parent Finality
- **Subnet**: What parent height subnet has finalized
- **Parent Chain**: Actual parent blockchain height
- **Lag**: Difference in blocks (⚠ if >30, ✗ if >100)

### 3. Network Health
- **CometBFT Peers**: P2P consensus connections (expected 2/2 for 3 validators)
- **Libp2p Peers**: IPC vote gossip connections
- **RPC**: Local RPC endpoint status

### 4. Mempool Status
- **Transactions**: Pending tx count / max capacity
- **Size**: Memory usage (⚠ if >50%, ✗ if >80%)
- **Status**: Overall mempool health

### 5. Checkpoint Activity
- **Signatures**: Number broadcast in recent logs
- **Last**: Time since last signature

### 6. Error Summary
Categorized error counts from recent logs:
- **Bottom-up Checkpoint**: Signature/mempool errors
- **Parent Finality**: Sync/vote errors
- **Network/P2P**: Connection/gossip errors
- **Consensus**: CometBFT timeout/round errors
- **RPC/API**: HTTP/timeout errors
- **Other**: Uncategorized errors

### 7. Recent Events
Last 5 significant events with timestamps

## Status Colors

| Symbol | Meaning | When Used |
|--------|---------|-----------|
| ✓ (Green) | Healthy | Normal operation |
| ⚠ (Yellow) | Warning | Degraded but functional |
| ✗ (Red) | Error | Requires attention |
| ● (Blue) | Info | No issues detected |

## Thresholds

### Block Production
- ✓ ≥30 blocks/minute
- ⚠ 10-29 blocks/minute
- ✗ <10 blocks/minute

### Parent Finality Lag
- ✓ ≤30 blocks behind
- ⚠ 31-100 blocks behind
- ✗ >100 blocks behind

### Mempool Utilization
- ✓ <50% full
- ⚠ 50-80% full
- ✗ >80% full

### Network Peers
- ✓ All expected peers connected
- ⚠ Some peers missing
- ✗ No peers connected

## Common Issues

### Problem: Metrics show 0
**Solution**: Check if validator is running
```bash
./ipc-manager check
./ipc-manager info
```

### Problem: High error rate
**Solution**: Check error categories
- Look at which category has most errors
- Use targeted command for details:
  - `./ipc-manager logs validator-1` for full logs
  - `./ipc-manager watch-finality` for finality issues
  - `./ipc-manager watch-blocks` for block production

### Problem: High finality lag
**Solution**: Parent finality sync issue
```bash
# Monitor finality progress
./ipc-manager watch-finality

# Check detailed subnet info
./ipc-manager info

# Review logs for finality errors
./ipc-manager logs validator-1 | grep -i finality
```

### Problem: Mempool full
**Solution**: Increase mempool size or reduce checkpoint frequency
```bash
# Check current mempool (from dashboard)
# If persistently >80%, increase size in CometBFT config
# Or adjust bottom_up_check_period in subnet config
```

### Problem: Low block production
**Solution**: Check consensus and connectivity
```bash
# Detailed block timing
./ipc-manager watch-blocks

# Check peers and status
./ipc-manager info

# Verify all validators online
./ipc-manager check
```

## Tips

### Performance
- Use longer refresh interval (5-10s) to reduce SSH load
- Monitor from management machine, not production nodes
- Dashboard uses ~1-2s per cycle for data collection

### Workflow
1. **Initial setup**: Use `check` and `info` commands
2. **Ongoing monitoring**: Use `dashboard` for real-time view
3. **Troubleshooting**: Use `watch-*` and `logs` commands
4. **Quick checks**: Use `dashboard` with longer interval

### Best Practices
- Keep dashboard running during critical operations
- Reset counters (`r` key) when starting new test
- Monitor during `cross-msg fund` operations
- Track checkpoint activity and errors

## Integration

### With Other Commands

```bash
# Initial diagnostics
./ipc-manager info

# Start monitoring
./ipc-manager dashboard

# In another terminal: detailed tracking
./ipc-manager watch-finality --target-epoch=3116500
./ipc-manager watch-blocks

# When issues detected: review logs
./ipc-manager logs validator-1 | grep ERROR
```

### With tmux

```bash
# Create tmux session with multiple panes
tmux new-session -d -s ipc-monitoring
tmux split-window -h
tmux split-window -v

# Pane 0: Dashboard
tmux send-keys -t 0 './ipc-manager dashboard' Enter

# Pane 1: Watch finality
tmux send-keys -t 1 './ipc-manager watch-finality' Enter

# Pane 2: Watch blocks
tmux send-keys -t 2 './ipc-manager watch-blocks' Enter

# Attach to session
tmux attach-session -t ipc-monitoring
```

## Comparison Matrix

| Command | Use When | Refresh | Scope |
|---------|----------|---------|-------|
| `dashboard` | General monitoring | Live (3s) | All metrics |
| `info` | Setup/diagnostics | One-time | Detailed checks |
| `watch-blocks` | Block performance | Live (2s) | Block timing only |
| `watch-finality` | Parent sync | Live (5s) | Finality only |
| `check` | Health validation | One-time | Connection/status |
| `logs` | Deep debugging | Live (tail) | Raw logs |

## Exit & Cleanup

The dashboard automatically:
- Shows cursor on exit
- Clears screen
- Releases resources
- Works with `q`, `Ctrl+C`, or terminal close

No manual cleanup required!

