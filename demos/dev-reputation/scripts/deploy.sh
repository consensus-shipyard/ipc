#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ACTOR_DIR="${ROOT_DIR}/actor"
OUT_ENV="${ROOT_DIR}/.env.actor"
FRONTEND_CONFIG="${ROOT_DIR}/frontend/config.js"

if [[ -z "${ADMIN_ADDRESS:-}" ]]; then
  echo "ADMIN_ADDRESS is required"
  exit 1
fi

if [[ -z "${AGENT_ADDRESS:-}" ]]; then
  echo "AGENT_ADDRESS is required"
  exit 1
fi

echo "Building actor WASM..."
(
  cd "${ACTOR_DIR}"
  rustup target add wasm32-unknown-unknown >/dev/null 2>&1 || true
  cargo build --target wasm32-unknown-unknown --release
)

WASM_PATH="${ACTOR_DIR}/target/wasm32-unknown-unknown/release/ipc_reputation_actor.wasm"
if [[ ! -f "${WASM_PATH}" ]]; then
  echo "WASM not found at ${WASM_PATH}"
  exit 1
fi

echo "Deploying actor via ipc-cli..."
if ! command -v ipc-cli >/dev/null 2>&1; then
  echo "ipc-cli not found in PATH"
  exit 1
fi

# These commands are intentionally explicit so operators can swap method names per network setup.
ACTOR_ADDRESS="$(ipc-cli actor create --wasm "${WASM_PATH}" | awk '/f0/{print $1}' | tail -n 1)"
if [[ -z "${ACTOR_ADDRESS}" ]]; then
  echo "Failed to parse actor address from deployment output"
  exit 1
fi

ipc-cli actor invoke \
  --to "${ACTOR_ADDRESS}" \
  --method 1 \
  --params "{\"admin\":\"${ADMIN_ADDRESS}\",\"initial_agent\":\"${AGENT_ADDRESS}\"}" >/dev/null

cat >"${OUT_ENV}" <<EOF
REPUTATION_ACTOR_ADDRESS=${ACTOR_ADDRESS}
EOF

cat >"${FRONTEND_CONFIG}" <<EOF
window.IPC_CONFIG = {
  actorAddress: "${ACTOR_ADDRESS}",
  rpcUrl: "https://api.calibration.node.glif.io/rpc/v1",
  basinUrl: "https://basin.tableland.xyz",
  agentUrl: "http://localhost:3001",
  chainId: 314159
};
EOF

echo "Actor deployed: ${ACTOR_ADDRESS}"
echo "Explorer: https://calibration.filscan.io/address/${ACTOR_ADDRESS}"
