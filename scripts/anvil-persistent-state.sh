#!/bin/bash

# Anvil Persistent State Manager
# This script helps manage Anvil state persistence across restarts

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
ANVIL_PORT=8545
RPC_URL="http://localhost:${ANVIL_PORT}"
STATE_DIR="$HOME/.ipc/anvil-state"
STATE_FILE="$STATE_DIR/state.json"
IPC_KEYSTORE="$HOME/.ipc/evm_keystore.json"

echo -e "${BLUE}🔄 Anvil Persistent State Manager${NC}"
echo "===================================="
echo ""

# Create state directory if it doesn't exist
mkdir -p "$STATE_DIR"

# Function to check if Anvil is running
check_anvil_running() {
    if curl -s -X POST -H "Content-Type: application/json" \
        --data '{"jsonrpc":"2.0","method":"net_version","params":[],"id":1}' \
        "$RPC_URL" > /dev/null 2>&1; then
        return 0
    else
        return 1
    fi
}

# Function to dump current state
dump_state() {
    echo -e "${BLUE}📦 Dumping Anvil state...${NC}"

    if ! check_anvil_running; then
        echo -e "${RED}❌ Anvil is not running${NC}"
        return 1
    fi

    # Dump state using anvil_dumpState RPC method
    local state=$(curl -s -X POST -H "Content-Type: application/json" \
        --data '{"jsonrpc":"2.0","method":"anvil_dumpState","params":[],"id":1}' \
        "$RPC_URL")

    if echo "$state" | grep -q '"result"'; then
        echo "$state" | grep -o '"result":"[^"]*"' | cut -d'"' -f4 > "$STATE_FILE"
        echo -e "${GREEN}✅ State saved to: $STATE_FILE${NC}"

        # Save metadata
        local timestamp=$(date +%s)
        echo "{\"timestamp\": $timestamp, \"date\": \"$(date)\"}" > "$STATE_DIR/metadata.json"

        return 0
    else
        echo -e "${RED}❌ Failed to dump state${NC}"
        echo "Response: $state"
        return 1
    fi
}

# Function to load state
load_state() {
    echo -e "${BLUE}📥 Loading Anvil state...${NC}"

    if [ ! -f "$STATE_FILE" ]; then
        echo -e "${RED}❌ No saved state found at: $STATE_FILE${NC}"
        return 1
    fi

    if ! check_anvil_running; then
        echo -e "${RED}❌ Anvil is not running${NC}"
        return 1
    fi

    local state_hex=$(cat "$STATE_FILE")

    # Load state using anvil_loadState RPC method
    local response=$(curl -s -X POST -H "Content-Type: application/json" \
        --data "{\"jsonrpc\":\"2.0\",\"method\":\"anvil_loadState\",\"params\":[\"$state_hex\"],\"id\":1}" \
        "$RPC_URL")

    if echo "$response" | grep -q '"result"'; then
        echo -e "${GREEN}✅ State loaded successfully${NC}"

        if [ -f "$STATE_DIR/metadata.json" ]; then
            echo -e "${BLUE}State metadata:${NC}"
            cat "$STATE_DIR/metadata.json" | grep -o '"date":"[^"]*"' | cut -d'"' -f4
        fi

        return 0
    else
        echo -e "${RED}❌ Failed to load state${NC}"
        echo "Response: $response"
        return 1
    fi
}

# Function to start Anvil with state
start_with_state() {
    echo -e "${BLUE}🚀 Starting Anvil with persistent state...${NC}"

    if [ ! -f "$STATE_FILE" ]; then
        echo -e "${YELLOW}⚠️  No saved state found. Starting fresh...${NC}"
        echo ""
        echo "After starting, fund your accounts and run:"
        echo "  $0 save"
        return 1
    fi

    # Note: Anvil doesn't support --load-state at startup via CLI
    # We need to start Anvil and then load the state
    echo -e "${YELLOW}⚠️  Anvil must be started first, then state can be loaded${NC}"
    echo ""
    echo "Steps:"
    echo "  1. Start Anvil: anvil --port 8545"
    echo "  2. Load state: $0 load"
    return 1
}

# Function to show state info
show_info() {
    echo -e "${BLUE}📊 State Information${NC}"
    echo ""

    if [ -f "$STATE_DIR/metadata.json" ]; then
        echo -e "${GREEN}Saved state exists${NC}"
        echo "  Location: $STATE_FILE"
        echo "  Saved: $(cat "$STATE_DIR/metadata.json" | grep -o '"date":"[^"]*"' | cut -d'"' -f4)"

        local size=$(du -h "$STATE_FILE" 2>/dev/null | cut -f1)
        echo "  Size: $size"
    else
        echo -e "${YELLOW}No saved state found${NC}"
    fi

    echo ""
    if check_anvil_running; then
        echo -e "${GREEN}Anvil is currently running${NC}"
    else
        echo -e "${YELLOW}Anvil is not running${NC}"
    fi
}

# Function to generate funded accounts file for Anvil
generate_funded_accounts_file() {
    echo -e "${BLUE}📝 Generating funded accounts configuration...${NC}"

    if [ ! -f "$IPC_KEYSTORE" ]; then
        echo -e "${RED}❌ IPC keystore not found${NC}"
        return 1
    fi

    local accounts_file="$STATE_DIR/funded-accounts.txt"

    # Extract addresses from keystore
    cat "$IPC_KEYSTORE" | grep -o '"address"[[:space:]]*:[[:space:]]*"0x[a-fA-F0-9]\{40\}"' | \
        grep -o '0x[a-fA-F0-9]\{40\}' | sort -u > "$accounts_file"

    local count=$(wc -l < "$accounts_file")
    echo -e "${GREEN}✅ Generated account list with $count addresses${NC}"
    echo "  File: $accounts_file"
    echo ""
    echo "You can use this file to verify all your accounts are funded."
}

# Show usage
usage() {
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  save    - Save current Anvil state"
    echo "  load    - Load saved Anvil state"
    echo "  info    - Show state information"
    echo "  list    - Generate list of funded accounts"
    echo ""
    echo "Examples:"
    echo "  $0 save    # Save current state for later"
    echo "  $0 load    # Load previously saved state"
    echo ""
}

# Main logic
case "${1:-}" in
    save)
        dump_state
        ;;
    load)
        load_state
        ;;
    start)
        start_with_state
        ;;
    info)
        show_info
        ;;
    list)
        generate_funded_accounts_file
        ;;
    *)
        usage
        exit 1
        ;;
esac

