#!/bin/bash

# Setup Anvil with IPC Keystore Keys
# This script starts Anvil and funds all accounts from the IPC keystore

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
ANVIL_PORT=8545
ANVIL_CHAIN_ID=31337
ANVIL_HOST="127.0.0.1"
RPC_URL="http://localhost:${ANVIL_PORT}"
MNEMONIC="test test test test test test test test test test test junk"
IPC_KEYSTORE="$HOME/.ipc/evm_keystore.json"
ANVIL_LOG_FILE="/tmp/anvil_ipc_keys.log"

# Default funding account (first account from the standard mnemonic)
FUNDER_ADDRESS="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
FUNDER_PRIVATE_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
INITIAL_BALANCE="10000" # ETH per account

echo -e "${BLUE}╔══════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  IPC Anvil Setup with Keystore Keys                     ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════╝${NC}"
echo ""

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
    echo "$response" | grep -o '"result":"[^"]*"' | cut -d'"' -f4 | xargs printf "%d" 2>/dev/null || echo "0"
}

# Function to kill existing Anvil processes
kill_anvil() {
    echo -e "${YELLOW}🔄 Stopping existing Anvil processes...${NC}"
    pkill -f "anvil.*$ANVIL_PORT" || true
    sleep 2
}

# Function to check if keystore exists
check_keystore() {
    if [ ! -f "$IPC_KEYSTORE" ]; then
        echo -e "${RED}❌ IPC keystore not found at: $IPC_KEYSTORE${NC}"
        echo ""
        echo "Please create your IPC keystore first using:"
        echo "  ipc-cli wallet import --keystore-path ~/.ipc"
        exit 1
    fi
    echo -e "${GREEN}✅ Found IPC keystore${NC}"
}

# Function to start Anvil
start_anvil() {
    echo -e "${BLUE}🚀 Starting Anvil...${NC}"
    echo "  Chain ID: $ANVIL_CHAIN_ID"
    echo "  Port: $ANVIL_PORT"
    echo "  RPC URL: $RPC_URL"
    echo ""

    # Start Anvil in the background
    anvil \
        --host "$ANVIL_HOST" \
        --port "$ANVIL_PORT" \
        --chain-id "$ANVIL_CHAIN_ID" \
        --mnemonic "$MNEMONIC" \
        --accounts 10 \
        --balance 1000000 \
        --block-time 1 \
        --gas-limit 30000000 \
        --gas-price 1000000000 \
        > "$ANVIL_LOG_FILE" 2>&1 &

    local anvil_pid=$!
    echo -e "${CYAN}  Process ID: $anvil_pid${NC}"

    # Wait for Anvil to be ready
    echo -e "${YELLOW}⏳ Waiting for Anvil to start...${NC}"
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
    echo ""
}

# Function to extract addresses from keystore
get_keystore_addresses() {
    if [ ! -f "$IPC_KEYSTORE" ]; then
        echo ""
        return 1
    fi

    # Parse JSON and extract addresses (excluding 'default-key')
    cat "$IPC_KEYSTORE" | grep -o '"address"[[:space:]]*:[[:space:]]*"[^"]*"' | \
        grep -o '0x[a-fA-F0-9]\{40\}' | sort -u
}

# Function to get balance
get_balance() {
    local address=$1
    local balance_hex=$(curl -s -X POST -H "Content-Type: application/json" \
        --data "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBalance\",\"params\":[\"$address\",\"latest\"],\"id\":1}" \
        "$RPC_URL" | grep -o '"result":"[^"]*"' | cut -d'"' -f4)

    # Convert hex to decimal
    if [ -n "$balance_hex" ]; then
        printf "%d" "$balance_hex" 2>/dev/null || echo "0"
    else
        echo "0"
    fi
}

# Function to fund an account
fund_account() {
    local to_address=$1
    local amount_eth=$2

    # Convert ETH to Wei (multiply by 10^18)
    local amount_wei=$(echo "$amount_eth * 1000000000000000000" | bc)
    local amount_hex=$(printf "0x%x" $amount_wei)

    # Send transaction from funder account
    local tx_hash=$(curl -s -X POST -H "Content-Type: application/json" \
        --data "{
            \"jsonrpc\":\"2.0\",
            \"method\":\"eth_sendTransaction\",
            \"params\":[{
                \"from\":\"$FUNDER_ADDRESS\",
                \"to\":\"$to_address\",
                \"value\":\"$amount_hex\",
                \"gas\":\"0x5208\"
            }],
            \"id\":1
        }" \
        "$RPC_URL" | grep -o '"result":"[^"]*"' | cut -d'"' -f4)

    if [ -n "$tx_hash" ] && [ "$tx_hash" != "null" ]; then
        return 0
    else
        return 1
    fi
}

# Function to fund all keystore accounts
fund_keystore_accounts() {
    echo -e "${BLUE}💰 Funding IPC keystore accounts...${NC}"
    echo ""

    local addresses=$(get_keystore_addresses)
    local count=0
    local funded=0
    local skipped=0

    if [ -z "$addresses" ]; then
        echo -e "${YELLOW}⚠️  No addresses found in keystore${NC}"
        return
    fi

    while IFS= read -r address; do
        count=$((count + 1))

        # Check if address already has sufficient balance
        local current_balance=$(get_balance "$address")
        local min_balance=$(echo "$INITIAL_BALANCE * 1000000000000000000 / 2" | bc) # Half of initial balance

        if [ "$current_balance" -ge "$min_balance" ]; then
            local balance_eth=$(echo "scale=2; $current_balance / 1000000000000000000" | bc)
            echo -e "${CYAN}  [$count] $address${NC}"
            echo -e "      ${GREEN}✓ Already funded ($balance_eth ETH)${NC}"
            skipped=$((skipped + 1))
        else
            echo -e "${CYAN}  [$count] $address${NC}"
            echo -n "      Funding with $INITIAL_BALANCE ETH... "

            if fund_account "$address" "$INITIAL_BALANCE"; then
                echo -e "${GREEN}✓${NC}"
                funded=$((funded + 1))
            else
                echo -e "${RED}✗ Failed${NC}"
            fi
        fi
    done <<< "$addresses"

    echo ""
    echo -e "${GREEN}✅ Funding complete${NC}"
    echo -e "   Total accounts: $count"
    echo -e "   Newly funded: $funded"
    echo -e "   Already funded: $skipped"
    echo ""
}

# Function to show account balances
show_balances() {
    echo -e "${BLUE}📊 Account Balances:${NC}"
    echo ""

    local addresses=$(get_keystore_addresses)
    local count=0

    while IFS= read -r address; do
        count=$((count + 1))
        local balance_wei=$(get_balance "$address")
        local balance_eth=$(echo "scale=4; $balance_wei / 1000000000000000000" | bc -l 2>/dev/null)

        echo -e "${CYAN}  [$count] $address${NC}"
        echo -e "      ${YELLOW}$balance_eth ETH${NC}"
    done <<< "$addresses"

    echo ""
}

# Function to check dependencies
check_dependencies() {
    local missing_deps=()

    if ! command -v anvil &> /dev/null; then
        missing_deps+=("anvil (from Foundry)")
    fi

    if ! command -v curl &> /dev/null; then
        missing_deps+=("curl")
    fi

    if ! command -v bc &> /dev/null; then
        missing_deps+=("bc (for balance calculations)")
    fi

    if ! command -v jq &> /dev/null; then
        echo -e "${YELLOW}⚠️  Warning: 'jq' not found. JSON parsing may be less reliable.${NC}"
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

# Function to save Anvil PID for easy management
save_anvil_info() {
    local pid=$(pgrep -f "anvil.*$ANVIL_PORT" | head -1)
    if [ -n "$pid" ]; then
        echo "$pid" > /tmp/anvil_ipc.pid
        echo -e "${CYAN}💾 Anvil PID saved to /tmp/anvil_ipc.pid${NC}"
    fi
}

# Function to create stop script
create_stop_script() {
    cat > /tmp/stop-anvil-ipc.sh << 'STOPSCRIPT'
#!/bin/bash
echo "Stopping Anvil..."
if [ -f /tmp/anvil_ipc.pid ]; then
    pid=$(cat /tmp/anvil_ipc.pid)
    kill $pid 2>/dev/null && echo "Anvil stopped (PID: $pid)" || echo "Anvil process not found"
    rm /tmp/anvil_ipc.pid
else
    pkill -f "anvil.*8545" && echo "Anvil stopped" || echo "No Anvil process found"
fi
STOPSCRIPT
    chmod +x /tmp/stop-anvil-ipc.sh
    echo -e "${CYAN}📝 Stop script created: /tmp/stop-anvil-ipc.sh${NC}"
}

# Main script logic
main() {
    check_dependencies
    check_keystore

    echo ""
    if check_anvil_running; then
        local current_chain_id=$(get_chain_id)
        echo -e "${GREEN}✅ Anvil is already running${NC}"
        echo "  Chain ID: $current_chain_id"
        echo "  RPC URL: $RPC_URL"
        echo ""

        echo -e "${BLUE}Do you want to:${NC}"
        echo "  1) Use existing Anvil and fund keystore accounts"
        echo "  2) Restart Anvil and fund accounts"
        echo "  3) Exit"
        echo ""
        read -p "Enter your choice (1-3): " choice

        case $choice in
            1)
                echo -e "${GREEN}📡 Using existing Anvil${NC}"
                echo ""
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
        echo -e "${YELLOW}🔍 Anvil is not running${NC}"
        start_anvil
    fi

    fund_keystore_accounts
    show_balances
    save_anvil_info
    create_stop_script

    echo ""
    echo -e "${GREEN}╔══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║  Setup Complete! 🎉                                      ║${NC}"
    echo -e "${GREEN}╚══════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${BLUE}📝 Next Steps:${NC}"
    echo "  • Anvil is running with all your IPC keystore accounts funded"
    echo "  • Use your IPC CLI with: --subnet /r31337"
    echo "  • RPC URL: $RPC_URL"
    echo "  • Chain ID: $ANVIL_CHAIN_ID"
    echo ""
    echo -e "${BLUE}🛠️  Management:${NC}"
    echo "  • Stop Anvil: /tmp/stop-anvil-ipc.sh"
    echo "  • View logs: cat $ANVIL_LOG_FILE"
    echo ""
    echo -e "${YELLOW}💡 Tip: Anvil is running in the background. Keep this terminal open or note the PID.${NC}"
    echo ""
}

# Handle Ctrl+C gracefully
trap 'echo -e "\n${YELLOW}Script interrupted. Anvil is still running in the background.${NC}"; exit 130' INT

# Run the main function
main

