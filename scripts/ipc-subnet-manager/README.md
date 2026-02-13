# IPC Subnet Manager

A robust script to manage IPC validator nodes with config-driven automation, supporting initialization, updates, and health checks.

## Features

- **Nuclear Init**: Completely wipe and reinitialize all validators from scratch
- **Config Updates**: Update node configurations without destroying data
- **Health Checks**: Comprehensive validation of validator health
- **Automated Peering**: Automatic CometBFT and libp2p peer mesh configuration
- **Federated Power**: Automatic validator power setup for federated subnets
- **Logs Streaming**: Easy access to validator logs

## Prerequisites

### Local Machine
- `bash` 4.0+ (⚠️ macOS ships with Bash 3.2, you need to upgrade)
- `yq` - YAML processor ([install](https://github.com/mikefarah/yq))
- `ssh` with key-based authentication to all validators
- `scp` for file transfers

```bash
# macOS - Install both bash and yq
brew install bash yq

# Linux - Install yq (bash 4.0+ usually pre-installed)
wget https://github.com/mikefarah/yq/releases/latest/download/yq_linux_amd64 -O /usr/local/bin/yq
chmod +x /usr/local/bin/yq
```

**macOS Users**: After installing bash via Homebrew, run the script with:
```bash
/usr/local/bin/bash ipc-subnet-manager.sh <command>
# Or add an alias to your ~/.zshrc or ~/.bash_profile:
alias ipc-manager='/usr/local/bin/bash /path/to/ipc-subnet-manager.sh'
```

### Remote Validators
- Ubuntu/Debian-based Linux
- `ipc-cli` binary installed
- `cometbft` binary in PATH
- User with sudo access (default: `philip`)
- IPC user (default: `ipc`)
- SSH key-based authentication configured

## Installation

1. Clone or copy the `ipc-subnet-manager` directory:
```bash
cd /path/to/ipc/scripts
ls ipc-subnet-manager/
# ipc-subnet-manager.sh  ipc-subnet-config.yml  lib/  README.md
```

2. Make the script executable:
```bash
chmod +x ipc-subnet-manager/ipc-subnet-manager.sh
```

3. Configure your subnet (see Configuration section)

## Configuration

Edit `ipc-subnet-config.yml` to match your setup:

```yaml
subnet:
  id: "/r314159/t410f..."          # Your subnet ID
  parent_rpc: "https://..."         # Parent chain RPC
  parent_chain_id: "/r314159"       # Parent chain ID

validators:
  - name: "validator-1"
    ip: "34.73.187.192"
    ssh_user: "philip"
    ipc_user: "ipc"
    role: "primary"
  # ... more validators

paths:
  ipc_binary: "/home/ipc/ipc/target/release/ipc-cli"
  node_home: "/home/ipc/.ipc-node"
```

### Environment Variable Overrides

You can override any config value with environment variables:

```bash
# Override subnet ID
export IPC_SUBNET_ID="/r314159/t410f..."

# Override validator IPs
export IPC_VALIDATORS_0_IP="10.0.0.1"
export IPC_VALIDATORS_1_IP="10.0.0.2"

# Override parent RPC
export IPC_PARENT_RPC="https://custom-rpc.example.com"
```

## Usage

### Initialize Subnet (Nuclear Option)

⚠️ **WARNING**: This will destroy all existing data and reinitialize from scratch!

```bash
./ipc-subnet-manager.sh init

# Skip confirmation prompt
./ipc-subnet-manager.sh init --yes
```

**What it does:**
1. Pre-flight checks (SSH, binaries, config)
2. Stops all running nodes
3. Creates timestamped backups
4. Wipes all node data
5. Initializes primary validator
6. Initializes secondary validators with primary's peer info
7. Updates all configs with full peer mesh
8. Configures CometBFT persistent peers
9. Configures libp2p static addresses
10. Sets validator key configuration
11. Sets federated power for all validators
12. Starts all nodes in order
13. Runs health checks

### Update Configuration

Update node configs without destroying data (useful after manual changes or to fix peer connectivity):

```bash
./ipc-subnet-manager.sh update-config
```

**What it does:**
1. Collects current peer info from all validators
2. Regenerates CometBFT and libp2p peer configs
3. Updates config files on all nodes
4. Restarts nodes

### Health Check

Run comprehensive health checks on all validators:

```bash
./ipc-subnet-manager.sh check
```

**Checks:**
- ✓ Process running
- ✓ Ports listening (26656, 26655, 8545)
- ✓ CometBFT peer count
- ✓ Block height progression
- ✓ Recent errors in logs

**Example Output:**
```
========================================
  Health Check
========================================

  -- Checking validator-1
[✓] Process running
[✓] Ports listening (3/3)
[✓] CometBFT peers: 2/2
[✓] Block height: 1542
[✓] No recent errors

  -- Checking validator-2
[✓] Process running
[✓] Ports listening (3/3)
[✓] CometBFT peers: 2/2
[✓] Block height: 1542
[✓] No recent errors

[SUCCESS] ✓ All validators are healthy!
```

### Restart Nodes

Gracefully restart all validator nodes:

```bash
./ipc-subnet-manager.sh restart

# Skip confirmation
./ipc-subnet-manager.sh restart --yes
```

### View Logs

Tail logs from a specific validator:

```bash
./ipc-subnet-manager.sh logs validator-1

# This will show filtered logs containing:
# - ParentFinality events
# - ERROR messages
# - WARN messages
```

Press `Ctrl+C` to stop tailing.

### Dry Run Mode

Preview what the script would do without making changes:

```bash
./ipc-subnet-manager.sh init --dry-run
./ipc-subnet-manager.sh update-config --dry-run
```

## Troubleshooting

### SSH Connection Issues

1. **Test SSH connectivity manually:**
```bash
ssh philip@34.73.187.192 "sudo su - ipc -c 'whoami'"
```

2. **Ensure key-based auth is set up:**
```bash
ssh-copy-id philip@34.73.187.192
```

3. **Check sudo permissions:**
```bash
ssh philip@34.73.187.192 "sudo -l"
```

### Validator Won't Start

1. **Check if process is hung:**
```bash
ssh philip@validator-ip "ps aux | grep ipc-cli"
```

2. **Check logs for errors:**
```bash
./ipc-subnet-manager.sh logs validator-1
```

3. **Manually stop and restart:**
```bash
ssh philip@validator-ip "sudo su - ipc -c 'pkill -f ipc-cli'"
ssh philip@validator-ip "sudo su - ipc -c '/home/ipc/ipc/target/release/ipc-cli node start'"
```

### No Peer Connectivity

1. **Check firewall rules:**
```bash
# Port 26656 (CometBFT P2P)
# Port 26655 (libp2p)
# Should be open for all validator IPs
```

2. **Verify peer info:**
```bash
ssh philip@validator-ip "sudo su - ipc -c 'cat ~/.ipc-node/peer-info.json'"
```

3. **Update configs:**
```bash
./ipc-subnet-manager.sh update-config
```

### Parent Finality Not Advancing

1. **Check parent RPC connectivity:**
```bash
curl -X POST https://api.calibration.node.glif.io/rpc/v1 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"Filecoin.ChainHead","params":[],"id":1}'
```

2. **Check for lookback errors:**
```bash
./ipc-subnet-manager.sh logs validator-1 | grep "lookback"
```

3. **Verify validator voting power:**
```bash
# From a validator
ssh philip@validator-ip "sudo su - ipc -c 'ipc-cli subnet list-validators --subnet /r314159/t410f...'"
```

### yq Not Found

```bash
# macOS
brew install yq

# Linux
sudo wget https://github.com/mikefarah/yq/releases/latest/download/yq_linux_amd64 -O /usr/local/bin/yq
sudo chmod +x /usr/local/bin/yq
```

## File Structure

```
ipc-subnet-manager/
├── ipc-subnet-manager.sh          # Main script
├── ipc-subnet-config.yml          # Configuration file
├── lib/
│   ├── colors.sh                  # Color output utilities
│   ├── ssh.sh                     # SSH helper functions
│   ├── config.sh                  # Config parsing and management
│   └── health.sh                  # Health checks and node operations
└── README.md                      # This file
```

## Safety Features

- **Lock file**: Prevents concurrent executions of destructive operations
- **Confirmation prompts**: Required for destructive operations (can skip with `--yes`)
- **Automatic backups**: Created before wiping node data
- **Dry-run mode**: Preview actions without executing
- **SSH timeout**: 10-second timeout to prevent hanging
- **Comprehensive validation**: Pre-flight checks before any operation

## Known Limitations

1. **16-hour parent lookback limit**: If the subnet falls >16 hours behind, it cannot sync with public Calibration RPC
2. **No automatic recovery**: Script won't automatically fix chain halt or consensus issues
3. **Single subnet support**: Currently manages one subnet at a time
4. **No monitoring integration**: No built-in Prometheus/alerting (coming soon)

## Future Enhancements

- [ ] Binary deployment automation
- [ ] Multi-subnet support
- [ ] Automatic recovery from common issues
- [ ] Monitoring integration (Prometheus)
- [ ] Alerting via webhooks
- [ ] Cloud provider integration
- [ ] Auto-provisioning of VMs

## Contributing

When making changes:
1. Test with `--dry-run` first
2. Update this README
3. Add appropriate logging
4. Handle errors gracefully

## License

Same as IPC project (MIT/Apache-2.0)

