# Quick Start Guide

## 1. Install Prerequisites

```bash
# macOS (requires Bash 4.0+ and yq)
brew install bash yq

# Linux (yq only, bash 4.0+ usually pre-installed)
wget https://github.com/mikefarah/yq/releases/latest/download/yq_linux_amd64 -O /usr/local/bin/yq
chmod +x /usr/local/bin/yq
```

**macOS Note**: You'll need to run the script with the newer bash:
```bash
/usr/local/bin/bash ipc-subnet-manager.sh --help
```

## 2. Configure Your Subnet

Edit `ipc-subnet-config.yml`:

```bash
cd /Users/philip/github/ipc/scripts/ipc-subnet-manager
vi ipc-subnet-config.yml
```

**Update these fields:**
- `subnet.id` - Your subnet ID from creation
- `validators[].ip` - IP addresses of your validators
- `validators[].ssh_user` - Your SSH user (default: philip)
- `paths.ipc_binary` - Path to ipc-cli on remote hosts

## 3. Test Connectivity

```bash
# Test SSH to all validators
for ip in 34.73.187.192 35.237.175.224 34.75.205.89; do
  echo "Testing $ip..."
  ssh philip@$ip "sudo su - ipc -c 'whoami'"
done
```

## 4. Run Health Check (Optional)

If you have existing nodes running, check their health:

```bash
./ipc-subnet-manager.sh check
```

## 5. Initialize Subnet

⚠️ **WARNING**: This will destroy all existing data!

```bash
# Dry run first to see what will happen
./ipc-subnet-manager.sh init --dry-run

# Actually do it
./ipc-subnet-manager.sh init
```

## 6. Monitor Progress

```bash
# Check health
./ipc-subnet-manager.sh check

# View logs from validator-1
./ipc-subnet-manager.sh logs validator-1
```

## Common Commands

```bash
# Initialize subnet from scratch
./ipc-subnet-manager.sh init

# Update configs without destroying data
./ipc-subnet-manager.sh update-config

# Health check
./ipc-subnet-manager.sh check

# Restart all nodes
./ipc-subnet-manager.sh restart

# View logs
./ipc-subnet-manager.sh logs validator-1

# Help
./ipc-subnet-manager.sh --help
```

## Troubleshooting

### Can't SSH to validators
```bash
# Set up SSH keys
ssh-copy-id philip@34.73.187.192
```

### yq command not found
```bash
# macOS
brew install yq
```

### Script shows permission denied
```bash
chmod +x ipc-subnet-manager.sh
```

### Validators won't start
```bash
# Check logs for errors
./ipc-subnet-manager.sh logs validator-1

# Try manual start on one node
ssh philip@34.73.187.192 "sudo su - ipc -c '/home/ipc/ipc/target/release/ipc-cli node start'"
```

## Expected Timeline

| Step | Time |
|------|------|
| Pre-flight checks | ~10s |
| Stop nodes | ~5s |
| Backup data | ~30s |
| Wipe data | ~5s |
| Initialize primary | ~30s |
| Initialize secondaries | ~60s |
| Collect peer info | ~15s |
| Update configs | ~10s |
| Set federated power | ~30s |
| Start nodes | ~15s |
| Health checks | ~20s |
| **Total** | **~4-5 minutes** |

## What to Watch For

✅ **Good Signs:**
- All health checks pass (green checkmarks)
- Block height > 0 and increasing
- CometBFT peers = N-1 (e.g., 2/2 for 3 validators)
- No recent errors in logs

❌ **Bad Signs:**
- Process not running
- Block height stuck at 0
- No CometBFT peers
- Errors about "lookback" or "failed to get Tendermint status"

## Next Steps

After successful initialization:

1. **Fund the subnet wallet:**
```bash
ipc-cli cross-msg fund --subnet $SUBNET_ID --from $WALLET --to $SUBNET_WALLET --amount 1
```

2. **Monitor parent finality:**
```bash
./ipc-subnet-manager.sh logs validator-1 | grep ParentFinality
```

3. **Check balances:**
```bash
# On subnet
curl -X POST http://validator-ip:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_getBalance","params":["0xYOUR_ADDRESS","latest"],"id":1}'
```

