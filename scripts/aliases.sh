#!/bin/bash

# IPC Development Aliases and Shortcuts
# Source this file in your shell: source scripts/aliases.sh

# Anvil Management
alias anvil-ipc='./scripts/setup-anvil-with-ipc-keys.sh'
alias anvil-start='./scripts/quick-anvil.sh'
alias anvil-stop='/tmp/stop-anvil-ipc.sh'
alias anvil-logs='cat /tmp/anvil_ipc_keys.log'
alias anvil-state='./scripts/anvil-persistent-state.sh'

# Anvil State Management
alias anvil-save='./scripts/anvil-persistent-state.sh save'
alias anvil-load='./scripts/anvil-persistent-state.sh load'
alias anvil-info='./scripts/anvil-persistent-state.sh info'

# IPC CLI Shortcuts (with keystore)
alias ipc='ipc-cli --keystore-path ~/.ipc'
alias ipc-wallet='ipc-cli wallet --keystore-path ~/.ipc'
alias ipc-subnet='ipc-cli subnet --keystore-path ~/.ipc'

# Quick Commands
alias ipc-accounts='ipc-cli wallet list --keystore-path ~/.ipc'
alias ipc-default='ipc-cli wallet get-default --keystore-path ~/.ipc'

# Anvil RPC Helpers
function anvil-balance() {
    local addr=${1:-"0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"}
    curl -s -X POST -H "Content-Type: application/json" \
        --data "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBalance\",\"params\":[\"$addr\",\"latest\"],\"id\":1}" \
        http://localhost:8545 | jq -r '.result' | xargs printf "%d\n" | awk '{print $1/1000000000000000000 " ETH"}'
}

function anvil-blocknum() {
    curl -s -X POST -H "Content-Type: application/json" \
        --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
        http://localhost:8545 | jq -r '.result' | xargs printf "%d\n"
}

function anvil-accounts() {
    echo "Reading accounts from IPC keystore..."
    cat ~/.ipc/evm_keystore.json | jq -r '.[] | select(.address != "default-key") | .address'
}

# Color output helpers
export ANVIL_GREEN='\033[0;32m'
export ANVIL_BLUE='\033[0;34m'
export ANVIL_NC='\033[0m'

echo -e "${ANVIL_GREEN}✅ IPC Development aliases loaded${ANVIL_NC}"
echo ""
echo "Available commands:"
echo "  anvil-start     - Start Anvil with IPC keystore funding"
echo "  anvil-stop      - Stop Anvil"
echo "  anvil-logs      - View Anvil logs"
echo "  anvil-save      - Save current Anvil state"
echo "  anvil-load      - Load saved Anvil state"
echo "  anvil-balance   - Check account balance (default or specify address)"
echo "  anvil-accounts  - List all IPC keystore accounts"
echo "  ipc             - IPC CLI with keystore"
echo "  ipc-accounts    - List IPC wallet accounts"
echo ""

