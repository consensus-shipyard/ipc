/**
 * @ipc-network/bridge-sdk
 *
 * TypeScript SDK for initiating and monitoring IPC cross-chain token bridge transfers
 * between Filecoin Calibration and Ethereum Sepolia.
 *
 * @example
 * ```ts
 * import { BridgeClient } from "@ipc-network/bridge-sdk";
 *
 * const client = new BridgeClient({ ... });
 * const receipt = await client.lockTokens({ ... }, signer);
 * const status  = await client.waitForCompletion(receipt.transferId);
 * ```
 */

export { BridgeClient } from "./client.js";
export type {
  BridgeConfig,
  LockParams,
  TransferReceipt,
  TransferStatus,
  TransferState,
  WaitOpts,
  TokensLockedEvent,
  TokensMintedEvent,
} from "./types.js";
export { BridgeLockAbi, BridgeMintAbi, WrappedTokenAbi } from "./abis/index.js";
