# @ipc-network/bridge-sdk

TypeScript SDK for the IPC cross-chain token bridge between **Filecoin Calibration** and **Ethereum Sepolia**.

## Install

```bash
npm install @ipc-network/bridge-sdk ethers
# or
pnpm add @ipc-network/bridge-sdk ethers
```

## Quickstart

```ts
import { BridgeClient } from "@ipc-network/bridge-sdk";
import { ethers } from "ethers";

// 1. Create client
const client = new BridgeClient({
  filecoinRpc:       "https://api.calibration.node.glif.io/rpc/v1",
  ethereumRpc:       "https://rpc.sepolia.org",
  bridgeLockAddress: "0x<BRIDGE_LOCK_PROXY>",
  bridgeMintAddress: "0x<BRIDGE_MINT_PROXY>",
});

// 2. Set up a signer on Filecoin Calibration
const signer = new ethers.Wallet(process.env.PRIVATE_KEY!, client.filecoinProvider);

// 3. Approve the BridgeLock contract to spend your tokens
const erc20 = new ethers.Contract(TOKEN_ADDRESS, [
  "function approve(address spender, uint256 amount) returns (bool)"
], signer);
await erc20.approve(client.config.bridgeLockAddress, ethers.parseUnits("100", 18));

// 4. Lock tokens and initiate the bridge transfer
const receipt = await client.lockTokens({
  tokenAddress: TOKEN_ADDRESS,
  amount:       ethers.parseUnits("100", 18),
  recipient:    "0x<ETHEREUM_RECIPIENT>",
}, signer);

console.log("Transfer ID:", receipt.transferId);
console.log("Lock tx:", receipt.lockTxHash);

// 5. Wait for the mint to complete on Ethereum (~2–5 min on testnets)
const status = await client.waitForCompletion(receipt.transferId, {
  timeoutMs:      10 * 60 * 1000, // 10 minutes
  pollIntervalMs: 10_000,          // poll every 10s
  onPoll: (s) => console.log("Current state:", s.state),
});

console.log("Minted at tx:", status.mintTxHash);
```

## API

### `new BridgeClient(config: BridgeConfig)`

| Field | Type | Description |
|---|---|---|
| `filecoinRpc` | `string` | JSON-RPC URL for Filecoin Calibration |
| `ethereumRpc` | `string` | JSON-RPC URL for Ethereum Sepolia |
| `bridgeLockAddress` | `string` | Deployed BridgeLock proxy address |
| `bridgeMintAddress` | `string` | Deployed BridgeMint proxy address |

### `lockTokens(params, signer) → Promise<TransferReceipt>`

Initiates a cross-chain transfer. Calls `BridgeLock.lock()` on Filecoin.

**Params:**
- `tokenAddress` — ERC20 token address on Filecoin
- `amount` — Amount in token smallest units (`bigint`)
- `recipient` — Ethereum recipient address
- `ipcFee?` — Override the IPC gateway fee (defaults to `BridgeLock.ipcFee()`)

**Pre-condition:** Call `token.approve(bridgeLockAddress, amount)` first.

**Returns:** `TransferReceipt` with `transferId`, `lockTxHash`, `lockBlock`, `lockTimestamp`, `amount`, `recipient`, `tokenAddress`.

### `getTransferStatus(transferId) → Promise<TransferStatus>`

Queries the current state across both chains:

| State | Meaning |
|---|---|
| `"locked"` | Lock tx mined on Filecoin; relay pending |
| `"relaying"` | BridgeMint has recorded the transferId; mint tx pending |
| `"minted"` | Wrapped tokens minted on Ethereum ✓ |
| `"failed"` | Mint failed |
| `"unknown"` | transferId not found on either chain |

### `waitForCompletion(transferId, opts?) → Promise<TransferStatus>`

Polls `getTransferStatus` until `"minted"` or `"failed"`, or throws on timeout.

**Options:**
- `timeoutMs` — Default: 300,000 ms (5 min)
- `pollIntervalMs` — Default: 5,000 ms (5 sec)
- `onPoll` — Progress callback `(status: TransferStatus) => void`

### Event subscriptions

```ts
// Subscribe to lock events on Filecoin
const unsubLock = client.onTokensLocked((event) => {
  console.log("Locked:", event.transferId, event.amount);
});

// Subscribe to mint events on Ethereum
const unsubMint = client.onTokensMinted((event) => {
  console.log("Minted:", event.transferId, event.amount);
});

// Cleanup
unsubLock();
unsubMint();
```

## Development

```bash
# Install deps
pnpm install

# Type-check
pnpm typecheck

# Build
pnpm build

# Run tests
pnpm test
```

## ABI updates

If contracts are redeployed or upgraded, regenerate ABIs:

```bash
cd contracts && forge build
cp out/BridgeLock.sol/BridgeLock.json sdk/bridge/src/abis/BridgeLock.json
cp out/BridgeMint.sol/BridgeMint.json sdk/bridge/src/abis/BridgeMint.json
cp out/WrappedToken.sol/WrappedToken.json sdk/bridge/src/abis/WrappedToken.json
```
