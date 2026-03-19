/**
 * @file client.ts
 * BridgeClient — the main entry point for the IPC Bridge SDK.
 *
 * @example
 * ```ts
 * import { BridgeClient } from "@ipc-network/bridge-sdk";
 * import { ethers } from "ethers";
 *
 * const client = new BridgeClient({
 *   filecoinRpc: "https://api.calibration.node.glif.io/rpc/v1",
 *   ethereumRpc: "https://rpc.sepolia.org",
 *   bridgeLockAddress: "0x...",
 *   bridgeMintAddress: "0x...",
 * });
 *
 * const signer = new ethers.Wallet(process.env.PRIVATE_KEY!, client.filecoinProvider);
 *
 * // Approve first
 * const erc20 = new ethers.Contract(TOKEN_ADDR, erc20Abi, signer);
 * await erc20.approve(client.config.bridgeLockAddress, amount);
 *
 * // Lock and relay
 * const receipt = await client.lockTokens({
 *   tokenAddress: TOKEN_ADDR,
 *   amount: ethers.parseUnits("100", 18),
 *   recipient: "0xEthereumRecipient",
 * }, signer);
 *
 * const status = await client.waitForCompletion(receipt.transferId);
 * console.log("Minted at tx:", status.mintTxHash);
 * ```
 */

import { ethers } from "ethers";
import BridgeLockAbi from "./abis/BridgeLock.json" assert { type: "json" };
import BridgeMintAbi from "./abis/BridgeMint.json" assert { type: "json" };
import type {
  BridgeConfig,
  LockParams,
  TokensLockedEvent,
  TokensMintedEvent,
  TransferReceipt,
  TransferState,
  TransferStatus,
  WaitOpts,
} from "./types.js";

// ─── Constants ────────────────────────────────────────────────────────────────

const DEFAULT_TIMEOUT_MS = 5 * 60 * 1000; // 5 minutes
const DEFAULT_POLL_INTERVAL_MS = 5_000;    // 5 seconds
const TOKENS_LOCKED_EVENT = "TokensLocked";
const TOKENS_MINTED_EVENT = "TokensMinted";

// ─── BridgeClient ─────────────────────────────────────────────────────────────

export class BridgeClient {
  /** Read-only config (set at construction time). */
  readonly config: Readonly<BridgeConfig>;

  /** ethers.js provider for Filecoin Calibration (read-only). */
  readonly filecoinProvider: ethers.JsonRpcProvider;

  /** ethers.js provider for Ethereum Sepolia (read-only). */
  readonly ethereumProvider: ethers.JsonRpcProvider;

  /** Read-only BridgeLock contract instance (Filecoin side). */
  private readonly bridgeLock: ethers.Contract;

  /** Read-only BridgeMint contract instance (Ethereum side). */
  private readonly bridgeMint: ethers.Contract;

  constructor(config: BridgeConfig) {
    this.config = Object.freeze({ ...config });
    this.filecoinProvider = new ethers.JsonRpcProvider(config.filecoinRpc);
    this.ethereumProvider = new ethers.JsonRpcProvider(config.ethereumRpc);
    this.bridgeLock = new ethers.Contract(
      config.bridgeLockAddress,
      BridgeLockAbi,
      this.filecoinProvider,
    );
    this.bridgeMint = new ethers.Contract(
      config.bridgeMintAddress,
      BridgeMintAbi,
      this.ethereumProvider,
    );
  }

  // ─── lockTokens ─────────────────────────────────────────────────────────────

  /**
   * Lock ERC20 tokens on Filecoin and initiate a cross-chain transfer.
   *
   * @param params  Transfer parameters (token, amount, recipient, optional ipcFee).
   * @param signer  An ethers Signer connected to Filecoin Calibration.
   * @returns       A TransferReceipt containing the transferId and lock tx details.
   *
   * @throws If the lock transaction reverts or the receipt cannot be parsed.
   *
   * Note: The caller is responsible for calling `token.approve(bridgeLockAddress, amount)`
   * before this method, and for ensuring msg.value >= ipcFee.
   */
  async lockTokens(
    params: LockParams,
    signer: ethers.Signer,
  ): Promise<TransferReceipt> {
    const { tokenAddress, amount, recipient } = params;

    if (!ethers.isAddress(tokenAddress)) {
      throw new Error(`Invalid tokenAddress: ${tokenAddress}`);
    }
    if (!ethers.isAddress(recipient)) {
      throw new Error(`Invalid recipient: ${recipient}`);
    }
    if (amount <= 0n) {
      throw new Error("amount must be > 0");
    }

    // Resolve IPC fee from contract if not provided
    const ipcFee = params.ipcFee ?? (await this.bridgeLock.ipcFee()) as bigint;

    // Connect contract to the signer (Filecoin side)
    const bridgeLockSigned = this.bridgeLock.connect(signer) as ethers.Contract;

    // Submit lock transaction
    const tx: ethers.TransactionResponse = await bridgeLockSigned.lock(
      tokenAddress,
      amount,
      recipient,
      { value: ipcFee },
    );

    const receipt = await tx.wait();
    if (!receipt) throw new Error("Lock transaction returned null receipt");

    // Parse the TokensLocked event from the receipt
    const lockEvent = this._parseLockEvent(receipt);
    if (!lockEvent) {
      throw new Error(
        `TokensLocked event not found in tx ${tx.hash}. ` +
        "Ensure the transaction was mined and the ABI matches.",
      );
    }

    const block = await this.filecoinProvider.getBlock(receipt.blockNumber);

    return {
      transferId: lockEvent.transferId,
      lockTxHash: tx.hash,
      lockBlock: receipt.blockNumber,
      lockTimestamp: block?.timestamp ?? 0,
      amount: lockEvent.amount,
      recipient: lockEvent.recipient,
      tokenAddress: lockEvent.tokenAddress,
    };
  }

  // ─── getTransferStatus ────────────────────────────────────────────────────

  /**
   * Query the current status of a cross-chain transfer.
   *
   * Checks both chains:
   * - Looks for a `TokensMinted` event on Ethereum with the given transferId → "minted"
   * - Checks `processedTransfers(transferId)` on BridgeMint → "relaying" if true but no event yet
   * - Looks for a `TokensLocked` event on Filecoin → "locked"
   * - Otherwise → "unknown"
   *
   * @param transferId  0x-prefixed 32-byte hex string from TransferReceipt.transferId.
   */
  async getTransferStatus(transferId: string): Promise<TransferStatus> {
    const normalizedId = this._normalizeTransferId(transferId);

    // 1. Check for mint on Ethereum first (fastest resolution)
    const mintEvent = await this._findMintEvent(normalizedId);
    if (mintEvent) {
      const mintBlock = await this.ethereumProvider.getBlock(mintEvent.blockNumber);
      // Also find the original lock for full context
      const lockEvent = await this._findLockEvent(normalizedId);
      return {
        transferId: normalizedId,
        state: "minted",
        lockTxHash: lockEvent?.txHash,
        mintTxHash: mintEvent.txHash,
        amount: mintEvent.amount,
        recipient: mintEvent.recipient,
        tokenAddress: lockEvent?.tokenAddress,
        wrappedTokenAddress: mintEvent.wrappedTokenAddress,
        lockedAt: lockEvent ? await this._txTimestamp("filecoin", lockEvent.txHash) : undefined,
        mintedAt: mintBlock?.timestamp,
      };
    }

    // 2. Check if BridgeMint has it recorded (relay in progress)
    const isProcessed = await this.bridgeMint.isProcessed(normalizedId) as boolean;
    if (isProcessed) {
      const lockEvent = await this._findLockEvent(normalizedId);
      return {
        transferId: normalizedId,
        state: "relaying",
        lockTxHash: lockEvent?.txHash,
        amount: lockEvent?.amount,
        recipient: lockEvent?.recipient,
        tokenAddress: lockEvent?.tokenAddress,
        lockedAt: lockEvent
          ? await this._txTimestamp("filecoin", lockEvent.txHash)
          : undefined,
      };
    }

    // 3. Check for lock on Filecoin
    const lockEvent = await this._findLockEvent(normalizedId);
    if (lockEvent) {
      return {
        transferId: normalizedId,
        state: "locked",
        lockTxHash: lockEvent.txHash,
        amount: lockEvent.amount,
        recipient: lockEvent.recipient,
        tokenAddress: lockEvent.tokenAddress,
        lockedAt: await this._txTimestamp("filecoin", lockEvent.txHash),
      };
    }

    return { transferId: normalizedId, state: "unknown" };
  }

  // ─── waitForCompletion ────────────────────────────────────────────────────

  /**
   * Poll until the transfer reaches "minted" or "failed" state, or times out.
   *
   * @param transferId  0x-prefixed 32-byte hex string.
   * @param opts        Timeout, poll interval, and progress callback.
   * @returns           The final TransferStatus when resolved.
   *
   * @throws {Error}  If the timeout is exceeded before the transfer completes.
   */
  async waitForCompletion(
    transferId: string,
    opts: WaitOpts = {},
  ): Promise<TransferStatus> {
    const {
      timeoutMs = DEFAULT_TIMEOUT_MS,
      pollIntervalMs = DEFAULT_POLL_INTERVAL_MS,
      onPoll,
    } = opts;

    const deadline = Date.now() + timeoutMs;

    while (Date.now() < deadline) {
      const status = await this.getTransferStatus(transferId);
      onPoll?.(status);

      if (status.state === "minted" || status.state === "failed") {
        return status;
      }

      const remaining = deadline - Date.now();
      if (remaining <= 0) break;
      await sleep(Math.min(pollIntervalMs, remaining));
    }

    throw new Error(
      `waitForCompletion: timed out after ${timeoutMs}ms for transferId ${transferId}`,
    );
  }

  // ─── Event subscriptions ──────────────────────────────────────────────────

  /**
   * Subscribe to TokensLocked events from BridgeLock on Filecoin.
   *
   * @param handler   Called with each new event.
   * @returns         A cleanup function — call it to unsubscribe.
   */
  onTokensLocked(
    handler: (event: TokensLockedEvent) => void,
  ): () => void {
    const listener = (...args: unknown[]) => {
      const e = args[args.length - 1] as ethers.EventLog;
      const decoded = this.bridgeLock.interface.parseLog({
        topics: e.topics as string[],
        data: e.data,
      });
      if (!decoded) return;
      handler({
        tokenAddress: decoded.args[0] as string,
        sender: decoded.args[1] as string,
        recipient: decoded.args[2] as string,
        amount: decoded.args[3] as bigint,
        transferId: decoded.args[4] as string,
        blockNumber: e.blockNumber,
        txHash: e.transactionHash,
      });
    };
    this.bridgeLock.on(TOKENS_LOCKED_EVENT, listener);
    return () => { this.bridgeLock.off(TOKENS_LOCKED_EVENT, listener); };
  }

  /**
   * Subscribe to TokensMinted events from BridgeMint on Ethereum.
   *
   * @param handler   Called with each new event.
   * @returns         A cleanup function — call it to unsubscribe.
   */
  onTokensMinted(
    handler: (event: TokensMintedEvent) => void,
  ): () => void {
    const listener = (...args: unknown[]) => {
      const e = args[args.length - 1] as ethers.EventLog;
      const decoded = this.bridgeMint.interface.parseLog({
        topics: e.topics as string[],
        data: e.data,
      });
      if (!decoded) return;
      handler({
        wrappedTokenAddress: decoded.args[0] as string,
        recipient: decoded.args[1] as string,
        amount: decoded.args[2] as bigint,
        transferId: decoded.args[3] as string,
        blockNumber: e.blockNumber,
        txHash: e.transactionHash,
      });
    };
    this.bridgeMint.on(TOKENS_MINTED_EVENT, listener);
    return () => { this.bridgeMint.off(TOKENS_MINTED_EVENT, listener); };
  }

  // ─── Helpers: event scanning ──────────────────────────────────────────────

  private async _findLockEvent(
    transferId: string,
  ): Promise<TokensLockedEvent | null> {
    try {
      // Filter by the transferId (4th indexed topic in TokensLocked)
      // TokensLocked(address indexed token, address indexed sender, address indexed recipient, uint256 amount, bytes32 transferId)
      // transferId is NOT indexed, so we must scan logs and filter
      const filter = this.bridgeLock.filters[TOKENS_LOCKED_EVENT]();
      const logs = await this.bridgeLock.queryFilter(filter, 0, "latest");
      for (const log of logs) {
        const el = log as ethers.EventLog;
        const tid = el.args[4] as string;
        if (tid.toLowerCase() === transferId.toLowerCase()) {
          return {
            tokenAddress: el.args[0] as string,
            sender: el.args[1] as string,
            recipient: el.args[2] as string,
            amount: el.args[3] as bigint,
            transferId: tid,
            blockNumber: el.blockNumber,
            txHash: el.transactionHash,
          };
        }
      }
    } catch {
      // Network errors: return null and let caller handle
    }
    return null;
  }

  private async _findMintEvent(
    transferId: string,
  ): Promise<TokensMintedEvent | null> {
    try {
      // TokensMinted(address indexed token, address indexed recipient, uint256 amount, bytes32 indexed transferId)
      // transferId IS indexed (3rd indexed topic)
      const filter = this.bridgeMint.filters[TOKENS_MINTED_EVENT](
        null, null, transferId,
      );
      const logs = await this.bridgeMint.queryFilter(filter, 0, "latest");
      if (logs.length === 0) return null;
      const el = logs[0] as ethers.EventLog;
      return {
        wrappedTokenAddress: el.args[0] as string,
        recipient: el.args[1] as string,
        amount: el.args[2] as bigint,
        transferId: el.args[3] as string,
        blockNumber: el.blockNumber,
        txHash: el.transactionHash,
      };
    } catch {
      return null;
    }
  }

  // ─── Helpers: receipt parsing ─────────────────────────────────────────────

  private _parseLockEvent(
    receipt: ethers.TransactionReceipt,
  ): TokensLockedEvent | null {
    for (const log of receipt.logs) {
      try {
        const parsed = this.bridgeLock.interface.parseLog({
          topics: log.topics as string[],
          data: log.data,
        });
        if (parsed?.name === TOKENS_LOCKED_EVENT) {
          return {
            tokenAddress: parsed.args[0] as string,
            sender: parsed.args[1] as string,
            recipient: parsed.args[2] as string,
            amount: parsed.args[3] as bigint,
            transferId: parsed.args[4] as string,
            blockNumber: receipt.blockNumber,
            txHash: receipt.hash,
          };
        }
      } catch {
        // Log from a different contract — skip
      }
    }
    return null;
  }

  // ─── Helpers: misc ────────────────────────────────────────────────────────

  private async _txTimestamp(
    chain: "filecoin" | "ethereum",
    txHash: string,
  ): Promise<number | undefined> {
    try {
      const provider =
        chain === "filecoin" ? this.filecoinProvider : this.ethereumProvider;
      const tx = await provider.getTransaction(txHash);
      if (!tx?.blockNumber) return undefined;
      const block = await provider.getBlock(tx.blockNumber);
      return block?.timestamp;
    } catch {
      return undefined;
    }
  }

  private _normalizeTransferId(transferId: string): string {
    const stripped = transferId.replace(/^0x/i, "");
    const hex = `0x${stripped}`;
    if (!/^0x[0-9a-fA-F]{64}$/.test(hex)) {
      throw new Error(
        `Invalid transferId: expected 0x-prefixed 32-byte hex string, got "${transferId}"`,
      );
    }
    return hex.toLowerCase();
  }
}

// ─── Internal utilities ───────────────────────────────────────────────────────

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
