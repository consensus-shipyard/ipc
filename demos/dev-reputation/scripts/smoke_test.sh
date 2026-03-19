#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
AGENT_DIR="${ROOT_DIR}/agent"
AGENT_URL="${AGENT_URL:-http://localhost:3001}"
HANDLE="${1:-torvalds}"
WALLET="${2:-0x000000000000000000000000000000000000dEaD}"
TIMEOUT_SECONDS=300

cleanup() {
  if [[ -n "${AGENT_PID:-}" ]] && kill -0 "${AGENT_PID}" >/dev/null 2>&1; then
    kill "${AGENT_PID}" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

echo "Starting agent..."
(
  cd "${AGENT_DIR}"
  npm install >/dev/null
  node src/index.js >/tmp/dev-reputation-agent.log 2>&1 &
  echo $! > /tmp/dev-reputation-agent.pid
)
AGENT_PID="$(cat /tmp/dev-reputation-agent.pid)"
sleep 2

echo "Submitting score job for ${HANDLE}..."
JOB_ID="$(curl -sS -X POST "${AGENT_URL}/score" -H "Content-Type: application/json" -d "{\"github_handle\":\"${HANDLE}\",\"wallet_address\":\"${WALLET}\"}" | python3 -c 'import json,sys;print(json.load(sys.stdin)["job_id"])')"
echo "job_id=${JOB_ID}"

START_TS="$(date +%s)"
while true; do
  JOB_JSON="$(curl -sS "${AGENT_URL}/job/${JOB_ID}")"
  STATUS="$(python3 -c 'import json,sys;print(json.load(sys.stdin).get("status",""))' <<<"${JOB_JSON}")"
  STEP="$(python3 -c 'import json,sys;print(json.load(sys.stdin).get("progress",{}).get("step",""))' <<<"${JOB_JSON}")"
  PCT="$(python3 -c 'import json,sys;print(json.load(sys.stdin).get("progress",{}).get("percentage",0))' <<<"${JOB_JSON}")"
  echo "status=${STATUS} step=${STEP} pct=${PCT}"
  if [[ "${STATUS}" == "complete" ]]; then
    break
  fi
  if [[ "${STATUS}" == "error" ]]; then
    echo "FAIL: scoring job failed"
    exit 1
  fi
  NOW="$(date +%s)"
  if (( NOW - START_TS > TIMEOUT_SECONDS )); then
    echo "FAIL: timeout waiting for scoring job"
    exit 1
  fi
  sleep 5
done

SCORE_JSON="$(curl -sS "${AGENT_URL}/score/${HANDLE}")"
SCORE="$(python3 -c 'import json,sys;print(json.load(sys.stdin)["score"])' <<<"${SCORE_JSON}")"
TIER="$(python3 -c 'import json,sys;print(json.load(sys.stdin)["tier"])' <<<"${SCORE_JSON}")"
CID="$(python3 -c 'import json,sys;print(json.load(sys.stdin)["evidence_cid"])' <<<"${SCORE_JSON}")"
HASH_EXPECTED="$(python3 -c 'import json,sys;print(json.load(sys.stdin)["document_hash"])' <<<"${SCORE_JSON}")"

if (( SCORE < 0 || SCORE > 100 )); then
  echo "FAIL: score out of range: ${SCORE}"
  exit 1
fi

if [[ ! "${TIER}" =~ ^(principal|senior|mid|junior|early-career)$ ]]; then
  echo "FAIL: invalid tier: ${TIER}"
  exit 1
fi

echo "Verifying Basin content hash..."
BASIN_API_URL="${BASIN_API_URL:-https://basin.tableland.xyz}"
BASIN_BUCKET="${BASIN_BUCKET:-default}"
DOC="$(curl -sS "${BASIN_API_URL}/api/v1/buckets/${BASIN_BUCKET}/objects/${CID}")"
HASH_COMPUTED="$(python3 -c 'import sys;from hashlib import sha3_256;data=sys.stdin.read().encode();print("0x"+sha3_256(data).hexdigest())' <<<"${DOC}")"
if [[ "${HASH_COMPUTED,,}" != "${HASH_EXPECTED,,}" ]]; then
  echo "FAIL: hash mismatch computed=${HASH_COMPUTED} expected=${HASH_EXPECTED}"
  exit 1
fi

echo "Checking on-chain record (best effort)..."
if [[ -n "${REPUTATION_ACTOR_ADDRESS:-}" && -n "${IPC_RPC_URL:-}" ]]; then
  RPC_PAYLOAD="{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"IPC.ReputationGetScore\",\"params\":[\"${REPUTATION_ACTOR_ADDRESS}\",\"${WALLET}\"]}"
  RPC_OUT="$(curl -sS -X POST "${IPC_RPC_URL}" -H "Content-Type: application/json" -d "${RPC_PAYLOAD}")"
  ONCHAIN_CID="$(python3 -c 'import json,sys;print((json.load(sys.stdin).get("result") or {}).get("evidence_cid",""))' <<<"${RPC_OUT}")"
  if [[ -n "${ONCHAIN_CID}" && "${ONCHAIN_CID}" != "${CID}" ]]; then
    echo "FAIL: on-chain CID mismatch"
    exit 1
  fi
fi

echo "PASS: score=${SCORE} tier=${TIER} cid=${CID}"
