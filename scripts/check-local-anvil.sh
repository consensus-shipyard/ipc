#!/bin/bash

# Check Local Anvil Status
# Quick status check for your Local Anvil development environment

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

RPC_URL="http://localhost:8545"
EXPECTED_CHAIN_ID=31337

echo -e "${BLUE}📊 Local Anvil Status Check${NC}"
echo "============================="

# Check if Anvil is running
if curl -s -X POST -H "Content-Type: application/json" \
    --data '{"jsonrpc":"2.0","method":"net_version","params":[],"id":1}' \
    "$RPC_URL" > /dev/null 2>&1; then

    echo -e "${GREEN}✅ Anvil is running${NC}"

    # Get chain ID
    response=$(curl -s -X POST -H "Content-Type: application/json" \
        --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
        "$RPC_URL")
    chain_id=$(echo "$response" | grep -o '"result":"[^"]*"' | cut -d'"' -f4 | xargs printf "%d")

    echo "  Chain ID: $chain_id"
    echo "  RPC URL: $RPC_URL"

    if [ "$chain_id" -eq "$EXPECTED_CHAIN_ID" ]; then
        echo -e "${GREEN}  ✅ Chain ID matches expected ($EXPECTED_CHAIN_ID)${NC}"
    else
        echo -e "${YELLOW}  ⚠️  Chain ID mismatch (expected $EXPECTED_CHAIN_ID)${NC}"
    fi
else
    echo -e "${RED}❌ Anvil is not running${NC}"
    echo "  Run: ./scripts/setup-local-anvil.sh"
    exit 1
fi

echo ""

# Check IPC keystore
echo -e "${BLUE}🔑 IPC Keystore Status:${NC}"
if command -v ipc-cli &> /dev/null; then
    if ipc-cli wallet list --keystore-path ~/.ipc 2>/dev/null | grep -q "0x"; then
        account_count=$(ipc-cli wallet list --keystore-path ~/.ipc 2>/dev/null | grep "0x" | wc -l)
        echo -e "${GREEN}  ✅ $account_count accounts in keystore${NC}"
    else
        echo -e "${YELLOW}  ⚠️  No accounts found in keystore${NC}"
        echo "  Run: ./scripts/setup-local-anvil.sh"
    fi
else
    echo -e "${RED}  ❌ ipc-cli not found${NC}"
fi

echo ""

# Check contract deployment status
echo -e "${BLUE}🏗️  Contract Deployment Status:${NC}"

# Check if gateway contract is deployed
gateway_addr="0xfaaddc93baf78e89dcf37ba67943e1be8f37bb8c"
gateway_code=$(curl -s -X POST -H "Content-Type: application/json" \
    --data "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getCode\",\"params\":[\"$gateway_addr\",\"latest\"],\"id\":1}" \
    "$RPC_URL" | grep -o '"result":"[^"]*"' | cut -d'"' -f4)

if [ "$gateway_code" != "0x" ] && [ ${#gateway_code} -gt 4 ]; then
    echo -e "${GREEN}  ✅ Gateway contract deployed at $gateway_addr${NC}"
else
    echo -e "${RED}  ❌ Gateway contract not deployed${NC}"
    echo "  Deploy contracts through the IPC UI"
fi

echo ""
echo -e "${BLUE}💡 Next steps:${NC}"
echo "  1. Open IPC UI at http://localhost:3000"
echo "  2. Select 'Local Anvil' network"
echo "  3. Deploy contracts if needed"
echo "  4. Create your subnet"