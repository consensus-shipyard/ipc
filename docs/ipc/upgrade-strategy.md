# IPC Upgrade Strategy: From Manual to Automated

**Version:** 1.0
**Date:** November 3, 2025
**Status:** Planning

## Executive Summary

This document outlines a phased approach to evolve IPC's upgrade mechanism from manual coordination to fully automated, network-driven upgrades. The strategy addresses immediate needs (next 2 weeks) while building toward a production-grade, zero-coordination upgrade system over the next few months.

### Key Requirements

1. **Short-term (2 weeks):** Minimal downtime upgrades for IPC team-operated networks
2. **Medium-term (2-3 months):** Automated upgrades with "restart node and it upgrades" UX
3. **Long-term vision:** Network self-coordinates upgrades based on validator readiness
4. **Constraint:** No backward compatibility required; breaking changes acceptable with upgrade path
5. **Environment support:** Must work across testnet, mainnet, and private deployments

---

## Current State Analysis

### Two Independent Upgrade Systems

#### 1. Smart Contract Upgrades (On-Chain Actors)

**Components:**
- Gateway Diamond (singleton in every subnet)
- Subnet Actor Diamond (per-subnet logic in parent)
- Subnet Registry Diamond (factory contract)

**Current Process:**
```bash
# Manual steps required:
1. Edit contract code in contracts/src/
2. Convert subnet ID to ETH address via external tool (Beryx)
3. Set RPC_URL and PRIVATE_KEY environment variables
4. Run: make upgrade-sa-diamond SUBNET_ACTOR_ADDRESS=0x... NETWORK=calibrationnet
```

**Pain Points:**
- Requires private key holder to execute
- No coordination mechanism
- Manual address conversion
- No verification of success

#### 2. Fendermint Binary Upgrades (Validator Nodes)

**Current Mechanisms:**

**A. UpgradeScheduler (State Migrations)**
- Hardcoded migrations compiled into binary
- Executed at predetermined block heights
- **Limitation:** Migrations must be known at compile time

**B. halt_height (Binary Switching)**
```toml
# .fendermint/config/default.toml
halt_height = 10000  # Node exits with code 2 at this height
```

**Current Process:**
```
1. Team discusses halt_height via Discord/Slack
2. Each operator manually edits config file
3. Each operator restarts Fendermint to load config
4. Wait for network to reach halt_height
5. All nodes halt simultaneously
6. Each operator manually:
   - Stops process (if auto-restart enabled)
   - Replaces binary
   - Updates halt_height to 0
   - Restarts Fendermint
7. Network resumes
```

**Pain Points:**
- Requires out-of-band coordination (chat, email)
- Manual config editing on every node
- Requires process restarts before upgrade
- Simultaneous downtime for all nodes
- No verification all nodes upgraded
- No rollback mechanism
- High risk of human error
- If operator misses halt_height update, node becomes stuck

---

## Phased Upgrade Strategy

### Phase 1: Improved Manual Process (2 weeks)
**Goal:** Reduce downtime and coordination overhead for IPC team operations

### Phase 2: Semi-Automated Coordination (2-3 months)
**Goal:** "Restart node with new binary, network handles the rest" UX

### Phase 3: Network-Driven Upgrades (Future)
**Goal:** Network automatically schedules upgrades when quorum of nodes ready

---

## Phase 1: Improved Manual Process

**Timeline:** 2 weeks
**Target Users:** IPC team internal operations
**Downtime Goal:** < 30 seconds

### 1.1 Upgrade Coordinator CLI Tool

**New tool:** `ipc-cli upgrade` subcommands

```bash
# Propose an upgrade (creates on-chain upgrade proposal)
ipc-cli upgrade propose \
  --height 15000 \
  --binary-url https://github.com/ipc/releases/v0.2.0/fendermint \
  --binary-hash sha256:abc123... \
  --contracts gateway,subnet-actor \
  --network calibration

# Check upgrade status
ipc-cli upgrade status --network calibration

# Signal node readiness (operator confirms binary downloaded)
ipc-cli upgrade ready --validator-address 0x...

# Execute upgrade (updates contracts if specified)
ipc-cli upgrade execute --network calibration
```

**Benefits:**
- Single source of truth for upgrade plan
- Automated address conversion
- Built-in verification
- Coordination visible on-chain

### 1.2 Upgrade Registry Smart Contract

**New contract:** `UpgradeRegistry.sol`

```solidity
struct UpgradeProposal {
    uint64 id;
    uint64 targetHeight;
    bytes32 binaryHash;
    string binaryUrl;
    address proposer;
    uint64 proposedAt;
    bool executed;
    mapping(address => bool) validatorReady;
    uint64 readyCount;
}

function proposeUpgrade(
    uint64 targetHeight,
    bytes32 binaryHash,
    string calldata binaryUrl
) external returns (uint64 proposalId);

function signalReady(uint64 proposalId) external;

function getUpgradeStatus(uint64 proposalId)
    external view returns (UpgradeProposal memory);
```

**Deployment:**
- One registry per subnet
- Gateway holds reference to current registry
- Can be upgraded via diamond pattern

### 1.3 Fendermint Upgrade Monitor

**New module:** `fendermint/app/src/upgrade_monitor.rs`

```rust
pub struct UpgradeMonitor {
    registry_contract: Address,
    tendermint_client: TendermintClient,
    current_proposal: Option<UpgradeProposal>,
}

impl UpgradeMonitor {
    // Query registry every N blocks
    async fn check_for_upgrades(&self, current_height: BlockHeight);

    // Download and verify binary
    async fn prepare_upgrade(&self, proposal: &UpgradeProposal) -> Result<PathBuf>;

    // Update halt_height automatically
    async fn set_halt_height(&self, height: BlockHeight) -> Result<()>;

    // Signal readiness after successful preparation
    async fn signal_ready(&self, proposal_id: u64) -> Result<()>;
}
```

**Integration:**
- Runs as background task in Fendermint
- Queries registry every 100 blocks
- Auto-updates `halt_height` in memory (no config file edit needed)
- Logs all upgrade activities

### 1.4 Process Flow (Phase 1)

```
┌─────────────────────────────────────────────────────────────┐
│ Step 1: Propose Upgrade (IPC Team Lead)                     │
├─────────────────────────────────────────────────────────────┤
│ $ ipc-cli upgrade propose --height 15000 --binary-url ...   │
│ ✓ Upgrade proposal #7 created                               │
│ ✓ Target height: 15000                                      │
│ ✓ Binary: v0.2.0 (sha256:abc123...)                         │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│ Step 2: Fendermint Auto-Detects (All Validator Nodes)       │
├─────────────────────────────────────────────────────────────┤
│ [INFO] Upgrade proposal #7 detected                         │
│ [INFO] Downloading binary from IPFS...                      │
│ [INFO] Verifying hash... ✓                                  │
│ [INFO] Setting halt_height=15000                            │
│ [INFO] Signaling ready to registry                          │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│ Step 3: Monitor Readiness (Anyone)                          │
├─────────────────────────────────────────────────────────────┤
│ $ ipc-cli upgrade status                                    │
│ Upgrade #7 (target height: 15000)                           │
│ Ready: 4/4 validators (100%)                                │
│ Current height: 14850                                       │
│ ETA: ~2 minutes                                             │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│ Step 4: Automatic Halt (Block 15000)                        │
├─────────────────────────────────────────────────────────────┤
│ [INFO] Block 15000 reached                                  │
│ [INFO] Halting due to upgrade #7                            │
│ [INFO] Executing pre-upgrade tasks...                       │
│ [INFO] Exiting with code 2 (upgrade halt)                   │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│ Step 5: Binary Swap (Orchestrator or Manual)                │
├─────────────────────────────────────────────────────────────┤
│ Option A: Manual (systemd, docker-compose, etc.)            │
│   - Operator updates binary in deployment config            │
│   - Restarts service                                        │
│                                                             │
│ Option B: Upgrade Orchestrator (planned Phase 2)            │
│   - Detects exit code 2                                     │
│   - Swaps binary automatically                              │
│   - Restarts Fendermint                                     │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│ Step 6: Resume (All Nodes)                                  │
├─────────────────────────────────────────────────────────────┤
│ [INFO] Starting Fendermint v0.2.0                           │
│ [INFO] Detecting upgrade #7 completed                       │
│ [INFO] Executing upgrade scheduler migrations...            │
│ [INFO] State migration completed                            │
│ [INFO] Resuming consensus at height 15001                   │
└─────────────────────────────────────────────────────────────┘
```

### 1.5 Implementation Tasks (Phase 1)

1. **Create UpgradeRegistry contract** (2 days)
   - Define schema
   - Implement proposal/ready signaling
   - Write tests
   - Deploy to test networks

2. **Add upgrade monitor to Fendermint** (3 days)
   - Query registry contract
   - Download/verify binaries
   - Auto-update halt_height
   - Signal readiness

3. **Extend ipc-cli with upgrade commands** (2 days)
   - `upgrade propose`
   - `upgrade status`
   - `upgrade ready` (manual signal if needed)

4. **Integration testing** (2 days)
   - 4-validator test network
   - Simulate upgrade flow end-to-end
   - Test failure scenarios

5. **Documentation** (1 day)
   - Operator guide
   - Architecture docs
   - Runbook for troubleshooting

**Total:** ~10 days (2 weeks with buffer)

---

## Phase 2: Semi-Automated Coordination

**Timeline:** 2-3 months
**Target Users:** External subnet operators
**UX Goal:** Operator updates binary and restarts; network handles upgrade

### 2.1 Upgrade Orchestrator (Cosmovisor-Style)

**New binary:** `ipc-orchestrator`

Wraps Fendermint process and manages lifecycle:

```yaml
# orchestrator-config.yaml
fendermint:
  binary_path: /usr/local/bin/fendermint
  data_dir: ~/.fendermint
  auto_download: true
  binary_registry: ipfs://...

upgrade:
  auto_apply: true
  backup_enabled: true
  rollback_on_failure: true
  max_downtime: 60s
```

**Features:**

1. **Binary Management**
   - Maintains directory of version binaries
   - Downloads from IPFS/GitHub based on registry
   - Verifies signatures and hashes

2. **Automatic Upgrade Application**
   - Monitors Fendermint exit codes
   - Code 0: Normal exit
   - Code 1: Error (don't restart)
   - Code 2: Upgrade halt (apply upgrade)

3. **Rollback Protection**
   - Creates state backup before upgrade
   - Sets timeout for new version (5 minutes)
   - Reverts if new version fails to start

4. **Health Monitoring**
   - Checks if node is keeping up with consensus
   - Alerts if node falls behind after upgrade
   - Can trigger automatic rollback

### 2.2 Enhanced Upgrade Proposals with Governance

**Extended UpgradeRegistry:**

```solidity
struct UpgradeProposal {
    // ... existing fields ...

    // Governance fields
    uint64 votingPeriod;
    uint64 votingDeadline;
    mapping(address => bool) votes;
    uint64 yesVotes;
    uint64 noVotes;
    uint64 totalVotingPower;

    // Execution fields
    uint64 executionWindow; // Blocks after targetHeight to complete
    bytes migrationData;    // Optional state migration params

    // Rollback
    bool rolled_back;
    string rollbackReason;
}

function vote(uint64 proposalId, bool support) external;
function executeUpgrade(uint64 proposalId) external;
function rollbackUpgrade(uint64 proposalId, string calldata reason) external;
```

**Voting Mechanism:**
- Validators vote with voting power proportional to stake
- Proposal passes with 2/3+ majority
- Voting period: 7 days typical
- After passing, `targetHeight` set automatically

### 2.3 Dynamic Upgrade Scheduling

**Problem:** Hardcoded migrations in UpgradeScheduler aren't flexible

**Solution:** Runtime-loadable upgrade handlers

```rust
// fendermint/vm/interpreter/src/fvm/upgrades.rs

pub enum UpgradeHandler<DB> {
    // Existing: compiled-in function
    Compiled(MigrationFunc<DB>),

    // New: WASM-based migration
    Wasm {
        code: Vec<u8>,
        entry_point: String,
    },

    // New: Standard operations (no custom code)
    Standard(StandardUpgrade),
}

pub enum StandardUpgrade {
    // Deploy new contract at address
    DeployContract {
        bytecode: Vec<u8>,
        constructor_args: Vec<u8>,
    },

    // Upgrade existing contract
    UpgradeContract {
        address: Address,
        new_code: Vec<u8>,
    },

    // Patch state (key-value updates)
    PatchState {
        updates: Vec<(Address, Vec<u8>, Vec<u8>)>, // (actor, key, value)
    },

    // No-op (binary upgrade only)
    NoOp,
}
```

**Loading from UpgradeRegistry:**

```rust
impl UpgradeMonitor {
    async fn load_upgrade_handler(&self, proposal: &UpgradeProposal)
        -> Result<UpgradeHandler<DB>>
    {
        // Fetch migration data from proposal
        let migration_type = proposal.migration_data.type;

        match migration_type {
            MigrationType::Compiled => {
                // Look up in built-in registry
                get_compiled_migration(proposal.id)
            }
            MigrationType::Wasm => {
                // Download WASM from IPFS
                let wasm_code = ipfs_get(&proposal.migration_data.wasm_cid).await?;
                Ok(UpgradeHandler::Wasm {
                    code: wasm_code,
                    entry_point: "migrate".to_string(),
                })
            }
            MigrationType::Standard => {
                // Parse standard operations
                let ops = decode_standard_ops(&proposal.migration_data.ops)?;
                Ok(UpgradeHandler::Standard(ops))
            }
            MigrationType::NoOp => {
                Ok(UpgradeHandler::Standard(StandardUpgrade::NoOp))
            }
        }
    }
}
```

### 2.4 Operator Experience (Phase 2)

**Before Upgrade (Operator):**

```bash
# 1. Upgrade is proposed on-chain (by governance or admin)
# 2. Operator receives notification (email, Slack bot, etc.)
# 3. Operator reviews proposal

$ ipc-orchestrator status
Current version: v0.1.5
Pending upgrade: v0.2.0
  - Target height: 25000 (in ~5 days)
  - Status: Approved by governance
  - Required: Update binary before height 25000
  - Migration: Standard (deploy new contract)

# 4. Operator updates config to auto-upgrade
$ ipc-orchestrator config set upgrade.auto_apply=true

# That's it! Orchestrator handles the rest.
```

**During Upgrade (Automatic):**

```
[Height 24900] Orchestrator: Preparing for upgrade #12
[Height 24900] Orchestrator: Downloading binary v0.2.0...
[Height 24900] Orchestrator: Binary verified (sha256:xyz789...)
[Height 24900] Orchestrator: Creating state backup...
[Height 24900] Orchestrator: Backup saved to ~/.fendermint/backups/upgrade-12
[Height 24900] Orchestrator: Ready for upgrade
[Height 25000] Fendermint: Halting for upgrade #12
[Height 25000] Fendermint: Exit code 2
[Height 25000] Orchestrator: Detected upgrade halt
[Height 25000] Orchestrator: Swapping binary v0.1.5 → v0.2.0
[Height 25000] Orchestrator: Starting Fendermint v0.2.0...
[Height 25001] Fendermint v0.2.0: Starting upgrade migration
[Height 25001] Fendermint v0.2.0: Deploying contract at 0xabc...
[Height 25001] Fendermint v0.2.0: Migration complete
[Height 25001] Fendermint v0.2.0: Resuming consensus
[Height 25002] Orchestrator: Health check passed
[Height 25002] Orchestrator: Upgrade #12 successful
```

**If Upgrade Fails:**

```
[Height 25001] Fendermint v0.2.0: Migration failed: contract deployment error
[Height 25001] Fendermint v0.2.0: Exit code 1
[Height 25001] Orchestrator: ⚠️  New version failed to start
[Height 25001] Orchestrator: Initiating rollback...
[Height 25001] Orchestrator: Restoring state from backup
[Height 25001] Orchestrator: Swapping binary v0.2.0 → v0.1.5
[Height 25001] Orchestrator: Starting Fendermint v0.1.5...
[Height 25002] Fendermint v0.1.5: Resuming consensus
[Height 25002] Orchestrator: ⚠️  Upgrade #12 rolled back
[Height 25002] Orchestrator: Signaling rollback to network...
```

### 2.5 Implementation Tasks (Phase 2)

1. **Upgrade Orchestrator** (3 weeks)
   - Process wrapper with lifecycle management
   - Binary download/verification
   - Backup/restore functionality
   - Exit code monitoring
   - Rollback logic
   - Health checks

2. **Enhanced UpgradeRegistry with Governance** (2 weeks)
   - Voting mechanism
   - Proposal lifecycle management
   - Migration data storage
   - Events for monitoring

3. **Dynamic Upgrade Handlers** (2 weeks)
   - WASM runtime integration
   - Standard operation types
   - Handler loading from registry
   - Security sandboxing

4. **Integration with Orchestrator** (1 week)
   - Registry querying
   - Automatic scheduling
   - Readiness signaling
   - Failure reporting

5. **Testing & Validation** (2 weeks)
   - Multi-node testnet upgrades
   - Failure scenario testing
   - Rollback testing
   - Performance benchmarking

6. **Documentation & Tooling** (1 week)
   - Operator guide
   - Upgrade proposal template
   - Monitoring dashboards
   - Alerting setup guide

**Total:** ~11 weeks (~2.5 months)

---

## Phase 3: Network-Driven Upgrades

**Timeline:** Future (post-Phase 2)
**Goal:** Network self-coordinates based on validator readiness

### 3.1 Readiness-Based Scheduling

**Concept:** Don't set `targetHeight` in advance. Instead, network automatically schedules upgrade when enough validators signal readiness.

```solidity
struct UpgradeProposal {
    // ... existing fields ...

    // Readiness-based scheduling
    uint64 readinessThreshold; // e.g., 67% (2/3 validators)
    uint64 readinessDeadline;  // If not ready by this height, cancel
    uint64 schedulingWindow;   // Hours between ready threshold and execution

    bool autoScheduled;
    uint64 autoScheduledAt;
    uint64 autoScheduledHeight;
}

function checkAndSchedule(uint64 proposalId) external {
    UpgradeProposal storage p = proposals[proposalId];

    uint64 readyPower = calculateReadyVotingPower(proposalId);
    uint64 totalPower = getTotalVotingPower();

    if (readyPower * 100 / totalPower >= p.readinessThreshold) {
        // Quorum reached! Schedule upgrade
        uint64 currentHeight = block.number;
        p.targetHeight = currentHeight + blocksInHours(p.schedulingWindow);
        p.autoScheduled = true;
        p.autoScheduledAt = block.timestamp;
        p.autoScheduledHeight = currentHeight;

        emit UpgradeAutoScheduled(proposalId, p.targetHeight);
    }
}
```

**Flow:**

1. Upgrade proposed with `readinessThreshold=67%`, `schedulingWindow=24h`
2. Validators update binaries at their convenience
3. Each validator signals ready after successful binary download
4. When 67% ready, network automatically schedules upgrade in 24 hours
5. Remaining 33% have 24 hours to update or fall out of consensus

### 3.2 Graceful Degradation for Late Upgraders

**Problem:** What if validators miss the upgrade window?

**Solution:** Extended compatibility window

```rust
pub struct CompatibilityWindow {
    /// Block height where upgrade executed
    upgrade_height: BlockHeight,

    /// Blocks to allow old version to sync (grace period)
    grace_period: u64,

    /// Old version can sync blocks but not validate
    old_version_read_only: bool,
}

impl Fendermint {
    fn check_version_compatibility(&self, height: BlockHeight) -> Result<VersionMode> {
        if height < upgrade_height {
            // Pre-upgrade blocks
            Ok(VersionMode::Normal)
        } else if height < upgrade_height + grace_period {
            // Grace period: old version can sync but not validate
            if self.version < required_version {
                Ok(VersionMode::ReadOnly)
            } else {
                Ok(VersionMode::Normal)
            }
        } else {
            // After grace period: must upgrade
            if self.version < required_version {
                Err(anyhow!("Version too old. Please upgrade to continue."))
            } else {
                Ok(VersionMode::Normal)
            }
        }
    }
}
```

**Validator Experience:**

```
Validator on old version after upgrade:

[Height 30001] ⚠️  Network upgraded to v0.3.0
[Height 30001] ⚠️  You are running v0.2.0
[Height 30001] ⚠️  Entering read-only mode
[Height 30001] ℹ️  You can sync blocks but cannot validate
[Height 30001] ℹ️  Grace period: 1000 blocks (~8 hours)
[Height 30001] ℹ️  Upgrade before height 31001 to resume validation

[Height 30500] ⚠️  Grace period remaining: 500 blocks (~4 hours)
[Height 30900] ⚠️  Grace period remaining: 100 blocks (~48 minutes)
[Height 30990] 🚨 Grace period remaining: 10 blocks (~5 minutes)

[Height 31001] 🚨 Grace period expired
[Height 31001] 🚨 Shutting down. Please upgrade to v0.3.0.
```

### 3.3 Version Advertisement

**Validators advertise version in consensus messages:**

```rust
pub struct ValidatorInfo {
    address: Address,
    voting_power: u64,
    binary_version: String,  // e.g., "v0.3.0"
    protocol_version: u64,   // e.g., 3
}

// In CometBFT validator set
impl Validator {
    fn to_tendermint_validator(&self) -> tendermint::Validator {
        tendermint::Validator {
            // ... standard fields ...

            // Custom field for version
            extra: serde_json::to_vec(&ValidatorInfo {
                address: self.address,
                voting_power: self.power,
                binary_version: env!("CARGO_PKG_VERSION").to_string(),
                protocol_version: PROTOCOL_VERSION,
            }).unwrap(),
        }
    }
}
```

**Network Dashboard:**

```
Subnet Validator Status

Upgrade #15 (v0.3.0) - Auto-scheduling enabled
Ready: 8/12 validators (67%) ← Threshold: 67%
Status: ⚠️  Ready to schedule

Ready Validators (8):
  ✓ validator-1  v0.3.0  [Ready for 2 hours]
  ✓ validator-2  v0.3.0  [Ready for 1 hour]
  ✓ validator-3  v0.3.0  [Ready for 30 minutes]
  ...

Pending Validators (4):
  ⏳ validator-9  v0.2.0  [Last seen: 2 mins ago]
  ⏳ validator-10 v0.2.0  [Last seen: 5 mins ago]
  ...

⚡ Upgrade will auto-schedule in ~10 minutes if no more validators ready
📅 Estimated execution: 24 hours after scheduling
```

### 3.4 Implementation Tasks (Phase 3)

This is a future phase, but high-level tasks:

1. **Readiness-based scheduling logic** (2 weeks)
2. **Version advertisement in consensus** (2 weeks)
3. **Grace period & read-only mode** (2 weeks)
4. **Network monitoring dashboard** (1 week)
5. **Testing across scenarios** (2 weeks)

**Total:** ~9 weeks

---

## Smart Contract Upgrade Strategy

Smart contract upgrades (Gateway, Subnet Actor, Registry) work differently from binary upgrades since they're on-chain state changes.

### Current vs. Improved Flow

**Current (Manual):**
```bash
1. Developer edits contracts/src/gateway/GatewayFacet.sol
2. Developer runs: make upgrade-gw-diamond NETWORK=calibration
3. Transaction sent from developer's wallet
4. Upgrade happens immediately (no coordination)
```

**Improved (Coordinated):**

```bash
1. Developer edits contracts/src/gateway/GatewayFacet.sol
2. Developer proposes upgrade via registry:
   $ ipc-cli upgrade propose-contract \
       --contract gateway \
       --facets GatewayFacet,CheckpointingFacet \
       --network calibration

3. Registry emits event: ContractUpgradeProposed
4. Validators review bytecode diff (on-chain or via IPFS)
5. Validators vote (on-chain transaction)
6. If approved, scheduled for execution
7. Anyone can trigger execution after approval
```

### Coordinating Binary + Contract Upgrades

Often both need to upgrade together. The upgraded Fendermint binary may depend on new contract interfaces.

**Solution: Linked Upgrade Proposals**

```solidity
struct UpgradeProposal {
    // ... existing fields ...

    // Contract upgrades included in this proposal
    address[] contractsToUpgrade;
    bytes[] contractUpgradeData;

    // Execution order
    bool upgradeContractsFirst; // true = contracts before halt
}
```

**Execution Flow:**

```
Proposal: Upgrade to v0.3.0 + new Gateway contract

1. Proposal approved by governance
2. Ready threshold reached (67% validators)
3. Upgrade auto-scheduled for height 40000

[Height 39990] Pre-upgrade contract changes
[Height 39990] Execute contract upgrades (if upgradeContractsFirst=true)
[Height 39990] Gateway upgraded to v2
[Height 39990] Subnet Actor upgraded to v2

[Height 40000] Binary upgrade halt
[Height 40000] Validators swap to Fendermint v0.3.0
[Height 40000] Fendermint v0.3.0 starts
[Height 40000] Fendermint reads new contract interfaces ✓
[Height 40001] Network resumes with both upgrades complete
```

---

## Migration Path from Current to Phase 1

### Week 1: Core Infrastructure

**Day 1-2: UpgradeRegistry Contract**
```
File: contracts/contracts/upgrade/UpgradeRegistry.sol
- Define proposal struct
- Implement propose/vote/signal ready
- Add query methods
- Write unit tests
```

**Day 3-4: Fendermint Upgrade Monitor**
```
File: fendermint/app/src/upgrade/monitor.rs
- Query registry contract periodically
- Parse upgrade proposals
- Download/verify binaries
- Update halt_height dynamically
```

**Day 5: CLI Commands**
```
File: ipc/cli/src/commands/upgrade/
- upgrade propose
- upgrade status
- upgrade ready
```

### Week 2: Integration & Testing

**Day 6-7: Integration Testing**
```
- Deploy registry to test network
- 4-validator upgrade scenario
- Test failure cases
- Verify monitoring/alerting
```

**Day 8-9: Documentation**
```
- docs/ipc/upgrade-guide.md
- docs/ipc/upgrade-operator-runbook.md
- Update README with upgrade info
```

**Day 10: Production Deployment**
```
- Deploy UpgradeRegistry to Calibration testnet
- Update Fendermint binaries with monitor
- Announce new upgrade process
```

---

## Testing Strategy

### Phase 1 Testing

**Local 4-Validator Network:**
```bash
# scripts/test-upgrade.sh

1. Start 4-validator testnet
2. Propose upgrade via CLI
3. Verify all nodes detect proposal
4. Verify all nodes download binary
5. Verify all nodes signal ready
6. Wait for halt_height
7. Verify all nodes halt with exit code 2
8. Manually replace binaries
9. Verify all nodes resume
10. Verify state consistency
```

**Failure Scenarios:**
- One validator fails to download binary
- One validator halts early
- One validator doesn't halt
- Binary verification fails
- Network splits during upgrade

### Phase 2 Testing

**Automated Upgrade:**
- Orchestrator handles full upgrade cycle
- Test rollback on migration failure
- Test rollback on health check failure
- Test upgrade with contract changes

**Governance:**
- Vote on upgrade proposal
- Vote rejection
- Vote timeout
- Emergency upgrade

### Phase 3 Testing

**Readiness-Based:**
- Auto-schedule when threshold reached
- Validators join after scheduling
- Validators miss upgrade window
- Grace period expiration

---

## Monitoring & Observability

### Metrics to Track

**Upgrade Coordination:**
- `ipc_upgrade_proposal_count` - Total proposals created
- `ipc_upgrade_validators_ready` - Validators ready for upgrade
- `ipc_upgrade_time_to_ready` - Time from proposal to ready threshold
- `ipc_upgrade_completion_time` - Downtime duration

**Binary Management:**
- `ipc_binary_download_duration` - Time to download binary
- `ipc_binary_verification_success` - Verification success rate
- `ipc_orchestrator_restarts` - Number of orchestrator restarts
- `ipc_upgrade_rollbacks` - Number of rollbacks

**Consensus Health:**
- `ipc_consensus_lag` - Blocks behind after upgrade
- `ipc_validator_version_distribution` - Version distribution
- `ipc_upgrade_failures` - Failed upgrades

### Alerting Rules

```yaml
# alerts/upgrade.yml

- alert: UpgradeProposalCreated
  expr: increase(ipc_upgrade_proposal_count[5m]) > 0
  for: 1m
  annotations:
    summary: "New upgrade proposal #{{ $labels.proposal_id }}"

- alert: ValidatorNotReady
  expr: ipc_upgrade_validators_ready / ipc_total_validators < 0.67
  for: 1h
  annotations:
    summary: "Only {{ $value }}% validators ready for upgrade"

- alert: UpgradeHaltImminent
  expr: (ipc_upgrade_target_height - ipc_current_height) < 100
  for: 1m
  annotations:
    summary: "Upgrade halt in ~{{ $value }} blocks"

- alert: UpgradeRollback
  expr: increase(ipc_upgrade_rollbacks[5m]) > 0
  for: 1m
  annotations:
    summary: "⚠️  Upgrade rolled back on validator {{ $labels.validator }}"
```

---

## Security Considerations

### Binary Verification

**Problem:** Validators download binaries from IPFS/GitHub. How to prevent malicious binaries?

**Solutions:**

1. **Multi-signature Verification**
   ```
   Binary must be signed by M of N core developers
   Validators verify signatures before accepting
   ```

2. **Reproducible Builds**
   ```
   Build process documented
   Validators can rebuild from source
   Compare hash with distributed binary
   ```

3. **Staged Rollout**
   ```
   Deploy to testnet first
   Monitor for 48 hours
   Then deploy to mainnet
   ```

### Migration Security

**Problem:** WASM migrations in Phase 2/3 could be exploited

**Solutions:**

1. **Sandboxing**
   ```rust
   - Limit gas for migration execution
   - Restrict syscalls (no network, limited file I/O)
   - Read-only access to most state
   - Explicit permissions for state modifications
   ```

2. **Formal Verification**
   ```
   Critical migrations reviewed by security auditor
   Automated tests for common exploits
   Require supermajority for WASM migrations (75% vs 67%)
   ```

3. **Emergency Stop**
   ```solidity
   function emergencyHalt(uint64 proposalId, string reason)
       external
       onlyEmergencyMultisig
   {
       // Immediately cancel upgrade
       // Broadcast halt to all validators
       // Requires 3-of-5 emergency multisig
   }
   ```

---

## Cost-Benefit Analysis

### Phase 1 Benefits
- ✅ Single source of truth for upgrades
- ✅ Eliminate manual config editing
- ✅ Reduce downtime from ~5 minutes to ~30 seconds
- ✅ Reduce operator errors
- ✅ Auditability (all upgrades on-chain)

### Phase 1 Costs
- 🔨 2 weeks development
- 🔨 Additional on-chain storage (~1KB per proposal)
- 🔨 Network queries every 100 blocks (~negligible gas)

### Phase 2 Benefits
- ✅ "Set and forget" operator experience
- ✅ Automatic rollback on failure
- ✅ Governance-driven upgrades
- ✅ Dynamic migrations (no recompilation)
- ✅ Supports external operators

### Phase 2 Costs
- 🔨 2-3 months development
- 🔨 Additional operational complexity (orchestrator binary)
- 🔨 WASM runtime overhead (~5-10% during migration)
- 🔨 Increased on-chain data for migrations

### Phase 3 Benefits
- ✅ Zero coordination overhead
- ✅ Self-healing network
- ✅ Gradual upgrades (late adopters have time)
- ✅ Production-grade UX

### Phase 3 Costs
- 🔨 Additional 2-3 months development
- 🔨 More complex consensus logic
- 🔨 Grace period may delay finality for stragglers

---

## Open Questions & Future Considerations

### 1. Cross-Subnet Upgrade Coordination

**Question:** If a parent subnet upgrades, should child subnets also upgrade?

**Options:**
- A) Independent (children can run old version if compatible)
- B) Forced (parent upgrade triggers child upgrades)
- C) Coordinated (parent signals intent, children have window to upgrade)

**Recommendation:** Option C with compatibility window

### 2. Emergency Rollback Across Network

**Question:** If 10% of validators fail to upgrade, should network roll back?

**Options:**
- A) Continue with 90% (forking risk)
- B) Automatic rollback if <95% success
- C) Emergency governance vote to decide

**Recommendation:** Option B with monitoring, Option C as override

### 3. Multi-Version Consensus (Advanced)

**Question:** Can network run multiple versions simultaneously?

This is Phase 4+ territory, requires:
- Version-aware state transitions
- Backward-compatible consensus messages
- Complex testing matrix

**Recommendation:** Defer until Phase 3 is proven in production

### 4. Upgrade Scheduling Across Time Zones

**Question:** Global validator set may prefer different upgrade windows

**Solution:** Readiness-based scheduling (Phase 3) naturally handles this
- Validators in Europe ready first (morning)
- Validators in US ready next (their morning)
- Network schedules when threshold reached globally

---

## Success Metrics

### Phase 1 Success Criteria
- ✓ 100% of test upgrades succeed on testnet
- ✓ Average downtime < 60 seconds
- ✓ Zero manual config edits required
- ✓ All validators signal ready before halt

### Phase 2 Success Criteria
- ✓ 95%+ of validators successfully auto-upgrade
- ✓ Rollback mechanism tested and working
- ✓ External subnet operators adopt new process
- ✓ Average downtime < 30 seconds

### Phase 3 Success Criteria
- ✓ Network self-coordinates 90%+ of upgrades
- ✓ Late validators successfully sync during grace period
- ✓ No manual coordination needed
- ✓ Community operates upgrades without core team

---

## Appendix A: Alternative Approaches Considered

### A1. Hot Swapping (Rejected)

**Idea:** Swap binary without halting node

**Why Rejected:**
- Extremely complex (process isolation, state transfer)
- High risk of state corruption
- Not worth benefit for ~30 second downtime

### A2. Blue-Green Validator Sets (Rejected)

**Idea:** Two validator sets, upgrade one at a time

**Why Rejected:**
- Requires 2x validators (expensive)
- Complex handoff logic
- Only eliminates downtime, not coordination problem

### A3. Docker-Based Upgrades (Considered)

**Idea:** Orchestrator pulls new Docker images

**Why Considered:**
- Clean isolation
- Standard deployment pattern
- Easy rollback

**Trade-offs:**
- Requires Docker (not all deployments use it)
- Slightly slower startup
- Additional dependency

**Decision:** Support both Docker and binary-based in orchestrator

---

## Appendix B: Glossary

**halt_height:** Block height where Fendermint exits for upgrade

**UpgradeScheduler:** Rust module that executes migrations at block heights

**UpgradeRegistry:** Smart contract tracking upgrade proposals

**Orchestrator:** Wrapper process managing Fendermint lifecycle

**Migration:** State transformation executed during upgrade

**Readiness threshold:** Percentage of validators needed to auto-schedule

**Grace period:** Blocks where old version can sync but not validate

**Diamond pattern:** EIP-2535 upgradable contract architecture

**Binary hash:** Cryptographic hash verifying binary authenticity

---

## Next Steps

### Immediate Actions (This Week)

1. **Review & Approve** this document with IPC team
2. **Create GitHub Issues** for Phase 1 tasks
3. **Set up test infrastructure** (4-validator testnet)
4. **Assign developers** to Phase 1 implementation

### Week 1 Kickoff

1. **Design review** for UpgradeRegistry contract
2. **Begin contract implementation**
3. **Set up monitoring/logging** for upgrade events
4. **Draft operator communications** for new process

### Ongoing

- Weekly sync on progress
- Update this doc as implementation reveals new requirements
- Gather operator feedback during Phase 1
- Begin Phase 2 design during Phase 1 implementation

---

**Document Maintainer:** IPC Core Team
**Last Updated:** November 3, 2025
**Next Review:** After Phase 1 completion

