#!/bin/bash
# Debug Relayer Error Script
# Helps diagnose why checkpoint submission is failing

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

ANVIL_RPC="http://localhost:8555"
GATEWAY_ADDR="0x0cdd23b138f20e4744568f61c474ffe35c0bc1fb"
SUBNET_ADDR="0xf7226ed8aa4ed4c0a01edec290f0d015ddf414f2"

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}   IPC Relayer Error Diagnostic Tool${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Test 1: Check if Anvil is running
echo -e "${YELLOW}[1/7] Checking if Anvil is accessible...${NC}"
if curl -s -X POST -H "Content-Type: application/json" \
    --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
    "$ANVIL_RPC" > /dev/null 2>&1; then
    BLOCK=$(curl -s -X POST -H "Content-Type: application/json" \
        --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
        "$ANVIL_RPC" | jq -r '.result' | xargs printf "%d\n")
    echo -e "${GREEN}✓ Anvil is running at block $BLOCK${NC}"
else
    echo -e "${RED}✗ Cannot connect to Anvil at $ANVIL_RPC${NC}"
    echo -e "${YELLOW}  Make sure Anvil is running and accessible${NC}"
    exit 1
fi
echo ""

# Test 2: Check if Gateway contract exists
echo -e "${YELLOW}[2/7] Checking if Gateway contract is deployed...${NC}"
GATEWAY_CODE=$(curl -s -X POST -H "Content-Type: application/json" \
    --data "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getCode\",\"params\":[\"$GATEWAY_ADDR\",\"latest\"],\"id\":1}" \
    "$ANVIL_RPC" | jq -r '.result')

if [ "$GATEWAY_CODE" = "0x" ]; then
    echo -e "${RED}✗ No contract found at Gateway address: $GATEWAY_ADDR${NC}"
    echo -e "${YELLOW}  You need to deploy the IPC contracts to Anvil first${NC}"
    echo -e "${YELLOW}  Run: cd contracts && make deploy-ipc${NC}"
    exit 1
else
    echo -e "${GREEN}✓ Gateway contract exists (${#GATEWAY_CODE} bytes)${NC}"
fi
echo ""

# Test 3: Check if Subnet Actor contract exists
echo -e "${YELLOW}[3/7] Checking if Subnet Actor contract is deployed...${NC}"
SUBNET_CODE=$(curl -s -X POST -H "Content-Type: application/json" \
    --data "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getCode\",\"params\":[\"$SUBNET_ADDR\",\"latest\"],\"id\":1}" \
    "$ANVIL_RPC" | jq -r '.result')

if [ "$SUBNET_CODE" = "0x" ]; then
    echo -e "${RED}✗ No contract found at Subnet Actor address: $SUBNET_ADDR${NC}"
    echo -e "${YELLOW}  The subnet may not be properly created on the parent chain${NC}"
    exit 1
else
    echo -e "${GREEN}✓ Subnet Actor contract exists (${#SUBNET_CODE} bytes)${NC}"
fi
echo ""

# Test 4: Check last bottom-up checkpoint height on subnet
echo -e "${YELLOW}[4/7] Checking last bottom-up checkpoint height...${NC}"
LAST_CHECKPOINT=$(curl -s -X POST -H "Content-Type: application/json" \
    --data "{\"jsonrpc\":\"2.0\",\"method\":\"eth_call\",\"params\":[{\"to\":\"$SUBNET_ADDR\",\"data\":\"0xf566aa63\"},\"latest\"],\"id\":1}" \
    "$ANVIL_RPC" | jq -r '.result' | xargs printf "%d\n" 2>/dev/null || echo "error")

if [ "$LAST_CHECKPOINT" = "error" ]; then
    echo -e "${YELLOW}⚠ Could not query last checkpoint height (contract might not support this)${NC}"
else
    echo -e "${GREEN}✓ Last submitted checkpoint height: $LAST_CHECKPOINT${NC}"
fi
echo ""

# Test 5: Check if subnet is active/registered
echo -e "${YELLOW}[5/7] Checking if subnet is active...${NC}"
# Try to call bottomUpCheckPeriod on subnet actor
CHECK_PERIOD=$(curl -s -X POST -H "Content-Type: application/json" \
    --data "{\"jsonrpc\":\"2.0\",\"method\":\"eth_call\",\"params\":[{\"to\":\"$SUBNET_ADDR\",\"data\":\"0x5bb47808\"},\"latest\"],\"id\":1}" \
    "$ANVIL_RPC" | jq -r '.result' | xargs printf "%d\n" 2>/dev/null || echo "error")

if [ "$CHECK_PERIOD" = "error" ] || [ "$CHECK_PERIOD" = "0" ]; then
    echo -e "${RED}✗ Subnet appears to be inactive or not properly configured${NC}"
    echo -e "${YELLOW}  Bottom-up checkpoint period: $CHECK_PERIOD${NC}"
else
    echo -e "${GREEN}✓ Subnet is active with checkpoint period: $CHECK_PERIOD blocks${NC}"
fi
echo ""

# Test 6: Check subnet validator power/membership
echo -e "${YELLOW}[6/7] Checking validator membership...${NC}"
# This is a more complex check - just indicate it should be done
echo -e "${YELLOW}  Manual check required: Verify validators are properly joined${NC}"
echo -e "${YELLOW}  Run: ipc-cli subnet list-validators --subnet /r31337/t410f...${NC}"
echo ""

# Test 7: Check for pending checkpoints in subnet
echo -e "${YELLOW}[7/7] Summary and Recommendations${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "${GREEN}Common Issues and Solutions:${NC}"
echo ""
echo -e "1. ${YELLOW}Checkpoint doesn't exist yet${NC}"
echo -e "   - The subnet needs to produce blocks equal to the checkpoint period"
echo -e "   - Current checkpoint period: ${CHECK_PERIOD} blocks"
echo -e "   - Wait for subnet to reach next checkpoint height"
echo ""
echo -e "2. ${YELLOW}Invalid signatures${NC}"
echo -e "   - Validator addresses might not match between subnet and parent"
echo -e "   - Signatures might be incorrectly formatted"
echo -e "   - Check validator key configuration"
echo ""
echo -e "3. ${YELLOW}Quorum not reached${NC}"
echo -e "   - Not enough validators have signed the checkpoint"
echo -e "   - Check that validators are running and participating"
echo ""
echo -e "4. ${YELLOW}Bottom-up checkpointing disabled${NC}"
echo -e "   - Your config shows: bottomup.enabled = false"
echo -e "   - Enable it in ipc-subnet-config.yml if you want to run relayer"
echo ""
echo -e "${BLUE}To get more detailed error information:${NC}"
echo -e "  Run the relayer with: ${GREEN}RUST_LOG=debug,ipc_provider=trace${NC}"
echo ""
echo -e "${BLUE}To manually check contract state:${NC}"
echo -e "  cast call $SUBNET_ADDR \"lastBottomUpCheckpointHeight()\" --rpc-url $ANVIL_RPC"
echo -e "  cast call $GATEWAY_ADDR \"bottomUpCheckPeriod()\" --rpc-url $ANVIL_RPC"
echo ""

