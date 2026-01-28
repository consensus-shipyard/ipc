# IPC Subnet Manager - Technical Structure

## System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     IPC Subnet Manager                           │
│                    (Your Local Machine)                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌───────────────┐        ┌─────────────────────────────────┐  │
│  │  ipc-manager  │───────▶│  ipc-subnet-manager.sh          │  │
│  │  (wrapper)    │        │  - Command routing              │  │
│  └───────────────┘        │  - Lock management              │  │
│                            │  - Argument parsing             │  │
│                            └──────────┬──────────────────────┘  │
│                                       │                          │
│                    ┌──────────────────┼──────────────────────┐  │
│                    │                  │                      │  │
│         ┌──────────▼──────┐  ┌───────▼───────┐  ┌──────────▼──┐│
│         │   lib/colors.sh │  │ lib/config.sh │  │ lib/ssh.sh  ││
│         │  - log_error    │  │ - load_config │  │ - ssh_exec  ││
│         │  - log_success  │  │ - get_config  │  │ - scp_*     ││
│         │  - log_check    │  │ - extract_*   │  │ - test_ssh  ││
│         └─────────────────┘  └───────────────┘  └─────────────┘│
│                                       │                          │
│                            ┌──────────▼──────────────────────┐  │
│                            │      lib/health.sh              │  │
│                            │  - start_all_nodes()            │  │
│                            │  - stop_all_nodes()             │  │
│                            │  - initialize_*()               │  │
│                            │  - check_validator_health()     │  │
│                            └─────────────────────────────────┘  │
│                                                                   │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │           ipc-subnet-config.yml                           │  │
│  │  - Subnet ID, parent RPC, chain ID                        │  │
│  │  - Validator IPs, users, roles                            │  │
│  │  - Network ports                                           │  │
│  │  - Paths to binaries                                       │  │
│  │  - Init settings                                           │  │
│  └───────────────────────────────────────────────────────────┘  │
└───────────────────────────┬───────────────────────────────────┘
                            │ SSH/SCP
                            │
        ┌───────────────────┼───────────────────────┐
        │                   │                       │
        ▼                   ▼                       ▼
┌───────────────┐   ┌───────────────┐     ┌───────────────┐
│  Validator 1  │   │  Validator 2  │     │  Validator 3  │
│  (Primary)    │   │  (Secondary)  │     │  (Secondary)  │
├───────────────┤   ├───────────────┤     ├───────────────┤
│ 34.73.187.192 │   │35.237.175.224 │     │ 34.75.205.89  │
├───────────────┤   ├───────────────┤     ├───────────────┤
│ ~/.ipc-node/  │   │ ~/.ipc-node/  │     │ ~/.ipc-node/  │
│  ├─cometbft/  │   │  ├─cometbft/  │     │  ├─cometbft/  │
│  │  └─config/ │   │  │  └─config/ │     │  │  └─config/ │
│  ├─fendermint/│   │  ├─fendermint/│     │  ├─fendermint/│
│  │  ├─config/ │   │  │  ├─config/ │     │  │  ├─config/ │
│  │  └─validator│   │  │  └─validator│     │  │  └─validator│
│  │     .sk    │   │  │     .sk    │     │  │     .sk    │
│  └─logs/      │   │  └─logs/      │     │  └─logs/      │
└───────────────┘   └───────────────┘     └───────────────┘
        │                   │                       │
        └───────────────────┴───────────────────────┘
                    P2P Mesh Network
               (CometBFT + libp2p gossip)
```

## Command Flow

### `init` Command Flow

```
./ipc-manager init
    │
    ├─▶ 1. Check Bash version (4.0+)
    │
    ├─▶ 2. Load config (YAML parsing with yq)
    │
    ├─▶ 3. PRE-FLIGHT CHECKS
    │   ├─▶ Check yq, ssh, scp
    │   ├─▶ Validate config
    │   └─▶ Test SSH to all validators
    │
    ├─▶ 4. STOP ALL NODES
    │   └─▶ SSH: pkill -f "ipc-cli node start"
    │
    ├─▶ 5. BACKUP
    │   └─▶ SSH: cp -r ~/.ipc-node ~/.ipc-node.backup.{timestamp}
    │
    ├─▶ 6. WIPE
    │   └─▶ SSH: rm -rf ~/.ipc-node
    │
    ├─▶ 7. INITIALIZE PRIMARY (validator-1)
    │   ├─▶ Generate node-init.yml
    │   ├─▶ SCP node-init.yml to validator
    │   ├─▶ SSH: ipc-cli node init --config node-init.yml
    │   └─▶ Extract peer-info.json
    │
    ├─▶ 8. INITIALIZE SECONDARIES (validator-2, validator-3)
    │   ├─▶ Generate node-init.yml (with primary peer)
    │   ├─▶ SCP node-init.yml to validator
    │   └─▶ SSH: ipc-cli node init --config node-init.yml
    │
    ├─▶ 9. COLLECT PEER INFO
    │   ├─▶ CometBFT node IDs: cometbft show-node-id
    │   ├─▶ Libp2p peer IDs: grep logs for local_peer_id
    │   └─▶ Validator pubkeys: cat validator.sk
    │
    ├─▶ 10. UPDATE CONFIGS (full mesh)
    │   ├─▶ cometbft/config.toml
    │   │   └─▶ persistent_peers = "node1@ip1,node2@ip2"
    │   ├─▶ fendermint/config/default.toml
    │   │   ├─▶ external_addresses = ["/ip4/MY_IP/tcp/26655/p2p/MY_ID"]
    │   │   └─▶ static_addresses = ["/ip4/PEER1_IP/...", "/ip4/PEER2_IP/..."]
    │   └─▶ Add [validator_key] section
    │
    ├─▶ 11. SET FEDERATED POWER
    │   └─▶ SSH (primary): ipc-cli subnet set-federated-power
    │                       --validator-pubkeys pubkey1,pubkey2,pubkey3
    │                       --validator-power 1
    │
    ├─▶ 12. START ALL NODES
    │   ├─▶ Start primary first
    │   ├─▶ Wait 5 seconds
    │   └─▶ Start secondaries
    │
    └─▶ 13. HEALTH CHECKS
        ├─▶ Process running?
        ├─▶ Ports listening?
        ├─▶ CometBFT peers = N-1?
        ├─▶ Block height > 0?
        └─▶ Recent errors?
```

## File Operations

### Config Files Modified by Script

```
Validator Node: ~/.ipc-node/
│
├── cometbft/
│   └── config/
│       └── config.toml
│           Modified: persistent_peers = "..."
│
└── fendermint/
    └── config/
        └── default.toml
            Modified:
              - [resolver.connection].external_addresses
              - [resolver.discovery].static_addresses
            Added:
              - [validator_key] section
```

### Generated Files

```
Local Temp:
  /tmp/node-init-validator-1.yml  (deleted after use)
  /tmp/node-init-validator-2.yml  (deleted after use)
  /tmp/node-init-validator-3.yml  (deleted after use)

Remote:
  /home/ipc/node-init.yml         (kept for reference)

Lock:
  /tmp/ipc-subnet-manager.lock    (created/deleted automatically)
```

## Data Flow

### Configuration Loading
```
ipc-subnet-config.yml
    │
    ├─▶ yq eval '.subnet.id'         ──▶  $subnet_id
    ├─▶ yq eval '.validators[0].ip'  ──▶  $ip
    ├─▶ yq eval '.validators[0].role' ──▶  $role
    │
    └─▶ Environment overrides:
        $IPC_SUBNET_ID               ──▶  Overrides config value
        $IPC_VALIDATORS_0_IP         ──▶  Overrides validator IP
```

### Peer Information Collection
```
Validator Node
    │
    ├─▶ cometbft show-node-id
    │   └─▶ "9bb7ae0c618788f9398a47163e9d2b488ea7e296"
    │       └─▶ COMETBFT_PEERS[0] = "9bb7...@34.73.187.192:26656"
    │
    ├─▶ grep 'local_peer_id' logs/*.log
    │   └─▶ "16Uiu2HAkytjpBRaCyjVDAoEZ9K5U2fDiLPK5KripKrzQXs5PpNsh"
    │       └─▶ LIBP2P_PEERS[0] = "/ip4/34.73.187.192/tcp/26655/p2p/16Uiu2..."
    │
    └─▶ cat fendermint/validator.sk
        └─▶ "0xABCD1234..."
            └─▶ VALIDATOR_PUBKEYS[0] = "ABCD1234..." (without 0x)
```

## SSH Operations

### SSH Command Wrapping
```
Local: ./ipc-manager check
    │
    └─▶ ssh philip@34.73.187.192 "sudo su - ipc -c 'COMMAND'"
            │
            └─▶ Remote execution as 'ipc' user
                │
                └─▶ Result returned to local script
```

### File Transfer
```
Local: generate_node_init_yml()
    │
    ├─▶ Create temp file: /tmp/node-init-validator-1.yml
    │
    └─▶ scp_to_host()
        ├─▶ scp /tmp/node-init-validator-1.yml philip@ip:/tmp/
        └─▶ ssh philip@ip "sudo mv /tmp/node-init-validator-1.yml /home/ipc/node-init.yml"
            └─▶ ssh philip@ip "sudo chown ipc:ipc /home/ipc/node-init.yml"
```

## Error Handling

```
Command Execution
    │
    ├─▶ SSH Timeout (10s)
    │   └─▶ log_error "Connection timeout"
    │
    ├─▶ Permission Denied
    │   └─▶ log_error "SSH keys not configured"
    │
    ├─▶ Command Failed
    │   └─▶ log_error "Operation failed"
    │       └─▶ Show output
    │
    └─▶ Lock File Exists
        └─▶ log_error "Another instance running"
            └─▶ Exit 1
```

## Health Check Logic

```
check_validator_health()
    │
    ├─▶ Process Running?
    │   └─▶ pgrep -f "ipc-cli node start"
    │       ├─▶ Found    ──▶  ✓ Process running
    │       └─▶ Not found ──▶  ✗ Process not running
    │
    ├─▶ Ports Listening?
    │   └─▶ netstat -tuln | grep -E ':(26656|26655|8545)'
    │       ├─▶ 3/3 ──▶  ✓ Ports listening
    │       └─▶ <3  ──▶  ✗ Ports not listening
    │
    ├─▶ CometBFT Peers?
    │   └─▶ curl localhost:26657/net_info | grep n_peers
    │       ├─▶ count >= N-1 ──▶  ✓ CometBFT peers: 2/2
    │       └─▶ count < N-1  ──▶  ✗ CometBFT peers: 0/2
    │
    ├─▶ Block Height?
    │   └─▶ curl localhost:26657/status | grep latest_block_height
    │       ├─▶ height > 0 ──▶  ✓ Block height: 1542
    │       └─▶ height = 0 ──▶  ✗ Block height: 0
    │
    └─▶ Recent Errors?
        └─▶ tail -100 logs/*.log | grep -i ERROR
            ├─▶ Empty ──▶  ✓ No recent errors
            └─▶ Found ──▶  ✗ Recent errors found
```

## State Management

### Global State
```bash
# Validators array
VALIDATORS=("validator-1" "validator-2" "validator-3")

# Peer info (associative arrays)
COMETBFT_PEERS[0]="9bb7...@34.73.187.192:26656"
COMETBFT_PEERS[1]="0fe9...@35.237.175.224:26656"
COMETBFT_PEERS[2]="a576...@34.75.205.89:26656"

LIBP2P_PEERS[0]="/ip4/34.73.187.192/tcp/26655/p2p/16Uiu2..."
LIBP2P_PEERS[1]="/ip4/35.237.175.224/tcp/26655/p2p/16Uiu2..."
LIBP2P_PEERS[2]="/ip4/34.75.205.89/tcp/26655/p2p/16Uiu2..."

VALIDATOR_PUBKEYS[0]="ABCD1234..."
VALIDATOR_PUBKEYS[1]="EFGH5678..."
VALIDATOR_PUBKEYS[2]="IJKL9012..."
```

## Future Expansion Points

### Modular Design Allows:
```
1. Binary Deployment
   └─▶ lib/deploy.sh (new)
       ├─▶ download_binaries()
       ├─▶ verify_checksums()
       └─▶ install_binaries()

2. Monitoring Integration
   └─▶ lib/monitoring.sh (new)
       ├─▶ export_prometheus_metrics()
       ├─▶ send_webhook_alert()
       └─▶ log_to_loki()

3. Multi-Subnet Support
   └─▶ Multiple config files
       ├─▶ ipc-subnet-config-subnet1.yml
       ├─▶ ipc-subnet-config-subnet2.yml
       └─▶ ./ipc-manager --subnet subnet1 init

4. Automatic Recovery
   └─▶ lib/recovery.sh (new)
       ├─▶ detect_chain_halt()
       ├─▶ fix_peer_connectivity()
       └─▶ resync_from_snapshot()
```

---

This structure provides a solid foundation for managing IPC validator infrastructure at scale.

