#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
rm -rf output
mkdir -p output
docker build -t ipc-agent-test .

EXTRA_ENV=()
[ -n "${ANTHROPIC_MODEL:-}" ] && EXTRA_ENV+=(-e "ANTHROPIC_MODEL=$ANTHROPIC_MODEL")
[ -n "${ANTHROPIC_MAX_TOKENS:-}" ] && EXTRA_ENV+=(-e "ANTHROPIC_MAX_TOKENS=$ANTHROPIC_MAX_TOKENS")
[ -n "${CONTEXT_LIMIT:-}" ] && EXTRA_ENV+=(-e "CONTEXT_LIMIT=$CONTEXT_LIMIT")

docker run --rm \
  --user "$(id -u):$(id -g)" \
  -e "ANTHROPIC_API_KEY=${ANTHROPIC_API_KEY:?set ANTHROPIC_API_KEY}" \
  -e "TASK_PROMPT=$(cat task.txt)" \
  "${EXTRA_ENV[@]}" \
  -v "$(pwd)/output:/app/output" \
  ipc-agent-test
