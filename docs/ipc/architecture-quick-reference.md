# IPC Architecture Quick Reference

> Quick reference guide for IPC architecture, components, and common operations

## Core Concepts

| Concept | Description |
|---------|-------------|
| **Root Network (L1)** | Trust anchor (Filecoin/Calibration) running Gateway and Subnet Actor contracts |
| **Subnet** | Child blockchain with own validators, consensus, and execution environment |
| **Gateway Contract** | Singleton managing IPC protocol, collateral, and cross-net messages (one per subnet) |
| **Subnet Actor** | Contract governing specific child subnet (deployed in parent) |
| **Fendermint** | Subnet node implementation (CometBFT + ABCI++ + FVM/FEVM) |
| **Top-Down** | Parent → Child information flow (finality, messages, validator changes) |
| **Bottom-Up** | Child → Parent information flow (checkpoints, messages, confirmations) |
| **Checkpoint** | Batch of child state + messages submitted to parent with validator signatures |
| **Relayer** | Process that submits checkpoints from child to parent (permissionless) |
| **Configuration ID** | Version number for validator set (tracks membership changes) |

## Component Architecture

```
┌─────────────────────────────────────────────────────────┐
│ L1 Root Network (Filecoin)                              │
│ ┌─────────┐  ┌──────────────┐  ┌──────────┐           │
│ │ Gateway │  │ Subnet Actors│  │ Registry │           │
│ └─────────┘  └──────────────┘  └──────────┘           │
└────────────────────┬─────────────────────────┬──────────┘
            ▲ Top-Down (Finality)     Bottom-Up (Checkpoints) ▼
            │                                  │
┌───────────┴──────────────────────────────────┴─────────┐
│ Subnet Level 1                                          │
│ ┌───────────────────────────────────────────────┐      │
│ │ Fendermint Node (Validator)                   │      │
│ │ ┌──────────────┐                              │      │
│ │ │  CometBFT    │ ← Byzantine Fault Tolerance  │      │
│ │ ├──────────────┤                              │      │
│ │ │  ABCI++ App  │ ← Transaction coordination   │      │
│ │ ├──────────────┤                              │      │
│ │ │  FVM/FEVM    │ ← Smart contract execution   │      │
│ │ └──────────────┘                              │      │
│ │ │ Parent Syncer │ IPLD Resolver │             │      │
│ │ └───────────────┴───────────────┘             │      │
│ └───────────────────────────────────────────────┘      │
│ ┌─────────┐  ┌──────────────┐                          │
│ │ Gateway │  │  Registry    │                          │
│ └─────────┘  └──────────────┘                          │
└─────────────────────────────────────────────────────────┘
```

## Key Communication Flows

### Top-Down (Parent → Child)

```
Parent RPC
    ↓ (polling)
Parent Syncer (cache parent blocks)
    ↓ (publish votes)
Vote Tally (GossipSub)
    ↓ (quorum detected)
Block Proposer (include ParentFinality)
    ↓ (consensus)
Block Validation (verify finality)
    ↓ (execution)
Apply Validator Changes + Execute Top-Down Messages
```

**What flows down:**
- Parent block height + hash (finality commitment)
- Top-down messages: `fund`, `pre-fund`
- Validator changes: join, leave, stake, unstake
- Configuration updates

**Timing**: ~2-5 minutes (depends on finality delay)

### Bottom-Up (Child → Parent)

```
User triggers release
    ↓
Message queued in child
    ↓
Checkpoint period reached OR message limit hit
    ↓
Validators create checkpoint
    ↓
Each validator signs checkpoint
    ↓
Quorum reached (>2/3 power)
    ↓
QuorumEvent emitted
    ↓
Relayer monitors event
    ↓
Relayer submits to parent Gateway
    ↓
Parent validates signatures + executes messages
```

**What flows up:**
- Bottom-up messages: `release`, `pre-release`
- Configuration confirmations (validator set sync)
- Child block anchoring (height + hash)

**Timing**: ~5-10 minutes (depends on checkpoint period)

## CLI Command Reference

### Wallet Management
```bash
# Create new wallet
ipc-cli wallet new --wallet-type evm

# Import wallet
ipc-cli wallet import --wallet-type evm --private-key <KEY>

# Export wallet (hex format for Metamask)
ipc-cli wallet export --wallet-type evm --address <ADDR> --hex -o key.txt

# Set default wallet
ipc-cli wallet set-default --address <ADDR> --wallet-type evm

# Check balances
ipc-cli wallet balances --wallet-type evm --subnet <SUBNET_ID>
```

### Subnet Operations
```bash
# List active subnets
ipc-cli subnet list --subnet <PARENT_ID>

# Join as validator
ipc-cli subnet join --subnet <SUBNET_ID> --collateral 10

# Add more stake
ipc-cli subnet stake --subnet <SUBNET_ID> --collateral 5

# Reduce stake
ipc-cli subnet unstake --subnet <SUBNET_ID> --collateral 3

# Leave subnet
ipc-cli subnet leave --subnet <SUBNET_ID>

# Claim collateral/rewards
ipc-cli subnet claim --subnet <SUBNET_ID> [--reward]
```

### Cross-Net Messages
```bash
# Send funds to child subnet
ipc-cli cross-msg fund --subnet <SUBNET_ID> --to <ADDR> 100

# Send funds to parent subnet
ipc-cli cross-msg release --subnet <SUBNET_ID> --to <ADDR> 50

# Pre-fund before subnet bootstrap
ipc-cli cross-msg pre-fund --subnet <SUBNET_ID> 10

# Pre-release before subnet starts
ipc-cli cross-msg pre-release --subnet <SUBNET_ID> 5

# Check parent finality in child
ipc-cli cross-msg parent-finality --subnet <SUBNET_ID>

# List top-down messages
ipc-cli cross-msg list-topdown-msgs --subnet <SUBNET_ID> --epoch <N>
```

### Checkpoint Operations
```bash
# Run relayer
ipc-cli checkpoint relayer --subnet <SUBNET_ID>

# List bottom-up checkpoints
ipc-cli checkpoint list-bottomup --from-epoch <N> --to-epoch <M> --subnet <SUBNET_ID>

# Check quorum events
ipc-cli checkpoint quorum-reached-events --from-epoch <N> --to-epoch <M> --subnet <SUBNET_ID>

# Check if checkpoint submitted
ipc-cli checkpoint has-submitted-bottomup-height --subnet <SUBNET_ID> --submitter <ADDR>

# Get latest checkpoint height
ipc-cli checkpoint last-bottomup-checkpoint-height --subnet <SUBNET_ID>

# List validator changes
ipc-cli checkpoint list-validator-changes --from-epoch <N> --to-epoch <M> --subnet <SUBNET_ID>
```

### Send Value Within Subnet
```bash
# Transfer tokens to another address in same subnet
ipc-cli subnet send-value --subnet <SUBNET_ID> --to <ADDR> 10
```

## Fendermint Infrastructure

### Deploy Bootstrap Node
```bash
cargo make --makefile infra/Makefile.toml \
    -e SUBNET_ID=<SUBNET_ID> \
    -e CMT_P2P_HOST_PORT=26656 \
    -e CMT_RPC_HOST_PORT=26657 \
    -e BOOTSTRAPS=<SEED_NODES> \
    -e PARENT_REGISTRY=<REGISTRY_ADDR> \
    -e PARENT_GATEWAY=<GATEWAY_ADDR> \
    bootstrap
```

### Deploy Validator Node
```bash
cargo make --makefile infra/Makefile.toml \
    -e PRIVATE_KEY_PATH=<KEY_FILE> \
    -e SUBNET_ID=<SUBNET_ID> \
    -e CMT_P2P_HOST_PORT=26656 \
    -e CMT_RPC_HOST_PORT=26657 \
    -e ETHAPI_HOST_PORT=8545 \
    -e BOOTSTRAPS=<SEED_NODES> \
    -e PARENT_REGISTRY=<REGISTRY_ADDR> \
    -e PARENT_GATEWAY=<GATEWAY_ADDR> \
    child-validator
```

### Deploy Full Node
```bash
cargo make --makefile infra/Makefile.toml \
    -e SUBNET_ID=<SUBNET_ID> \
    -e BOOTSTRAPS=<SEED_NODES> \
    -e PARENT_REGISTRY=<REGISTRY_ADDR> \
    -e PARENT_GATEWAY=<GATEWAY_ADDR> \
    child-fullnode
```

## Configuration Files

### ipc-cli Config (`~/.ipc/config.toml`)
```toml
keystore_path = "~/.ipc"

# Filecoin Calibration Testnet
[[subnets]]
id = "/r314159"

[subnets.config]
network_type = "fevm"
provider_http = "https://api.calibration.node.glif.io/rpc/v1"
gateway_addr = "0x1AEe8A878a22280fc2753b3C63571C8F895D2FE3"
registry_addr = "0x0b4e239FF21b40120cDa817fba77bD1B366c1bcD"

# Local Anvil (for development)
[[subnets]]
id = "/r31337"

[subnets.config]
network_type = "fevm"
provider_http = "http://127.0.0.1:8545"
gateway_addr = "<DEPLOYED_GATEWAY_ADDR>"
registry_addr = "<DEPLOYED_REGISTRY_ADDR>"
```

### Fendermint Config (`~/.fendermint/config/default.toml`)
```toml
[abci]
[abci.listen]
host = "0.0.0.0"
port = 26658

[db]
state_hist_size = 1000

[resolver]
network = "mainnet"

[eth.listen]
host = "0.0.0.0"
port = 8545
```

## Smart Contract Addresses

### Calibration Testnet (r314159)
| Contract | Address |
|----------|---------|
| Gateway | `0x1AEe8A878a22280fc2753b3C63571C8F895D2FE3` |
| Registry | `0x0b4e239FF21b40120cDa817fba77bD1B366c1bcD` |

## Subnet ID Format

```
/r<chainid>/<subnet-actor-address>/<subnet-actor-address>/...
```

**Examples:**
- Root: `/r314159` (Calibration)
- Level 1: `/r314159/t410fexample...`
- Level 2: `/r314159/t410fexample.../t410fchild...`

## Key Parameters

| Parameter | Typical Value | Description |
|-----------|---------------|-------------|
| `bottomup_check_period` | 100 blocks | Frequency of checkpoint creation |
| `min_validators` | 3-5 | Minimum validators to bootstrap subnet |
| `min_collateral` | 1-10 FIL | Minimum total collateral required |
| `majority_percentage` | 67% | Quorum threshold (>2/3) |
| `finality_delay` | 10-20 blocks | Parent blocks to wait before final |
| `max_proposal_range` | 50 blocks | Max parent height advance per child block |
| `MAX_MSGS_PER_BATCH` | 100 | Triggers intermediate checkpoint |

## Security Model

### Trust Assumptions
- ✅ L1 root network is secure
- ✅ >2/3 of validator power is honest
- ✅ Economic security via collateral
- ✅ Finality delay prevents parent reorgs
- ✅ Cryptographic signatures are unforgeable

### Attack Mitigations
- **Long-range attacks**: Block hash anchoring in checkpoints
- **Double-spend**: BFT consensus + checkpoint finality
- **Validator collusion**: Requires >2/3 stake (economically irrational)
- **Checkpoint censorship**: Multiple relayers provide redundancy
- **Parent reorg**: Finality delay ensures stability

## Common Issues & Debugging

### Issue: Cross-net message not executing
**Check:**
1. Message included in checkpoint? `ipc-cli checkpoint list-bottomup`
2. Checkpoint submitted? `ipc-cli checkpoint last-bottomup-checkpoint-height`
3. Relayer running? Check for `QuorumReached` events
4. Parent finality committed? `ipc-cli cross-msg parent-finality`

### Issue: Validator not signing checkpoints
**Check:**
1. Validator has funds for gas? `ipc-cli wallet balances`
2. Validator in current power table? `ipc-cli subnet list`
3. Configuration ID confirmed? Check validator changes
4. Fendermint node running? Check logs

### Issue: Subnet won't bootstrap
**Check:**
1. Minimum validators joined? `ipc-cli subnet list`
2. Minimum collateral met? Check total collateral
3. All validators funded? Each needs genesis balance
4. Subnet Actor registered? Should appear in list

## Performance Tuning

### Optimize Checkpoint Submission
- Reduce `bottomup_check_period` for faster cross-net messages
- Increase `MAX_MSGS_PER_BATCH` to batch more messages
- Run multiple relayers for redundancy

### Optimize Finality
- Reduce finality delay (but increase reorg risk)
- Increase validator count for faster quorum
- Use faster parent RPC endpoint

### Optimize Network
- Deploy bootstrap nodes geographically distributed
- Use persistent peer connections
- Enable Prometheus metrics for monitoring

## Monitoring Metrics

### Critical Metrics
- Time since last checkpoint submitted
- Parent finality lag (child view vs actual)
- Validator signing rate
- Cross-net message queue depth
- Relayer submission success rate

### Alerting Thresholds
- ⚠️ No checkpoint submitted in 2x checkpoint period
- ⚠️ Parent finality lag > 100 blocks
- ⚠️ Validator missed >10% of checkpoint signatures
- ⚠️ Message queue > 80% of MAX_MSGS_PER_BATCH

## Useful Links

- **Documentation**: [docs/ipc/](.)
- **Architecture Details**: [architecture-overview.md](./architecture-overview.md)
- **Usage Guide**: [usage.md](./usage.md)
- **Contract Reference**: [contracts.md](./contracts.md)
- **Troubleshooting**: [../troubleshooting-subnet-deployment.md](../troubleshooting-subnet-deployment.md)
- **IPC Website**: https://www.ipc.space/
- **Calibration Faucet**: https://faucet.calibration.fildev.network/funds.html

## Quick Start Checklist

### For Users
- [ ] Install IPC CLI: `make build`
- [ ] Initialize config: `ipc-cli config init`
- [ ] Create wallet: `ipc-cli wallet new --wallet-type evm`
- [ ] Get testnet FIL from faucet
- [ ] Join existing subnet: `ipc-cli subnet join --subnet <ID> --collateral 10`
- [ ] Test cross-net: `ipc-cli cross-msg fund --subnet <ID> 1`

### For Validators
- [ ] Export validator key: `ipc-cli wallet export --wallet-type evm --address <ADDR> --hex`
- [ ] Join subnet with collateral: `ipc-cli subnet join --subnet <ID> --collateral 10`
- [ ] Deploy Fendermint node: `cargo make child-validator`
- [ ] Monitor node logs: `docker logs -f fendermint`
- [ ] Run relayer: `ipc-cli checkpoint relayer --subnet <ID>`

### For Subnet Creators
- [ ] Design subnet parameters (validators, collateral, period)
- [ ] Deploy subnet actor via registry
- [ ] Fund initial validators: `ipc-cli cross-msg pre-fund`
- [ ] Wait for minimum validators to join
- [ ] Coordinate validator node deployment
- [ ] Bootstrap subnet and deploy infrastructure
- [ ] Monitor checkpoint submissions

---

**Version**: 1.0  
**Last Updated**: January 2026
