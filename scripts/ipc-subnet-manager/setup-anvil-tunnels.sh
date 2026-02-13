#!/bin/bash
# Setup SSH tunnels from local Anvil to remote validator nodes
# This allows remote VMs to access Anvil running on localhost

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CONFIG_FILE="${SCRIPT_DIR}/ipc-subnet-config.yml"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Parse YAML to get validator IPs and SSH users
# You'll need yq installed or we'll use basic grep
parse_config() {
    if command -v yq &> /dev/null; then
        # Using yq for proper YAML parsing
        VALIDATOR_COUNT=$(yq eval '.validators | length' "$CONFIG_FILE")
    else
        # Fallback to grep (less robust but works for simple cases)
        VALIDATOR_COUNT=3
    fi
}

# Extract validator info
get_validator_info() {
    local idx=$1
    if command -v yq &> /dev/null; then
        VALIDATOR_IP=$(yq eval ".validators[$idx].ip" "$CONFIG_FILE")
        VALIDATOR_USER=$(yq eval ".validators[$idx].ssh_user" "$CONFIG_FILE")
        VALIDATOR_NAME=$(yq eval ".validators[$idx].name" "$CONFIG_FILE")
    else
        # Fallback: hardcoded from config
        case $idx in
            0)
                VALIDATOR_IP="34.73.187.192"
                VALIDATOR_USER="philip"
                VALIDATOR_NAME="validator-1"
                ;;
            1)
                VALIDATOR_IP="35.237.175.224"
                VALIDATOR_USER="philip"
                VALIDATOR_NAME="validator-2"
                ;;
            2)
                VALIDATOR_IP="34.75.205.89"
                VALIDATOR_USER="philip"
                VALIDATOR_NAME="validator-3"
                ;;
        esac
    fi
}

# Local Anvil port
LOCAL_ANVIL_PORT=8545
# Remote port on VMs (can be the same or different)
REMOTE_ANVIL_PORT=8555

echo -e "${GREEN}Setting up SSH tunnels to remote validators...${NC}"
echo -e "Local Anvil: localhost:${LOCAL_ANVIL_PORT}"
echo ""

# Parse config
parse_config

# Array to store background process PIDs
declare -a TUNNEL_PIDS

# Cleanup function
cleanup() {
    echo -e "\n${YELLOW}Cleaning up SSH tunnels...${NC}"
    for pid in "${TUNNEL_PIDS[@]}"; do
        if ps -p "$pid" > /dev/null 2>&1; then
            echo "Killing tunnel process $pid"
            kill "$pid" 2>/dev/null || true
        fi
    done
    exit 0
}

# Register cleanup on script exit
trap cleanup SIGINT SIGTERM EXIT

# Setup tunnels for each validator
for i in $(seq 0 $((VALIDATOR_COUNT - 1))); do
    get_validator_info $i

    echo -e "${GREEN}Setting up tunnel for ${VALIDATOR_NAME}${NC}"
    echo -e "  Remote: ${VALIDATOR_USER}@${VALIDATOR_IP}"
    echo -e "  Mapping: ${VALIDATOR_IP}:${REMOTE_ANVIL_PORT} -> localhost:${LOCAL_ANVIL_PORT}"

    # Create reverse SSH tunnel
    # -N: Don't execute remote command
    # -R: Reverse port forwarding (remote:local)
    # -o ServerAliveInterval=60: Keep connection alive
    # -o ExitOnForwardFailure=yes: Exit if tunnel can't be established
    # -o LogLevel=ERROR: Suppress setsockopt warnings
    ssh -N \
        -R ${REMOTE_ANVIL_PORT}:localhost:${LOCAL_ANVIL_PORT} \
        -o ServerAliveInterval=60 \
        -o ServerAliveCountMax=3 \
        -o ExitOnForwardFailure=yes \
        -o LogLevel=ERROR \
        ${VALIDATOR_USER}@${VALIDATOR_IP} 2>/dev/null &

    TUNNEL_PID=$!
    TUNNEL_PIDS+=("$TUNNEL_PID")

    echo -e "  ${GREEN}✓${NC} Tunnel established (PID: $TUNNEL_PID)"
    echo ""

    # Give it a moment to establish
    sleep 1

    # Check if tunnel is still running
    if ! ps -p "$TUNNEL_PID" > /dev/null 2>&1; then
        echo -e "  ${RED}✗${NC} Tunnel failed to establish!"
        exit 1
    fi
done

echo -e "${GREEN}All tunnels established successfully!${NC}"
echo ""
echo "The remote VMs can now access Anvil via:"
echo "  http://localhost:${REMOTE_ANVIL_PORT}"
echo ""
echo "Press Ctrl+C to close all tunnels and exit."
echo ""

# Keep script running and monitor tunnels
while true; do
    sleep 5

    # Check if all tunnels are still alive
    for i in "${!TUNNEL_PIDS[@]}"; do
        pid="${TUNNEL_PIDS[$i]}"
        if ! ps -p "$pid" > /dev/null 2>&1; then
            echo -e "${RED}Tunnel $pid died unexpectedly!${NC}"
            cleanup
        fi
    done
done

