# IPC Cross-Chain Token Bridge — Security Considerations

**Date:** 2026-03-19
**Author:** Bridge Bot (QA workstream)

---

## Trust Model

The bridge operates across two blockchains and an IPC subnet. The trust chain is:

```
[Filecoin user]
    → trusts BridgeLock.sol (audited, UUPS proxy, admin-controlled)
        → trusts IPC Gateway (IPC infrastructure)
            → trusts bridge-relay WASM actor (deployed by IPC subnet operators)
                → trusts BridgeMint.sol (audited, UUPS proxy, admin-controlled)
                    → trusts WrappedToken.sol (deployed by BridgeMint admin)
```

**Critical trust boundary:** The IPC subnet gateway is the root of trust for cross-chain message delivery. If the gateway is compromised, an attacker could forge `from` addresses in IPC envelopes. BridgeMint cannot independently verify that the gateway is honest — this is an inherent limitation of the IPC cross-chain messaging model.

---

## Attack Surface

### 1. Unauthorized mint (HIGH priority)

**Threat:** Attacker mints wrapped tokens on Ethereum without locking real tokens on Filecoin.

**Mitigations:**
- `onlyGateway` modifier — only the deployed IPC gateway address can deliver messages
- `_validateOrigin` — both the source subnet ID and the BridgeLock FvmAddress must match the registered values
- Two checks must fail simultaneously: gateway impersonation AND origin spoofing

**Residual risk:** Gateway contract compromise. Mitigated by using the IPC-team-deployed gateway contract (not user-deployed). A gateway upgrade would require the IPC team's multisig.

### 2. Replay attack / double-spend (HIGH priority)

**Threat:** Attacker replays a `TokensLocked` event to mint tokens twice for a single lock.

**Mitigations (three independent layers):**
1. **BridgeLock nonce** — each `lock()` increments `_nonce`, making transferIds unique by construction
2. **bridge-relay HAMT** — actor rejects duplicate transferIds before emitting the relay event
3. **BridgeMint processedTransfers** — `DuplicateTransfer` reverts if the same transferId is seen

**Residual risk:** None identified. All three layers must be bypassed simultaneously.

### 3. Reentrancy (MEDIUM priority)

**Threat:** Malicious ERC20 token's `transfer` callback re-enters `BridgeLock.lock()`.

**Mitigations:**
- State changes (`_nonce++`, `processedTransfers[transferId] = true`) are committed **before** `safeTransferFrom` (CEI pattern)
- `IpcExchange.performIpcCall` is `nonReentrant` — prevents re-entry at the IPC dispatch level
- Slither rates these reentrancy warnings as `benign` — confirmed by manual inspection

**Residual risk:** Low. A malicious ERC20 re-entering `lock()` would encounter an already-set `processedTransfers` entry and revert cleanly (no funds at risk; lock would fail).

### 4. Admin key compromise (HIGH priority)

**Threat:** Attacker obtains `DEFAULT_ADMIN_ROLE` key and either upgrades contracts to drain funds, or rescues tokens via `rescueTokens`.

**Mitigations (current, testnet):**
- Single deployer EOA holds admin role — acceptable for testnet only

**Required for mainnet:**
- Replace EOA with a Gnosis Safe multisig (≥3/5 threshold)
- Add a 48-hour timelock to UUPS upgrade calls
- `PAUSER_ROLE` can be a separate, faster-response key (1/1 OK for pause emergencies)

### 5. Wrapped token minting authority (MEDIUM priority)

**Threat:** Attacker obtains `MINTER_ROLE` on a WrappedToken and mints arbitrary supply.

**Mitigations:**
- `MINTER_ROLE` on WrappedToken is granted only to the BridgeMint proxy address during `deployAndRegisterAsset`
- BridgeMint itself enforces replay protection before calling `WrappedToken.mint`
- `DEFAULT_ADMIN_ROLE` on WrappedToken is held by the same BridgeMint admin

**Residual risk:** If BridgeMint admin is compromised, they could grant `MINTER_ROLE` to an attacker. Same mitigation as #4 (multisig).

### 6. IPC subnet operator collusion (MEDIUM priority)

**Threat:** IPC subnet validators collude to forge bridge-relay events, causing BridgeMint to mint without corresponding locks.

**Mitigations:**
- BridgeMint validates the message origin against a specific BridgeLock address and subnet ID
- A forged message must bypass both the gateway and the origin check
- This attack requires validator collusion at the IPC subnet level (Byzantine fault)

**Residual risk:** Inherent in the IPC cross-chain model. Mitigation: use a subnet with sufficient decentralization and stake. Document this as a known trust assumption.

### 7. Reorg-based double-spend (LOW-MEDIUM priority)

**Threat:** Lock tx is included, relay fires, then Filecoin reorgs the lock tx out. Attacker re-spends the tokens while wrapped tokens remain minted on Ethereum.

**Mitigations:**
- Relay actor should enforce a confirmation threshold (≥12 Filecoin blocks ≈ ~5 min finality)
- BridgeMint's `processedTransfers` record remains even after a reorg — a re-submitted lock with the same parameters would produce the same transferId (same nonce, assuming the nonce state also reorged), which would be rejected
- If the nonce state reorgs back, a new lock would produce a new transferId and proceed normally

**Residual risk:** Low if confirmation threshold is enforced. **Action required:** add `min_confirmations` parameter to bridge-relay actor before mainnet.

### 8. Token allowlist bypass (LOW priority)

**Threat:** User locks an unexpected/malicious token on Filecoin; wrapped token minted on Ethereum for an asset with no real value.

**Mitigations:**
- `tokenAllowlistEnabled` flag on BridgeLock — currently disabled for testnet flexibility
- When enabled, only whitelisted tokens can be locked

**Recommendation:** Enable allowlist for production deployment with a governance-controlled list.

### 9. UUPS proxy upgrade attack (LOW priority)

**Threat:** An upgrade to a malicious implementation contract drains all locked tokens.

**Mitigations:**
- `_authorizeUpgrade` is gated to `DEFAULT_ADMIN_ROLE` on both contracts
- UUPS proxies are upgradeable only by the admin — not by any user

**Recommendation:** Add a timelock to upgrade calls before mainnet (see #4).

### 10. Front-running (INFORMATIONAL)

**Threat:** Miner/validator front-runs a `lock()` call to steal the user's tokens.

**Analysis:** The `recipient` address is included in the `lock()` call parameters. An attacker cannot modify the recipient without controlling the sender's key. Front-running in this model does not steal funds — at worst it delays the user's transaction. Not a meaningful attack vector.

---

## Known Limitations

1. **No confirmation threshold in bridge-relay actor** — the actor does not yet enforce a minimum block confirmation count before processing events. This must be added before mainnet to protect against reorg-based attacks.

2. **Single admin key (testnet only)** — both contracts use a single EOA as admin. Must be upgraded to multisig + timelock before mainnet.

3. **No automatic retry for failed relays** — if the IPC cross-message delivery fails (e.g., BridgeMint is paused when the message arrives), the tokens remain locked on Filecoin with no automatic recourse. The admin can unpause and the message may be retried by the IPC layer, or tokens can be rescued manually.

4. **Gateway trust assumption** — the security of the bridge is contingent on the security of the IPC gateway contracts and the subnet validator set. An independent audit of the IPC gateway is strongly recommended before mainnet.

5. **Gas price volatility** — the `ipcFee` is set at deployment time. If gas prices spike on the destination chain, the fee may be insufficient for message delivery. An admin can update the fee via `setIpcFee()`.

---

## Mainnet Checklist

Before deploying to mainnet, the following must be addressed:

- [ ] Full third-party smart contract audit of BridgeLock, BridgeMint, WrappedToken
- [ ] Replace deployer EOA with multisig (≥3/5) for DEFAULT_ADMIN_ROLE on both contracts
- [ ] Add 48-hour timelock to UUPS upgrade path
- [ ] Add `min_confirmations` parameter to bridge-relay actor
- [ ] Enable token allowlist on BridgeLock with governance-controlled whitelist
- [ ] Monitor `TokensLocked` + `TokensMinted` events for anomaly detection
- [ ] Incident response plan: who holds PAUSER_ROLE, how fast can they respond?
- [ ] IPC subnet validator set review: sufficient decentralization and stake
- [ ] Document and test the rescue procedure for edge cases (paused bridge, failed relay)
