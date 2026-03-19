/**
 * @file types.ts
 * Public type exports for the IPC Bridge SDK.
 */

// ─── Configuration ────────────────────────────────────────────────────────────

/** RPC and contract configuration for both chains. */
export interface BridgeConfig {
  /** JSON-RPC URL for Filecoin Calibration (e.g. https://api.calibration.node.glif.io/rpc/v1) */
  filecoinRpc: string;
  /** JSON-RPC URL for Ethereum Sepolia (e.g. https://rpc.sepolia.org) */
  ethereumRpc: string;
  /** Deployed BridgeLock proxy address on Filecoin Calibration. */
  bridgeLockAddress: string;
  /** Deployed BridgeMint proxy address on Ethereum Sepolia. */
  bridgeMintAddress: string;
}

// ─── lockTokens ───────────────────────────────────────────────────────────────

/** Parameters for initiating a cross-chain transfer via lockTokens(). */
export interface LockParams {
  /** ERC20 token contract address on Filecoin to lock. */
  tokenAddress: string;
  /** Amount to lock, in the token's smallest unit (use ethers.parseUnits for human amounts). */
  amount: bigint;
  /** Recipient address on Ethereum that will receive the minted wrapped tokens. */
  recipient: string;
  /** IPC fee (in wei/attoFIL) forwarded to the gateway for cross-chain dispatch.
   *  Defaults to the contract's current ipcFee if omitted. */
  ipcFee?: bigint;
}

/** Receipt returned by lockTokens() after the lock transaction is mined. */
export interface TransferReceipt {
  /** Unique 32-byte transfer identifier (hex string, 0x-prefixed). */
  transferId: string;
  /** The lock transaction hash on Filecoin. */
  lockTxHash: string;
  /** Block number of the lock transaction on Filecoin. */
  lockBlock: number;
  /** Timestamp (unix seconds) of the lock block. */
  lockTimestamp: number;
  /** Amount locked (in token smallest units). */
  amount: bigint;
  /** Recipient address on Ethereum. */
  recipient: string;
  /** Token address on Filecoin. */
  tokenAddress: string;
}

// ─── getTransferStatus / waitForCompletion ────────────────────────────────────

/** The lifecycle state of a cross-chain transfer. */
export type TransferState =
  | "locked"       // Lock tx mined on Filecoin; awaiting IPC relay
  | "relaying"     // IPC actor has picked it up; mint tx pending on Ethereum
  | "minted"       // Mint confirmed on Ethereum
  | "failed"       // Mint failed or timed out
  | "unknown";     // transferId not found on either chain

/** Full status snapshot for a transfer. */
export interface TransferStatus {
  transferId: string;
  state: TransferState;
  /** Lock tx hash (always populated if state != 'unknown'). */
  lockTxHash?: string;
  /** Mint tx hash (populated once state == 'minted'). */
  mintTxHash?: string;
  /** Amount in token smallest units. */
  amount?: bigint;
  /** Recipient on Ethereum. */
  recipient?: string;
  /** Token address on Filecoin. */
  tokenAddress?: string;
  /** WrappedToken address on Ethereum (populated once minted). */
  wrappedTokenAddress?: string;
  /** Unix timestamp when the lock was mined. */
  lockedAt?: number;
  /** Unix timestamp when the mint was confirmed. */
  mintedAt?: number;
}

// ─── waitForCompletion ────────────────────────────────────────────────────────

/** Options for waitForCompletion(). */
export interface WaitOpts {
  /** Maximum time to wait in milliseconds. Default: 300_000 (5 minutes). */
  timeoutMs?: number;
  /** Poll interval in milliseconds. Default: 5_000 (5 seconds). */
  pollIntervalMs?: number;
  /** Callback invoked on each poll with the latest status. */
  onPoll?: (status: TransferStatus) => void;
}

// ─── Event types ─────────────────────────────────────────────────────────────

/** Decoded TokensLocked event from BridgeLock.sol. */
export interface TokensLockedEvent {
  tokenAddress: string;
  sender: string;
  recipient: string;
  amount: bigint;
  transferId: string;
  blockNumber: number;
  txHash: string;
}

/** Decoded TokensMinted event from BridgeMint.sol. */
export interface TokensMintedEvent {
  wrappedTokenAddress: string;
  recipient: string;
  amount: bigint;
  transferId: string;
  blockNumber: number;
  txHash: string;
}
