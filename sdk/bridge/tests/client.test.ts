/**
 * Unit tests for BridgeClient.
 *
 * All network calls are mocked — no real RPC endpoints required.
 * Vitest is used as the test runner.
 */

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { ethers } from "ethers";
import { BridgeClient } from "../src/client.js";
import type { BridgeConfig, TransferStatus } from "../src/types.js";

// ─── Shared test config ────────────────────────────────────────────────────────

const LOCK_ADDR  = "0x" + "a".repeat(40);
const MINT_ADDR  = "0x" + "b".repeat(40);
const TOKEN_ADDR = "0x" + "c".repeat(40);
const RECIPIENT  = "0x" + "d".repeat(40);
const TRANSFER_ID = "0x" + "e".repeat(64);

const config: BridgeConfig = {
  filecoinRpc: "http://localhost:8545",
  ethereumRpc: "http://localhost:8546",
  bridgeLockAddress: LOCK_ADDR,
  bridgeMintAddress: MINT_ADDR,
};

// ─── Mock factory ─────────────────────────────────────────────────────────────

/**
 * Build a BridgeClient with all ethers network calls stubbed out.
 * Returns the client and handles to the mock contracts so tests can
 * override specific methods.
 */
function makeClient(overrides: {
  lockIpcFee?: bigint;
  lockTxHash?: string;
  lockBlock?: number;
  lockTimestamp?: number;
  lockEventFound?: boolean;
  mintEventFound?: boolean;
  isProcessed?: boolean;
} = {}) {
  const {
    lockIpcFee     = 10_000_000_000_000_000n, // 0.01 FIL
    lockTxHash     = "0x" + "f".repeat(64),
    lockBlock      = 100,
    lockTimestamp  = 1_700_000_000,
    lockEventFound = true,
    mintEventFound = false,
    isProcessed    = false,
  } = overrides;

  const client = new BridgeClient(config);

  // ── Mock filecoin provider ──────────────────────────────────────────────
  const mockFilecoinProvider = {
    getBlock: vi.fn().mockResolvedValue({ timestamp: lockTimestamp }),
    getTransaction: vi.fn().mockResolvedValue({ blockNumber: lockBlock }),
  };
  (client as any).filecoinProvider = mockFilecoinProvider;

  // ── Mock ethereum provider ──────────────────────────────────────────────
  const mockEthereumProvider = {
    getBlock: vi.fn().mockResolvedValue({ timestamp: lockTimestamp + 60 }),
    getTransaction: vi.fn().mockResolvedValue({ blockNumber: lockBlock + 5 }),
  };
  (client as any).ethereumProvider = mockEthereumProvider;

  // ── Mock bridgeLock contract ────────────────────────────────────────────
  const lockEventLog = {
    args: [TOKEN_ADDR, "0x" + "9".repeat(40), RECIPIENT, 100n, TRANSFER_ID],
    blockNumber: lockBlock,
    transactionHash: lockTxHash,
    topics: [],
    data: "0x",
  };

  const mockBridgeLock = {
    ipcFee: vi.fn().mockResolvedValue(lockIpcFee),
    connect: vi.fn().mockReturnThis(),
    lock: vi.fn().mockResolvedValue({
      hash: lockTxHash,
      wait: vi.fn().mockResolvedValue({
        hash: lockTxHash,
        blockNumber: lockBlock,
        logs: lockEventFound ? [lockEventLog] : [],
      }),
    }),
    filters: {
      TokensLocked: vi.fn().mockReturnValue({}),
    },
    queryFilter: vi.fn().mockResolvedValue(lockEventFound ? [lockEventLog] : []),
    on: vi.fn(),
    off: vi.fn(),
    interface: {
      parseLog: vi.fn().mockImplementation((log: any) => {
        if (lockEventFound) {
          return {
            name: "TokensLocked",
            args: [TOKEN_ADDR, "0x" + "9".repeat(40), RECIPIENT, 100n, TRANSFER_ID],
          };
        }
        return null;
      }),
    },
  };
  (client as any).bridgeLock = mockBridgeLock;

  // ── Mock bridgeMint contract ────────────────────────────────────────────
  const mintEventLog = {
    args: ["0x" + "7".repeat(40), RECIPIENT, 100n, TRANSFER_ID],
    blockNumber: lockBlock + 5,
    transactionHash: "0x" + "1".repeat(64),
    topics: [],
    data: "0x",
  };

  const mockBridgeMint = {
    isProcessed: vi.fn().mockResolvedValue(isProcessed),
    filters: {
      TokensMinted: vi.fn().mockReturnValue({}),
    },
    queryFilter: vi.fn().mockResolvedValue(mintEventFound ? [mintEventLog] : []),
    on: vi.fn(),
    off: vi.fn(),
    interface: {
      parseLog: vi.fn().mockImplementation(() => {
        if (mintEventFound) {
          return {
            name: "TokensMinted",
            args: ["0x" + "7".repeat(40), RECIPIENT, 100n, TRANSFER_ID],
          };
        }
        return null;
      }),
    },
  };
  (client as any).bridgeMint = mockBridgeMint;

  return { client, mockBridgeLock, mockBridgeMint, mockFilecoinProvider };
}

// ─── BridgeClient constructor ─────────────────────────────────────────────────

describe("BridgeClient constructor", () => {
  it("stores config as frozen", () => {
    const client = new BridgeClient(config);
    expect(client.config).toMatchObject(config);
    expect(Object.isFrozen(client.config)).toBe(true);
  });

  it("creates read-only providers", () => {
    const client = new BridgeClient(config);
    expect(client.filecoinProvider).toBeInstanceOf(ethers.JsonRpcProvider);
    expect(client.ethereumProvider).toBeInstanceOf(ethers.JsonRpcProvider);
  });
});

// ─── lockTokens ───────────────────────────────────────────────────────────────

describe("lockTokens", () => {
  it("returns a TransferReceipt with correct fields", async () => {
    const { client, mockBridgeLock } = makeClient();
    const signer = { address: "0x" + "8".repeat(40) } as any;

    const receipt = await client.lockTokens(
      { tokenAddress: TOKEN_ADDR, amount: 100n, recipient: RECIPIENT },
      signer,
    );

    expect(receipt.transferId).toBe(TRANSFER_ID);
    expect(receipt.lockTxHash).toBe("0x" + "f".repeat(64));
    expect(receipt.amount).toBe(100n);
    expect(receipt.recipient).toBe(RECIPIENT);
    expect(receipt.tokenAddress).toBe(TOKEN_ADDR);
  });

  it("calls lock() with correct arguments", async () => {
    const { client, mockBridgeLock } = makeClient();
    const signer = {} as any;

    await client.lockTokens(
      { tokenAddress: TOKEN_ADDR, amount: 250n, recipient: RECIPIENT },
      signer,
    );

    expect(mockBridgeLock.lock).toHaveBeenCalledWith(
      TOKEN_ADDR,
      250n,
      RECIPIENT,
      expect.objectContaining({ value: 10_000_000_000_000_000n }),
    );
  });

  it("uses provided ipcFee instead of fetching it", async () => {
    const { client, mockBridgeLock } = makeClient();
    const signer = {} as any;

    await client.lockTokens(
      { tokenAddress: TOKEN_ADDR, amount: 100n, recipient: RECIPIENT, ipcFee: 42n },
      signer,
    );

    expect(mockBridgeLock.ipcFee).not.toHaveBeenCalled();
    expect(mockBridgeLock.lock).toHaveBeenCalledWith(
      TOKEN_ADDR, 100n, RECIPIENT, expect.objectContaining({ value: 42n }),
    );
  });

  it("throws on invalid tokenAddress", async () => {
    const { client } = makeClient();
    await expect(
      client.lockTokens({ tokenAddress: "not-an-address", amount: 100n, recipient: RECIPIENT }, {} as any),
    ).rejects.toThrow("Invalid tokenAddress");
  });

  it("throws on invalid recipient", async () => {
    const { client } = makeClient();
    await expect(
      client.lockTokens({ tokenAddress: TOKEN_ADDR, amount: 100n, recipient: "bad" }, {} as any),
    ).rejects.toThrow("Invalid recipient");
  });

  it("throws on zero amount", async () => {
    const { client } = makeClient();
    await expect(
      client.lockTokens({ tokenAddress: TOKEN_ADDR, amount: 0n, recipient: RECIPIENT }, {} as any),
    ).rejects.toThrow("amount must be > 0");
  });

  it("throws if TokensLocked event is missing from receipt", async () => {
    const { client } = makeClient({ lockEventFound: false });
    await expect(
      client.lockTokens({ tokenAddress: TOKEN_ADDR, amount: 100n, recipient: RECIPIENT }, {} as any),
    ).rejects.toThrow("TokensLocked event not found");
  });
});

// ─── getTransferStatus ─────────────────────────────────────────────────────────

describe("getTransferStatus", () => {
  it("returns state=minted when mint event found on Ethereum", async () => {
    const { client } = makeClient({ mintEventFound: true, lockEventFound: true });
    const status = await client.getTransferStatus(TRANSFER_ID);
    expect(status.state).toBe("minted");
    expect(status.mintTxHash).toBeDefined();
  });

  it("returns state=relaying when isProcessed=true but no mint event yet", async () => {
    const { client } = makeClient({ isProcessed: true, mintEventFound: false });
    const status = await client.getTransferStatus(TRANSFER_ID);
    expect(status.state).toBe("relaying");
  });

  it("returns state=locked when lock event found but not processed", async () => {
    const { client } = makeClient({ isProcessed: false, mintEventFound: false, lockEventFound: true });
    const status = await client.getTransferStatus(TRANSFER_ID);
    expect(status.state).toBe("locked");
    expect(status.lockTxHash).toBeDefined();
  });

  it("returns state=unknown when nothing found", async () => {
    const { client } = makeClient({ isProcessed: false, mintEventFound: false, lockEventFound: false });
    const status = await client.getTransferStatus(TRANSFER_ID);
    expect(status.state).toBe("unknown");
  });

  it("normalises transferId without 0x prefix", async () => {
    const { client } = makeClient({ mintEventFound: true });
    const status = await client.getTransferStatus(TRANSFER_ID.slice(2)); // strip 0x
    expect(status.transferId).toBe(TRANSFER_ID.toLowerCase());
  });

  it("throws on invalid transferId", async () => {
    const { client } = makeClient();
    await expect(client.getTransferStatus("0xshort")).rejects.toThrow("Invalid transferId");
  });
});

// ─── waitForCompletion ─────────────────────────────────────────────────────────

describe("waitForCompletion", () => {
  it("resolves immediately when already minted", async () => {
    const { client } = makeClient({ mintEventFound: true });
    const status = await client.waitForCompletion(TRANSFER_ID, { timeoutMs: 5000 });
    expect(status.state).toBe("minted");
  });

  it("polls until minted state is reached", async () => {
    const { client } = makeClient({ mintEventFound: false, lockEventFound: true });
    let callCount = 0;
    const getStatusSpy = vi
      .spyOn(client, "getTransferStatus")
      .mockImplementation(async () => {
        callCount++;
        const state: TransferStatus["state"] = callCount < 3 ? "locked" : "minted";
        return { transferId: TRANSFER_ID, state };
      });

    const status = await client.waitForCompletion(TRANSFER_ID, {
      timeoutMs: 10_000,
      pollIntervalMs: 10,
    });
    expect(status.state).toBe("minted");
    expect(callCount).toBeGreaterThanOrEqual(3);
    getStatusSpy.mockRestore();
  });

  it("throws on timeout", async () => {
    const { client } = makeClient({ lockEventFound: true, mintEventFound: false, isProcessed: false });
    vi.spyOn(client, "getTransferStatus").mockResolvedValue({
      transferId: TRANSFER_ID,
      state: "locked",
    });

    await expect(
      client.waitForCompletion(TRANSFER_ID, { timeoutMs: 50, pollIntervalMs: 10 }),
    ).rejects.toThrow("timed out");
  });

  it("calls onPoll callback on each iteration", async () => {
    const { client } = makeClient();
    let pollCount = 0;
    vi.spyOn(client, "getTransferStatus")
      .mockResolvedValueOnce({ transferId: TRANSFER_ID, state: "locked" })
      .mockResolvedValueOnce({ transferId: TRANSFER_ID, state: "minted" });

    await client.waitForCompletion(TRANSFER_ID, {
      timeoutMs: 5000,
      pollIntervalMs: 10,
      onPoll: () => { pollCount++; },
    });
    expect(pollCount).toBe(2);
  });
});

// ─── Event subscriptions ──────────────────────────────────────────────────────

describe("onTokensLocked", () => {
  it("registers and deregisters listener", () => {
    const { client, mockBridgeLock } = makeClient();
    const handler = vi.fn();
    const cleanup = client.onTokensLocked(handler);
    expect(mockBridgeLock.on).toHaveBeenCalledWith("TokensLocked", expect.any(Function));
    cleanup();
    expect(mockBridgeLock.off).toHaveBeenCalled();
  });
});

describe("onTokensMinted", () => {
  it("registers and deregisters listener", () => {
    const { client, mockBridgeMint } = makeClient();
    const handler = vi.fn();
    const cleanup = client.onTokensMinted(handler);
    expect(mockBridgeMint.on).toHaveBeenCalledWith("TokensMinted", expect.any(Function));
    cleanup();
    expect(mockBridgeMint.off).toHaveBeenCalled();
  });
});

// ─── TransferId normalisation ─────────────────────────────────────────────────

describe("_normalizeTransferId (via getTransferStatus)", () => {
  it("accepts lowercase hex", async () => {
    const { client } = makeClient({ mintEventFound: true });
    const status = await client.getTransferStatus(TRANSFER_ID.toLowerCase());
    expect(status.transferId).toBe(TRANSFER_ID.toLowerCase());
  });

  it("accepts uppercase hex", async () => {
    const { client } = makeClient({ mintEventFound: true });
    const status = await client.getTransferStatus(TRANSFER_ID.toUpperCase());
    expect(status.transferId).toBe(TRANSFER_ID.toLowerCase());
  });

  it("rejects too-short id", async () => {
    const { client } = makeClient();
    await expect(client.getTransferStatus("0x1234")).rejects.toThrow("Invalid transferId");
  });

  it("rejects non-hex chars", async () => {
    const { client } = makeClient();
    await expect(client.getTransferStatus("0x" + "z".repeat(64))).rejects.toThrow("Invalid transferId");
  });
});
