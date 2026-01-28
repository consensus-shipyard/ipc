#!/bin/bash
# Test Anvil connectivity from remote VMs through SSH tunnels

set -euo pipefail

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Validator info
VALIDATORS=(
    "philip@34.73.187.192:validator-1"
    "philip@35.237.175.224:validator-2"
    "philip@34.75.205.89:validator-3"
)

REMOTE_PORT=8545

echo -e "${GREEN}Testing Anvil connectivity from remote VMs...${NC}"
echo ""

for validator_info in "${VALIDATORS[@]}"; do
    IFS=':' read -r validator name <<< "$validator_info"

    echo -e "${YELLOW}Testing ${name} (${validator})${NC}"

    # Test if port is listening
    echo -n "  Port check: "
    if ssh "${validator}" "nc -z localhost ${REMOTE_PORT} 2>/dev/null"; then
        echo -e "${GREEN}✓${NC} Port ${REMOTE_PORT} is accessible"
    else
        echo -e "${RED}✗${NC} Port ${REMOTE_PORT} is NOT accessible"
        echo "    Make sure the tunnel is running!"
        continue
    fi

    # Test Anvil RPC
    echo -n "  RPC check: "
    CHAIN_ID=$(ssh "${validator}" "curl -s -X POST -H 'Content-Type: application/json' \
        --data '{\"jsonrpc\":\"2.0\",\"method\":\"eth_chainId\",\"params\":[],\"id\":1}' \
        http://localhost:${REMOTE_PORT} 2>/dev/null | grep -o '\"result\":\"[^\"]*\"' | cut -d'\"' -f4")

    if [ -n "$CHAIN_ID" ]; then
        echo -e "${GREEN}✓${NC} Anvil responding (chainId: ${CHAIN_ID})"
    else
        echo -e "${RED}✗${NC} No response from Anvil"
    fi

    echo ""
done

echo -e "${GREEN}Test complete!${NC}"

