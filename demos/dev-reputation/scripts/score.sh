#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 2 ]]; then
  echo "Usage: ./scripts/score.sh {github_handle} {wallet_address}"
  exit 1
fi

GITHUB_HANDLE="$1"
WALLET_ADDRESS="$2"
AGENT_URL="${AGENT_URL:-http://localhost:3001}"

curl -sS -X POST "${AGENT_URL}/score" \
  -H "Content-Type: application/json" \
  -d "{\"github_handle\":\"${GITHUB_HANDLE}\",\"wallet_address\":\"${WALLET_ADDRESS}\"}"
echo
