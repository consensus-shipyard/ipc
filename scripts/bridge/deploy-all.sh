#!/usr/bin/env bash
# deploy-all.sh
# Single-command deployment of the IPC cross-chain token bridge.
#
# Deploys:
#   1. BridgeLock.sol proxy on Filecoin Calibration
#   2. (optional) TestToken ERC20 on Filecoin Calibration
#   3. WrappedToken impl + BridgeMint.sol proxy on Ethereum Sepolia
#   4. Wires BridgeLock → BridgeMint destination, registers asset mapping
#   5. Outputs deployments.json with all addresses
#
# Usage:
#   cp .env.example .env && vim .env
#   bash scripts/bridge/deploy-all.sh
#
# Prerequisites: .env populated, pnpm deps installed (pnpm install in contracts/)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
CONTRACTS_DIR="${REPO_ROOT}/contracts"
DEPLOYMENTS_DIR="${CONTRACTS_DIR}/deployments"

# ── Load .env ──────────────────────────────────────────────────────────────────
ENV_FILE="${REPO_ROOT}/.env"
if [[ ! -f "${ENV_FILE}" ]]; then
  echo "ERROR: .env not found. Copy .env.example to .env and fill in values." >&2
  exit 1
fi
# shellcheck disable=SC1090
source "${ENV_FILE}"

# ── Validate required vars ─────────────────────────────────────────────────────
required_vars=(
  PRIVATE_KEY
  FILECOIN_RPC_URL
  FILECOIN_IPC_GATEWAY
  ETHEREUM_RPC_URL
  ETHEREUM_IPC_GATEWAY
  IPC_SUBNET_ROOT
)
for var in "${required_vars[@]}"; do
  if [[ -z "${!var:-}" ]]; then
    echo "ERROR: Required environment variable \$${var} is not set." >&2
    exit 1
  fi
done

IPC_FEE="${IPC_FEE:-10000000000000000}"
mkdir -p "${DEPLOYMENTS_DIR}"

export PATH="${HOME}/.nvm/versions/node/v22.22.1/bin:${HOME}/.local/share/pnpm:${PATH}"

echo ""
echo "════════════════════════════════════════════════════════════"
echo "  IPC Cross-Chain Bridge — Full Deployment"
echo "════════════════════════════════════════════════════════════"
echo "  Filecoin RPC:   ${FILECOIN_RPC_URL}"
echo "  Ethereum RPC:   ${ETHEREUM_RPC_URL}"
echo "  Deployer:       $(cast wallet address "${PRIVATE_KEY}" 2>/dev/null || echo '(cast not available)')"
echo ""

cd "${CONTRACTS_DIR}"

# ── Step 1: Compile contracts ──────────────────────────────────────────────────
echo "▶ Step 1/5: Compiling contracts..."
export PATH="${HOME}/.foundry/bin:${PATH}"
forge build --skip test --quiet
echo "  ✓ Compiled"

# ── Step 2: Deploy TestToken on Filecoin (if no token provided) ───────────────
FILECOIN_TOKEN="${FILECOIN_TOKEN_ADDRESS:-}"
if [[ -z "${FILECOIN_TOKEN}" ]]; then
  echo ""
  echo "▶ Step 2/5: Deploying TestToken ERC20 on Filecoin Calibration..."
  TOKEN_OUTPUT=$(pnpm exec hardhat deploy-test-token \
    --network calibration 2>&1)
  echo "${TOKEN_OUTPUT}"
  FILECOIN_TOKEN=$(echo "${TOKEN_OUTPUT}" | grep '"testToken":' | sed 's/.*"testToken": "\(.*\)".*/\1/')
  if [[ -z "${FILECOIN_TOKEN}" ]]; then
    # Fallback: parse from deployments file
    FILECOIN_TOKEN=$(jq -r '.testToken // empty' "${DEPLOYMENTS_DIR}/test-token-calibration.json" 2>/dev/null || echo "")
  fi
  echo "  ✓ TestToken: ${FILECOIN_TOKEN}"
else
  echo ""
  echo "▶ Step 2/5: Using existing token: ${FILECOIN_TOKEN}"
fi

# ── Step 3: Deploy BridgeLock on Filecoin Calibration ─────────────────────────
echo ""
echo "▶ Step 3/5: Deploying BridgeLock on Filecoin Calibration..."
pnpm exec hardhat deploy-bridge-lock \
  --network calibration \
  --gateway "${FILECOIN_IPC_GATEWAY}" \
  --dest-root "${IPC_SUBNET_ROOT}" \
  --dest-receiver "0x0000000000000000000000000000000000000001" \
  --ipc-fee "${IPC_FEE}"

BRIDGE_LOCK_ADDRESS=$(jq -r '.proxy' "${DEPLOYMENTS_DIR}/bridge-lock-calibration.json")
echo "  ✓ BridgeLock proxy: ${BRIDGE_LOCK_ADDRESS}"

# ── Step 4: Deploy BridgeMint on Ethereum Sepolia ─────────────────────────────
echo ""
echo "▶ Step 4/5: Deploying BridgeMint + WrappedToken on Ethereum Sepolia..."
DEPLOY_MINT_ARGS=(
  --network sepolia
  --gateway "${ETHEREUM_IPC_GATEWAY}"
  --src-root "${IPC_SUBNET_ROOT}"
  --bridge-lock "${BRIDGE_LOCK_ADDRESS}"
)
# Register initial asset if a Filecoin token is set
if [[ -n "${FILECOIN_TOKEN}" ]]; then
  DEPLOY_MINT_ARGS+=(
    --filecoin-token "${FILECOIN_TOKEN}"
    --token-name "${WRAPPED_TOKEN_NAME:-Wrapped Token (IPC Bridge)}"
    --token-symbol "${WRAPPED_TOKEN_SYMBOL:-wTKN.ipc}"
  )
fi
pnpm exec hardhat deploy-bridge-mint "${DEPLOY_MINT_ARGS[@]}"

BRIDGE_MINT_ADDRESS=$(jq -r '.bridgeMintProxy' "${DEPLOYMENTS_DIR}/bridge-mint-sepolia.json")
WRAPPED_TOKEN_ADDRESS=$(jq -r '.initialWrappedToken // ""' "${DEPLOYMENTS_DIR}/bridge-mint-sepolia.json")
echo "  ✓ BridgeMint proxy:  ${BRIDGE_MINT_ADDRESS}"
echo "  ✓ WrappedToken:      ${WRAPPED_TOKEN_ADDRESS}"

# ── Step 5: Wire BridgeLock → BridgeMint (set destination) ───────────────────
echo ""
echo "▶ Step 5/5: Wiring BridgeLock → BridgeMint destination..."
pnpm exec hardhat set-bridge-destination \
  --network calibration \
  --bridge-lock "${BRIDGE_LOCK_ADDRESS}" \
  --dest-root "${IPC_SUBNET_ROOT}" \
  --dest-receiver "${BRIDGE_MINT_ADDRESS}"
echo "  ✓ BridgeLock destination set to ${BRIDGE_MINT_ADDRESS}"

# ── Write combined deployments.json ───────────────────────────────────────────
DEPLOYMENTS_JSON="${DEPLOYMENTS_DIR}/deployments.json"
jq -n \
  --arg bridgeLock "${BRIDGE_LOCK_ADDRESS}" \
  --arg bridgeMint "${BRIDGE_MINT_ADDRESS}" \
  --arg wrappedToken "${WRAPPED_TOKEN_ADDRESS}" \
  --arg filecoinToken "${FILECOIN_TOKEN}" \
  --arg filecoinRpc "${FILECOIN_RPC_URL}" \
  --arg ethereumRpc "${ETHEREUM_RPC_URL}" \
  --arg subnetRoot "${IPC_SUBNET_ROOT}" \
  --arg deployedAt "$(date -u +"%Y-%m-%dT%H:%M:%SZ")" \
  '{
    bridgeLock: $bridgeLock,
    bridgeMint: $bridgeMint,
    wrappedToken: $wrappedToken,
    filecoinToken: $filecoinToken,
    filecoinRpc: $filecoinRpc,
    ethereumRpc: $ethereumRpc,
    subnetRoot: $subnetRoot,
    deployedAt: $deployedAt
  }' > "${DEPLOYMENTS_JSON}"

echo ""
echo "════════════════════════════════════════════════════════════"
echo "  ✅ Deployment complete!"
echo "════════════════════════════════════════════════════════════"
echo ""
cat "${DEPLOYMENTS_JSON}"
echo ""
echo "  Deployment record saved to: ${DEPLOYMENTS_JSON}"
echo ""
echo "  Next: run 'make smoke-test' to verify the bridge end-to-end."
