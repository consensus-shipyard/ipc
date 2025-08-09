#!/bin/bash

# Setup Local Anvil for IPC Development
# This script helps set up a fresh Local Anvil environment or use an existing one
# and imports the generated accounts into the IPC keystore

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
ANVIL_PORT=8545
ANVIL_CHAIN_ID=31337
ANVIL_HOST="127.0.0.1"
RPC_URL="http://localhost:${ANVIL_PORT}"
MNEMONIC="test test test test test test test test test test test junk"
ACCOUNTS_TO_IMPORT=5
ANVIL_LOG_FILE="/tmp/anvil_setup.log"

echo -e "${BLUE}🔧 IPC Local Anvil Setup Script${NC}"
echo "=================================="

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

# Function to get chain ID from running Anvil
get_chain_id() {
    local response=$(curl -s -X POST -H "Content-Type: application/json" \
        --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
        "$RPC_URL")
    echo "$response" | grep -o '"result":"[^"]*"' | cut -d'"' -f4 | xargs printf "%d"
}

# Function to kill existing Anvil processes
kill_anvil() {
    echo -e "${YELLOW}🔄 Stopping existing Anvil processes...${NC}"
    pkill -f "anvil" || true
    sleep 2
}

# Function to start Anvil and capture account info
start_anvil() {
    echo -e "${BLUE}🚀 Starting Anvil...${NC}"
    echo "  Chain ID: $ANVIL_CHAIN_ID"
    echo "  Port: $ANVIL_PORT"
    echo "  Mnemonic: $MNEMONIC"

    # Start Anvil and capture its output
    anvil \
        --host "$ANVIL_HOST" \
        --port "$ANVIL_PORT" \
        --chain-id "$ANVIL_CHAIN_ID" \
        --mnemonic "$MNEMONIC" \
        --accounts 10 \
        --block-time 1 \
        > "$ANVIL_LOG_FILE" 2>&1 &

    local anvil_pid=$!
    echo "  PID: $anvil_pid"

    # Wait for Anvil to be ready
    echo -e "${YELLOW}⏳ Waiting for Anvil to be ready...${NC}"
    local timeout=30
    while ! check_anvil_running && [ $timeout -gt 0 ]; do
        sleep 1
        timeout=$((timeout - 1))
    done

    if [ $timeout -eq 0 ]; then
        echo -e "${RED}❌ Timeout waiting for Anvil to start${NC}"
        echo "Anvil log:"
        cat "$ANVIL_LOG_FILE" 2>/dev/null || echo "No log file found"
        exit 1
    fi

    echo -e "${GREEN}✅ Anvil is running${NC}"

    # Give Anvil a moment to write its startup info
    sleep 2
}

# Function to extract accounts from Anvil log
extract_accounts_from_log() {
    if [ ! -f "$ANVIL_LOG_FILE" ]; then
        echo -e "${RED}❌ Anvil log file not found${NC}"
        return 1
    fi

    # Extract private keys and addresses from Anvil output
    # Anvil outputs lines like: "Private Key: 0x..."
    grep "Private Key:" "$ANVIL_LOG_FILE" | head -$ACCOUNTS_TO_IMPORT | sed 's/.*Private Key: //'
}

# Function to extract addresses from Anvil log
extract_addresses_from_log() {
    if [ ! -f "$ANVIL_LOG_FILE" ]; then
        echo -e "${RED}❌ Anvil log file not found${NC}"
        return 1
    fi

    # Extract addresses from Anvil output
    # Anvil outputs lines with addresses in parentheses
    grep -o "(0x[a-fA-F0-9]\{40\})" "$ANVIL_LOG_FILE" | head -$ACCOUNTS_TO_IMPORT | tr -d '()'
}

# Predefined accounts from the standard test mnemonic
# These are deterministic and will always be the same
get_predefined_accounts() {
    cat << 'EOF'
0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266:0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
0x70997970C51812dc3A010C7d01b50e0d17dc79C8:0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d
0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC:0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a
0x90F79bf6EB2c4f870365E785982E1f101E93b906:0x7c852118294e51e653712a81e05800f419141751be58f605c371e15141b007a6
0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65:0x47e179ec197488593b187f80a00eb0da91f1b9d0b13f8733639f19c30a34926a
EOF
}

# Function to import accounts into IPC keystore
import_accounts() {
    echo -e "${BLUE}🔑 Importing accounts into IPC keystore...${NC}"

    # Use predefined accounts (deterministic from mnemonic)
    local account_data=$(get_predefined_accounts)
    local count=0

    while IFS=':' read -r address private_key && [ $count -lt $ACCOUNTS_TO_IMPORT ]; do
        count=$((count + 1))
        echo -e "${YELLOW}  Account $count: $address${NC}"

        # Import the account into IPC CLI wallet
        if echo "$private_key" | ipc-cli wallet import --keystore-path ~/.ipc 2>/dev/null; then
            echo -e "${GREEN}    ✅ Imported successfully${NC}"
        else
            echo -e "${YELLOW}    ⚠️  Account may already exist in keystore${NC}"
        fi
    done <<< "$account_data"
}

# Function to list accounts in keystore
list_keystore_accounts() {
    echo -e "${BLUE}📋 Accounts in IPC keystore:${NC}"
    if ipc-cli wallet list --keystore-path ~/.ipc 2>/dev/null; then
        echo ""
    else
        echo -e "${YELLOW}  No accounts found or error accessing keystore${NC}"
    fi
}

# Function to show account balances
show_balances() {
    echo -e "${BLUE}💰 Account balances on Local Anvil:${NC}"

    local account_data=$(get_predefined_accounts)
    local count=0

    while IFS=':' read -r address private_key && [ $count -lt $ACCOUNTS_TO_IMPORT ]; do
        count=$((count + 1))
        local balance_hex=$(curl -s -X POST -H "Content-Type: application/json" \
            --data "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBalance\",\"params\":[\"$address\",\"latest\"],\"id\":1}" \
            "$RPC_URL" | grep -o '"result":"[^"]*"' | cut -d'"' -f4)

        local balance_wei=$((balance_hex))
        local balance_eth=$(printf "%.4f" $(echo "scale=4; $balance_wei / 1000000000000000000" | bc -l 2>/dev/null || echo "10000.0000"))

        echo -e "${YELLOW}  Account $count ($address): $balance_eth ETH${NC}"
    done <<< "$account_data"
}

# Function to check dependencies
check_dependencies() {
    local missing_deps=()

    if ! command -v anvil &> /dev/null; then
        missing_deps+=("anvil (from Foundry)")
    fi

    if ! command -v ipc-cli &> /dev/null; then
        missing_deps+=("ipc-cli")
    fi

    if ! command -v curl &> /dev/null; then
        missing_deps+=("curl")
    fi

    if ! command -v bc &> /dev/null; then
        missing_deps+=("bc (for balance calculations)")
    fi

    if [ ${#missing_deps[@]} -ne 0 ]; then
        echo -e "${RED}❌ Missing dependencies:${NC}"
        for dep in "${missing_deps[@]}"; do
            echo "  - $dep"
        done
        echo ""
        echo "Please install the missing dependencies and try again."
        exit 1
    fi
}

# Main script logic
main() {
    check_dependencies

    echo ""
    if check_anvil_running; then
        local current_chain_id=$(get_chain_id)
        echo -e "${GREEN}✅ Anvil is already running${NC}"
        echo "  Chain ID: $current_chain_id"
        echo "  RPC URL: $RPC_URL"
        echo ""

        if [ "$current_chain_id" -eq "$ANVIL_CHAIN_ID" ]; then
            echo -e "${BLUE}Do you want to:${NC}"
            echo "  1) Use the existing Anvil instance"
            echo "  2) Restart Anvil with fresh accounts"
            echo "  3) Exit"
            echo ""
            read -p "Enter your choice (1-3): " choice

            case $choice in
                1)
                    echo -e "${GREEN}📡 Using existing Anvil instance${NC}"
                    ;;
                2)
                    kill_anvil
                    start_anvil
                    ;;
                3)
                    echo "Exiting..."
                    exit 0
                    ;;
                *)
                    echo -e "${RED}Invalid choice. Exiting...${NC}"
                    exit 1
                    ;;
            esac
        else
            echo -e "${YELLOW}⚠️  Anvil is running with different chain ID ($current_chain_id vs $ANVIL_CHAIN_ID)${NC}"
            echo "Restarting with correct chain ID..."
            kill_anvil
            start_anvil
        fi
    else
        echo -e "${YELLOW}🔍 Anvil is not running${NC}"
        start_anvil
    fi

    echo ""
    import_accounts
    echo ""
    list_keystore_accounts
    echo ""
    show_balances

    # Cleanup log file
    rm -f "$ANVIL_LOG_FILE"

    echo ""
    echo -e "${GREEN}🎉 Setup complete!${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo -e "${BLUE}📝 Next steps:${NC}"
    echo "  1. Open the IPC UI and select 'Local Anvil' network"
    echo "  2. Deploy IPC contracts through the UI"
    echo "  3. Create and deploy your subnet"
    echo ""
    echo -e "${BLUE}🌐 Network Info:${NC}"
    echo "  RPC URL: $RPC_URL"
    echo "  Chain ID: $ANVIL_CHAIN_ID"
    echo "  Accounts imported: $ACCOUNTS_TO_IMPORT"
    echo ""
    echo -e "${YELLOW}💡 Tip: Keep this terminal open to keep Anvil running${NC}"
}

# Run the main function
main