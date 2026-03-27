# IPC Cross-Chain Token Bridge — Operator Runbook

This document covers everything needed to deploy, verify, and monitor the IPC cross-chain token bridge on testnets (Filecoin Calibration ↔ Ethereum Sepolia).

---

## Architecture overview

```
[User on Filecoin Calibration]
    │
    │  ERC20.approve + BridgeLock.lock(token, amount, recipient)
    ▼
[BridgeLock.sol] ──── IPC cross-message ────▶ [IPC Subnet]
                                                    │
                                          [bridge-relay WASM actor]
                                          validates + marks processed
                                          emits bridge-relay/relayed event
                                                    │
                                                    ▼
                                          [BridgeMint.sol on Ethereum Sepolia]
                                          mints WrappedToken to recipient
    │
    ▼
[User receives wrapped tokens on Ethereum Sepolia]
```

**Contracts:**
- `BridgeLock.sol` — Filecoin Calibration; locks ERC20 tokens, dispatches IPC message
- `BridgeMint.sol` — Ethereum Sepolia; receives IPC messages, mints wrapped tokens
- `WrappedToken.sol` — per-asset ERC20 on Ethereum; minted/burned by BridgeMint

**Actor:**
- `bridge-relay` — Rust WASM actor on IPC subnet; validates events, enforces replay protection

---

## Prerequisites

| Tool | Version | Install |
|---|---|---|
| Node.js | ≥ 18 | https://nodejs.org |
| pnpm | ≥ 9 | `npm i -g pnpm` |
| Foundry (forge, cast) | latest | `curl -L https://foundry.paradigm.xyz \| bash && foundryup` |
| jq | any | `apt install jq` / `brew install jq` |

**Wallet funding:**
- Deployer account needs testnet FIL on Calibration (faucet: https://faucet.calibration.fildev.network)
- Deployer account needs testnet ETH on Sepolia (faucet: https://sepoliafaucet.com)

---

## Step 1 — Clone and install

```bash
git clone https://github.com/consensus-shipyard/ipc.git
cd ipc
pnpm install
cd contracts && pnpm install
```

---

## Step 2 — Configure environment

```bash
cp .env.example .env
```

Edit `.env` and fill in:

| Variable | Description | Where to find |
|---|---|---|
| `PRIVATE_KEY` | Deployer private key (0x-prefixed) | Your wallet |
| `FILECOIN_RPC_URL` | Filecoin Calibration RPC | https://api.calibration.node.glif.io/rpc/v1 |
| `FILECOIN_IPC_GATEWAY` | IPC Gateway on Calibration | IPC deployment docs |
| `ETHEREUM_RPC_URL` | Ethereum Sepolia RPC | https://rpc.sepolia.org |
| `ETHEREUM_IPC_GATEWAY` | IPC Gateway on Sepolia | IPC deployment docs |
| `IPC_SUBNET_ROOT` | Filecoin Calibration chain ID | `314159` |
| `IPC_FEE` | IPC gateway fee in attoFIL | `10000000000000000` (0.01 FIL) |
| `FILECOIN_TOKEN_ADDRESS` | Existing ERC20 to bridge (optional) | Leave blank to deploy a test token |

---

## Step 3 — Deploy all contracts

```bash
make -f Makefile.bridge deploy-all
```

This single command:
1. Compiles all contracts with Foundry
2. Deploys a `TestToken` ERC20 on Calibration (if `FILECOIN_TOKEN_ADDRESS` is blank)
3. Deploys `BridgeLock` proxy on Filecoin Calibration
4. Deploys `WrappedToken` impl + `BridgeMint` proxy on Ethereum Sepolia
5. Registers the asset mapping (filecoin token → wrapped token)
6. Wires `BridgeLock` to point to `BridgeMint`
7. Writes `contracts/deployments/deployments.json`

**Expected output:**
```
════════════════════════════════════════════════════════════
  ✅ Deployment complete!
════════════════════════════════════════════════════════════
{
  "bridgeLock": "0x...",
  "bridgeMint": "0x...",
  "wrappedToken": "0x...",
  "filecoinToken": "0x...",
  ...
}
```

### Individual deploy commands

```bash
# Deploy only BridgeLock on Calibration
cd contracts
pnpm exec hardhat deploy-bridge-lock \
  --network calibration \
  --gateway $FILECOIN_IPC_GATEWAY \
  --dest-root 314159 \
  --dest-receiver 0x0000000000000000000000000000000000000001

# Deploy only BridgeMint on Sepolia
pnpm exec hardhat deploy-bridge-mint \
  --network sepolia \
  --gateway $ETHEREUM_IPC_GATEWAY \
  --src-root 314159 \
  --bridge-lock <BRIDGE_LOCK_ADDRESS>

# Wire BridgeLock → BridgeMint
pnpm exec hardhat set-bridge-destination \
  --network calibration \
  --bridge-lock <BRIDGE_LOCK_ADDRESS> \
  --dest-root 314159 \
  --dest-receiver <BRIDGE_MINT_ADDRESS>
```

---

## Step 4 — Deploy the bridge-relay WASM actor

The bridge-relay actor runs on the IPC subnet and relays `TokensLocked` events to `BridgeMint`.

```bash
# Build the WASM actor
cd fendermint/actors/bridge-relay
cargo build --target wasm32-unknown-unknown --release
# Output: target/wasm32-unknown-unknown/release/fendermint_actor_bridge_relay.wasm

# Deploy to IPC subnet (follow fendermint actor deployment docs)
# The actor constructor requires:
#   - bridge_lock_addr: BridgeLock address on Filecoin Calibration
#   - bridge_mint_addr: BridgeMint address on Ethereum Sepolia
#   - validation_rules: { min_amount: 0, max_amount: 0, allowed_tokens: [] }
```

See [`fendermint/actors/bridge-relay/src/shared.rs`](../../fendermint/actors/bridge-relay/src/shared.rs) for the full `ConstructorParams` type.

---

## Step 5 — Run smoke test

```bash
make -f Makefile.bridge smoke-test
```

Or with custom params:
```bash
SMOKE_AMOUNT=100000000000000000 \
SMOKE_RECIPIENT=0xYourEthereumAddress \
bash scripts/bridge/smoke-test.sh
```

**What it checks:**
1. Deployer has sufficient token balance on Filecoin
2. `approve` + `lock()` succeeds on Filecoin Calibration
3. Wrapped tokens appear in recipient's wallet on Ethereum Sepolia
4. Minted amount exactly matches locked amount
5. `transferId` is marked as processed in `BridgeMint` (replay protection confirmed)

**Expected output:**
```
════════════════════════════════════════════════════════════
  ✅ SMOKE TEST PASSED  (3 checks passed, 0 failed)
════════════════════════════════════════════════════════════
```

---

## Verification commands

```bash
# Check BridgeLock config
cast call $BRIDGE_LOCK --rpc-url $FILECOIN_RPC_URL "ipcFee()(uint256)"
cast call $BRIDGE_LOCK --rpc-url $FILECOIN_RPC_URL "destReceiver()(address)"

# Check BridgeMint config
cast call $BRIDGE_MINT --rpc-url $ETHEREUM_RPC_URL "bridgeLockAddr()(address)"
cast call $BRIDGE_MINT --rpc-url $ETHEREUM_RPC_URL "wrappedTokens(address)(address)" $FILECOIN_TOKEN

# Check if a transferId was processed
cast call $BRIDGE_MINT --rpc-url $ETHEREUM_RPC_URL "isProcessed(bytes32)(bool)" $TRANSFER_ID

# Check wrapped token balance
cast call $WRAPPED_TOKEN --rpc-url $ETHEREUM_RPC_URL "balanceOf(address)(uint256)" $RECIPIENT
```

---

## Emergency procedures

### Pause the bridge (halt all new locks)
```bash
cast send $BRIDGE_LOCK "pause()" \
  --rpc-url $FILECOIN_RPC_URL --private-key $PRIVATE_KEY
cast send $BRIDGE_MINT "pause()" \
  --rpc-url $ETHEREUM_RPC_URL --private-key $PRIVATE_KEY
```

### Unpause
```bash
cast send $BRIDGE_LOCK "unpause()" \
  --rpc-url $FILECOIN_RPC_URL --private-key $PRIVATE_KEY
cast send $BRIDGE_MINT "unpause()" \
  --rpc-url $ETHEREUM_RPC_URL --private-key $PRIVATE_KEY
```

### Rescue stuck tokens (after confirming cross-chain leg failed)
```bash
# Rescue ERC20 tokens stuck in BridgeLock
cast send $BRIDGE_LOCK \
  "rescueTokens(address,address,uint256)" $TOKEN $RECIPIENT $AMOUNT \
  --rpc-url $FILECOIN_RPC_URL --private-key $PRIVATE_KEY
```

---

## Gas reference (measured on testnets)

| Operation | Approximate gas |
|---|---|
| `BridgeLock.lock()` | ~120,000 gas |
| `BridgeMint` mint via IPC | ~90,000 gas |
| `BridgeLock` proxy deploy | ~2,800,000 gas |
| `BridgeMint` proxy deploy | ~2,400,000 gas |

*Gas costs vary with network congestion. Measure for your deployment with `forge test --gas-report`.*

---

## Mainnet upgrade path

1. **Audit first** — both contracts pass Slither static analysis. A full third-party audit is recommended before mainnet.
2. **Key management** — replace deployer EOA with a multisig (e.g. Safe) for `DEFAULT_ADMIN_ROLE` on both contracts.
3. **IPC fee tuning** — measure actual gateway costs and set `ipcFee` accordingly.
4. **Token allowlist** — enable `tokenAllowlistEnabled` on BridgeLock and whitelist only audited tokens.
5. **Timelock** — add a timelock governor to the UUPS upgrade path before mainnet deployment.
6. **Monitoring** — subscribe to `TokensLocked` (Filecoin) and `TokensMinted` (Ethereum) events for real-time alerting.
