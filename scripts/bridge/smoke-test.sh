#!/usr/bin/env bash
# smoke-test.sh
# End-to-end smoke test for the IPC cross-chain token bridge.
#
# Performs one complete round-trip:
#   1. Approves BridgeLock to spend test tokens
#   2. Calls BridgeLock.lock() on Filecoin Calibration
#   3. Polls BridgeMint on Ethereum Sepolia until the wrapped tokens appear
#   4. Asserts the minted amount and recipient are correct
#   5. Prints a pass/fail summary
#
# Usage:
#   bash scripts/bridge/smoke-test.sh [--amount <uint256>] [--recipient <address>]
#
# Prerequisites:
#   - .env populated (or deployments.json exists with BRIDGE_LOCK_ADDRESS etc.)
#   - cast (Foundry) installed
#   - The bridge contracts are deployed and the relay actor is running
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
CONTRACTS_DIR="${REPO_ROOT}/contracts"
DEPLOYMENTS_JSON="${CONTRACTS_DIR}/deployments/deployments.json"

export PATH="${HOME}/.foundry/bin:${HOME}/.nvm/versions/node/v22.22.1/bin:${PATH}"

# ── Parse args ─────────────────────────────────────────────────────────────────
SMOKE_AMOUNT="${SMOKE_AMOUNT:-1000000000000000000}"   # 1 token (18 decimals)
SMOKE_RECIPIENT="${SMOKE_RECIPIENT:-}"
TIMEOUT_SECONDS="${SMOKE_TIMEOUT:-300}"               # 5 min default
POLL_INTERVAL="${SMOKE_POLL_INTERVAL:-10}"            # 10s between polls

while [[ $# -gt 0 ]]; do
  case "$1" in
    --amount)    SMOKE_AMOUNT="$2";    shift 2 ;;
    --recipient) SMOKE_RECIPIENT="$2"; shift 2 ;;
    --timeout)   TIMEOUT_SECONDS="$2"; shift 2 ;;
    *) echo "Unknown arg: $1" >&2; exit 1 ;;
  esac
done

# ── Load config ────────────────────────────────────────────────────────────────
ENV_FILE="${REPO_ROOT}/.env"
if [[ -f "${ENV_FILE}" ]]; then
  # shellcheck disable=SC1090
  source "${ENV_FILE}"
fi

# Prefer deployments.json over .env for contract addresses
if [[ -f "${DEPLOYMENTS_JSON}" ]]; then
  BRIDGE_LOCK_ADDRESS=$(jq -r '.bridgeLock' "${DEPLOYMENTS_JSON}")
  BRIDGE_MINT_ADDRESS=$(jq -r '.bridgeMint' "${DEPLOYMENTS_JSON}")
  FILECOIN_TOKEN=$(jq -r '.filecoinToken' "${DEPLOYMENTS_JSON}")
  FILECOIN_RPC_URL=$(jq -r '.filecoinRpc' "${DEPLOYMENTS_JSON}")
  ETHEREUM_RPC_URL=$(jq -r '.ethereumRpc' "${DEPLOYMENTS_JSON}")
fi

# Validate
for var in PRIVATE_KEY BRIDGE_LOCK_ADDRESS BRIDGE_MINT_ADDRESS FILECOIN_TOKEN FILECOIN_RPC_URL ETHEREUM_RPC_URL; do
  if [[ -z "${!var:-}" || "${!var}" == "null" ]]; then
    echo "ERROR: \$${var} is not set. Run deploy-all.sh first." >&2
    exit 1
  fi
done

DEPLOYER=$(cast wallet address "${PRIVATE_KEY}")
SMOKE_RECIPIENT="${SMOKE_RECIPIENT:-${DEPLOYER}}"
IPC_FEE="${IPC_FEE:-10000000000000000}"

echo ""
echo "════════════════════════════════════════════════════════════"
echo "  IPC Bridge Smoke Test"
echo "════════════════════════════════════════════════════════════"
echo "  BridgeLock:  ${BRIDGE_LOCK_ADDRESS} (Filecoin Calibration)"
echo "  BridgeMint:  ${BRIDGE_MINT_ADDRESS} (Ethereum Sepolia)"
echo "  Token:       ${FILECOIN_TOKEN}"
echo "  Amount:      ${SMOKE_AMOUNT}"
echo "  Recipient:   ${SMOKE_RECIPIENT}"
echo "  Timeout:     ${TIMEOUT_SECONDS}s"
echo ""

PASS=0
FAIL=0

check() {
  local label="$1"
  local result="$2"
  local expected="$3"
  if [[ "${result}" == "${expected}" ]]; then
    echo "  ✓ ${label}"
    PASS=$((PASS + 1))
  else
    echo "  ✗ ${label}: expected '${expected}', got '${result}'" >&2
    FAIL=$((FAIL + 1))
  fi
}

# ── Step 1: Check deployer token balance ───────────────────────────────────────
echo "▶ Step 1: Checking token balance..."
BALANCE_BEFORE=$(cast call "${FILECOIN_TOKEN}" \
  "balanceOf(address)(uint256)" "${DEPLOYER}" \
  --rpc-url "${FILECOIN_RPC_URL}")
echo "  Deployer token balance: ${BALANCE_BEFORE}"
if [[ "${BALANCE_BEFORE}" -lt "${SMOKE_AMOUNT}" ]]; then
  echo "ERROR: Insufficient token balance. Need ${SMOKE_AMOUNT}, have ${BALANCE_BEFORE}." >&2
  echo "       Mint test tokens first: cast send ${FILECOIN_TOKEN} 'mint(address,uint256)' ${DEPLOYER} ${SMOKE_AMOUNT} --rpc-url ${FILECOIN_RPC_URL} --private-key \$PRIVATE_KEY"
  exit 1
fi
echo "  ✓ Sufficient balance"

# ── Step 2: Approve BridgeLock ─────────────────────────────────────────────────
echo ""
echo "▶ Step 2: Approving BridgeLock to spend tokens..."
cast send "${FILECOIN_TOKEN}" \
  "approve(address,uint256)" "${BRIDGE_LOCK_ADDRESS}" "${SMOKE_AMOUNT}" \
  --rpc-url "${FILECOIN_RPC_URL}" \
  --private-key "${PRIVATE_KEY}" \
  --quiet
echo "  ✓ Approved ${SMOKE_AMOUNT} tokens"

# ── Step 3: Lock tokens ────────────────────────────────────────────────────────
echo ""
echo "▶ Step 3: Locking tokens (initiating bridge transfer)..."
LOCK_TX=$(cast send "${BRIDGE_LOCK_ADDRESS}" \
  "lock(address,uint256,address)" \
  "${FILECOIN_TOKEN}" "${SMOKE_AMOUNT}" "${SMOKE_RECIPIENT}" \
  --value "${IPC_FEE}" \
  --rpc-url "${FILECOIN_RPC_URL}" \
  --private-key "${PRIVATE_KEY}" \
  --json)

LOCK_TX_HASH=$(echo "${LOCK_TX}" | jq -r '.transactionHash')
LOCK_BLOCK=$(echo "${LOCK_TX}" | jq -r '.blockNumber')
echo "  ✓ Lock tx:   ${LOCK_TX_HASH}"
echo "  ✓ Lock block: ${LOCK_BLOCK}"

# Extract transferId from the TokensLocked event log
TRANSFER_ID=$(cast receipt "${LOCK_TX_HASH}" \
  --rpc-url "${FILECOIN_RPC_URL}" \
  --json | \
  jq -r '.logs[] | select(.topics[0] == "0x'"$(cast keccak "TokensLocked(address,address,address,uint256,bytes32)" | cut -c3-)"'") | .data' | \
  # transferId is the second 32-byte word in non-indexed data (after amount)
  cut -c67-130 | xargs printf "0x%s" || echo "")

# Fallback: use cast logs
if [[ -z "${TRANSFER_ID}" || "${TRANSFER_ID}" == "0x" ]]; then
  echo "  (Parsing transferId from logs...)"
  TRANSFER_ID=$(cast logs \
    --from-block "${LOCK_BLOCK}" \
    --to-block "${LOCK_BLOCK}" \
    --address "${BRIDGE_LOCK_ADDRESS}" \
    --rpc-url "${FILECOIN_RPC_URL}" \
    --json 2>/dev/null | \
    jq -r '.[0].data' | \
    awk '{print "0x" substr($0, 67, 64)}' || echo "unknown")
fi
echo "  ✓ Transfer ID: ${TRANSFER_ID}"

# ── Step 4: Poll for mint on Ethereum ──────────────────────────────────────────
echo ""
echo "▶ Step 4: Polling Ethereum Sepolia for minted tokens (timeout: ${TIMEOUT_SECONDS}s)..."
DEADLINE=$(($(date +%s) + TIMEOUT_SECONDS))
MINTED=false
MINT_TX_HASH=""

while [[ $(date +%s) -lt ${DEADLINE} ]]; do
  # Check WrappedToken balance of recipient
  WRAPPED_TOKEN=$(cast call "${BRIDGE_MINT_ADDRESS}" \
    "wrappedTokens(address)(address)" "${FILECOIN_TOKEN}" \
    --rpc-url "${ETHEREUM_RPC_URL}" 2>/dev/null || echo "0x0000000000000000000000000000000000000000")

  if [[ "${WRAPPED_TOKEN}" != "0x0000000000000000000000000000000000000000" && \
        "${WRAPPED_TOKEN}" != "" ]]; then
    MINTED_BALANCE=$(cast call "${WRAPPED_TOKEN}" \
      "balanceOf(address)(uint256)" "${SMOKE_RECIPIENT}" \
      --rpc-url "${ETHEREUM_RPC_URL}" 2>/dev/null || echo "0")

    if [[ "${MINTED_BALANCE}" -ge "${SMOKE_AMOUNT}" ]]; then
      MINTED=true
      echo "  ✓ Wrapped token balance: ${MINTED_BALANCE}"
      break
    fi
  fi

  REMAINING=$((DEADLINE - $(date +%s)))
  echo "  ⋯ Waiting... (${REMAINING}s remaining, wrapped balance: ${MINTED_BALANCE:-0})"
  sleep "${POLL_INTERVAL}"
done

# ── Step 5: Assert results ─────────────────────────────────────────────────────
echo ""
echo "▶ Step 5: Asserting results..."

if [[ "${MINTED}" == "true" ]]; then
  check "Transfer completed (minted)" "true" "true"
  check "Minted amount correct" "${MINTED_BALANCE}" "${SMOKE_AMOUNT}"

  # Verify replay protection: check transferId is marked on BridgeMint
  if [[ "${TRANSFER_ID}" != "unknown" ]]; then
    IS_PROCESSED=$(cast call "${BRIDGE_MINT_ADDRESS}" \
      "isProcessed(bytes32)(bool)" "${TRANSFER_ID}" \
      --rpc-url "${ETHEREUM_RPC_URL}" 2>/dev/null || echo "false")
    check "TransferId marked as processed (replay protection)" "${IS_PROCESSED}" "true"
  fi
else
  echo "  ✗ Transfer did NOT complete within ${TIMEOUT_SECONDS}s" >&2
  FAIL=$((FAIL + 1))
fi

# ── Summary ────────────────────────────────────────────────────────────────────
echo ""
echo "════════════════════════════════════════════════════════════"
if [[ ${FAIL} -eq 0 ]]; then
  echo "  ✅ SMOKE TEST PASSED  (${PASS} checks passed, 0 failed)"
else
  echo "  ❌ SMOKE TEST FAILED  (${PASS} passed, ${FAIL} failed)"
fi
echo "════════════════════════════════════════════════════════════"
echo ""

[[ ${FAIL} -eq 0 ]]
