# IPC Library Extraction - Quick Summary

## The Problem

**Current situation:**
- ~40% code duplication between CLI and fendermint
- Tight coupling between components
- Hard for third parties to integrate IPC functionality
- Unclear architectural boundaries

**Impact:**
- Maintenance burden (fix bugs in multiple places)
- Larger binaries (CLI includes fendermint dependencies)
- Inconsistent behavior across tools

---

## The Solution

Extract shared IPC functionality into `ipc-lib` - a high-level, well-documented library.

### Before
```
ipc-cli ──┬──> ipc-provider
          ├──> ipc-api
          └──> fendermint (genesis, deployer)

fendermint ──┬──> ipc-provider
             └──> ipc-api
```

### After
```
ipc-cli ────┐
            ├──> ipc-lib ──┬──> ipc-core
fendermint ─┘              ├──> ipc-contracts
                           └──> ipc-api
```

---

## What Goes Into ipc-lib

### 6 Core Modules

1. **`subnet`** - Subnet operations (create, join, leave, list)
2. **`checkpoint`** - Checkpoint management (create, submit, validate)
3. **`crossmsg`** - Cross-chain messaging (send, fund, propagate)
4. **`gateway`** - Gateway interactions (deploy, register, fund)
5. **`genesis`** - Genesis file creation (builder pattern)
6. **`config`** - Configuration management (load, save, query)

### What Stays Where

**Stays in CLI:**
- Command-line parsing
- Terminal UI
- Interactive prompts
- CLI services

**Stays in Fendermint:**
- ABCI application
- FVM interpreter
- State machine
- Actor implementations
- Block production

**Moves to ipc-lib:**
- All subnet operations
- Checkpoint logic
- Cross-chain messaging
- Genesis building
- Contract deployment

---

## API Preview

### Simple & Clean

```rust
// Create client
let client = IpcClient::builder()
    .network(NetworkType::Calibration)
    .rpc_url("https://api.node.glif.io")
    .wallet_path("~/.ipc/wallet")
    .build()
    .await?;

// Create subnet (was 50+ lines, now 5)
let subnet = client
    .subnet()
    .create()
    .name("my-subnet")
    .min_validators(3)
    .stake_requirement(TokenAmount::from_fil(10))
    .execute()
    .await?;

// Submit checkpoint (was 30+ lines, now 3)
let checkpoint = client.checkpoint().create_from_height(subnet_id, height).await?;
let tx = client.checkpoint().submit(checkpoint).await?;

// Genesis builder
let genesis = GenesisBuilder::new("my-chain")
    .add_validator(validator)
    .add_account(account)
    .build()?;
```

---

## Implementation Plan

### Timeline: 6 Weeks

| Week | Phase | Focus |
|------|-------|-------|
| 1 | Setup | Library structure, API design |
| 1-2 | Core | RPC clients, config, errors |
| 2-3 | Subnet | Extract subnet operations |
| 3-4 | Checkpoint | Checkpoint & cross-chain messaging |
| 4-5 | Genesis | Genesis & gateway management |
| 5-6 | Migration | Update CLI and fendermint |
| 6+ | Polish | Documentation, examples |

### Phases

1. **Phase 1:** Setup (1 week)
2. **Phase 2:** Extract types & utils (1 week)
3. **Phase 3:** Extract subnet ops (1 week)
4. **Phase 4:** Extract checkpoint & crossmsg (1 week)
5. **Phase 5:** Extract genesis & gateway (1 week)
6. **Phase 6:** Refactor CLI (0.5 week)
7. **Phase 7:** Refactor fendermint (0.5 week)
8. **Phase 8:** Documentation (ongoing)

---

## Benefits

### Quantifiable

- **35% reduction** in duplicated code
- **20% smaller** CLI binary
- **~60% less code** per CLI command
- **Single source** of truth for IPC operations

### Qualitative

- ✅ Clearer architecture
- ✅ Better testing (mockable APIs)
- ✅ Third-party integrations enabled
- ✅ Easier maintenance
- ✅ Comprehensive documentation

---

## Risks & Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| Breaking changes | High | Backward compat layer, gradual rollout |
| Performance | Medium | Benchmarking, profiling |
| API design | Medium | Early feedback, iteration |
| Migration issues | Medium | Comprehensive tests, docs |

---

## Success Criteria

- [ ] All CLI commands work with ipc-lib
- [ ] All fendermint operations work with ipc-lib
- [ ] 80%+ test coverage
- [ ] Complete API documentation
- [ ] 5+ working examples
- [ ] No performance regression
- [ ] Migration guide published

---

## Example: Before vs After

### Creating a Subnet

**Before (50+ lines in CLI):**
```rust
let provider = EvmSubnetManager::new(gateway, registry);
let config = SubnetConfig {
    name: args.name,
    min_validators: args.min_validators,
    min_validator_stake: args.stake,
    bottom_up_check_period: args.check_period,
    active_validators_limit: args.validators_limit,
    // ... 15 more fields
};
let tx = provider.create_subnet(config).await?;
let receipt = provider.wait_for_transaction(tx).await?;
let subnet_id = extract_subnet_id_from_logs(receipt)?;
// ... error handling, logging ...
```

**After (5 lines):**
```rust
let subnet = client
    .subnet()
    .create()
    .name(args.name)
    .min_validators(args.min_validators)
    .execute()
    .await?;
```

---

## File Structure

```
ipc/
├── api/           (existing)
├── types/         (existing)
├── wallet/        (existing)
├── core/          (refactored from provider)
└── lib/           (NEW)
    ├── subnet.rs
    ├── checkpoint.rs
    ├── crossmsg.rs
    ├── gateway.rs
    ├── genesis.rs
    ├── config.rs
    ├── contracts.rs
    └── tests/
        ├── subnet_tests.rs
        ├── checkpoint_tests.rs
        └── integration/
```

---

## Rollout

### Version Schedule

- **v0.1.0-alpha** (Week 4): Core modules, internal testing
- **v0.1.0-beta** (Week 5): CLI migrated, external testing
- **v0.1.0-rc** (Week 6): Everything migrated, docs complete
- **v0.1.0** (Week 7): Stable release, backward compat
- **v0.2.0** (Week 8+): Remove deprecated APIs

---

## Code Size Impact

| Component | Before | After | Change |
|-----------|--------|-------|--------|
| ipc-provider | 8,000 | 6,000 (core) | -25% |
| ipc-cli | 15,000 | 10,000 | -33% |
| fendermint (IPC) | 5,000 | 3,500 | -30% |
| **ipc-lib (NEW)** | 0 | 12,000 | +100% |
| **Total** | 28,000 | 31,500 | +13% |

**Net result:** Slight increase in total code, but massive reduction in duplication.

---

## Next Steps

1. **Review** this design doc with team
2. **Get buy-in** from stakeholders
3. **Create** GitHub issue for tracking
4. **Start Phase 1** - library structure setup
5. **Iterate** on API design with early feedback

---

## FAQ

**Q: Why not just clean up ipc-provider?**
A: Provider is low-level and tightly coupled. We need a high-level abstraction layer.

**Q: Will this break existing code?**
A: We'll maintain backward compatibility for at least one release cycle.

**Q: How much effort to migrate?**
A: CLI commands become ~60% shorter. Fendermint changes are minimal.

**Q: What about performance?**
A: Negligible overhead (~1-2%). We'll benchmark to confirm.

**Q: Can third parties use this?**
A: Yes! That's a key goal. Clean API + docs + examples.

**Q: What if we need to revert?**
A: Backward compat layer stays for 1+ releases. Low risk.

---

**Summary Version:** 1.0
**Created:** December 4, 2024
**For Full Details:** See `IPC_LIB_EXTRACTION_DESIGN.md`
