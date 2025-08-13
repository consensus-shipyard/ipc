#!/bin/bash

# IPC Subnet Deployment Validation Script
# This script validates all configured subnets in ~/.ipc/config.toml
# and checks their deployment status on the blockchain

# set -e removed to prevent early exit on command failures

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
CONFIG_FILE="$HOME/.ipc/config.toml"
RPC_URL="${RPC_URL:-http://localhost:8545}"
IPC_CLI="${IPC_CLI:-./target/release/ipc-cli}"

# Check if required tools are available
check_dependencies() {
    echo -e "${BLUE}Checking dependencies...${NC}"

    if ! command -v cast &> /dev/null; then
        echo -e "${RED}Error: 'cast' command not found. Please install foundry.${NC}"
        exit 1
    fi

    if ! command -v jq &> /dev/null; then
        echo -e "${RED}Error: 'jq' command not found. Please install jq.${NC}"
        exit 1
    fi

    if [[ ! -f "$IPC_CLI" ]]; then
        echo -e "${RED}Error: IPC CLI not found at $IPC_CLI${NC}"
        echo -e "${YELLOW}Please build it with: cargo build --release --bin ipc-cli${NC}"
        exit 1
    fi

    if [[ ! -f "$CONFIG_FILE" ]]; then
        echo -e "${RED}Error: IPC config file not found at $CONFIG_FILE${NC}"
        exit 1
    fi

    echo -e "${GREEN}✓ All dependencies found${NC}"
}

# Parse subnet information from config.toml
parse_config() {
    echo -e "${BLUE}Parsing IPC configuration...${NC}"

    # Extract subnet IDs from the TOML array format
    # Look for lines with 'id = "/subnet/path"'
    subnets=($(grep '^id = "' "$CONFIG_FILE" | cut -d'"' -f2))

    if [[ ${#subnets[@]} -eq 0 ]]; then
        echo -e "${YELLOW}No subnets found in configuration${NC}"
        exit 0
    fi

    echo -e "${GREEN}✓ Found ${#subnets[@]} configured subnet(s)${NC}"
    for subnet in "${subnets[@]}"; do
        echo "  - $subnet"
    done
}

# Get subnet configuration details
get_subnet_config() {
    local subnet_id="$1"

        # Find the subnet section and extract config values
    # This is complex due to TOML array format, so we'll use awk
    local config_data=$(awk -v subnet="$subnet_id" '
        /^\[\[subnets\]\]/ {
            # If we found a subnet in the previous section, output it before starting new one
            if (found_subnet && in_config) {
                print gateway "|" registry "|" rpc; exit
            }
            in_subnet = 1; found_subnet = 0; in_config = 0; gateway = ""; registry = ""; rpc = ""; next
        }
        in_subnet && /^id = / {
            gsub(/id = "/, ""); gsub(/"/, "")
            if ($0 == subnet) found_subnet = 1
            else { in_subnet = 0; found_subnet = 0 }
            next
        }
        found_subnet && /^\[subnets\.config\]/ { in_config = 1; next }
        found_subnet && in_config && /^gateway_addr = / {
            gsub(/gateway_addr = "/, ""); gsub(/"/, ""); gateway = $0
        }
        found_subnet && in_config && /^registry_addr = / {
            gsub(/registry_addr = "/, ""); gsub(/"/, ""); registry = $0
        }
        found_subnet && in_config && /^provider_http = / {
            gsub(/provider_http = "/, ""); gsub(/"/, ""); rpc = $0
        }
        END {
            if (found_subnet && in_config) print gateway "|" registry "|" rpc
        }
    ' "$CONFIG_FILE" | head -1)

    echo "$config_data"
}

# Check if a contract exists at an address
check_contract_exists() {
    local address="$1"
    local rpc="$2"

    if [[ -z "$address" || "$address" == "null" ]]; then
        echo "false"
        return
    fi

    local code=$(cast code "$address" --rpc-url "$rpc" 2>/dev/null || echo "0x")

    if [[ "$code" == "0x" || -z "$code" ]]; then
        echo "false"
    else
        echo "true"
    fi
}

# Convert f4 address to Ethereum address
f4_to_eth() {
    local f4_addr="$1"

    # Root subnets (like /r31337) don't have actor addresses
    if [[ "$f4_addr" =~ ^/r[0-9]+$ ]]; then
        echo "N/A-ROOT-SUBNET"
        return
    fi

    # Extract the actor part (everything after the last '/')
    local actor_part=$(echo "$f4_addr" | awk -F'/' '{print $NF}')

    # Check if actor part looks like a valid f4 address
    if [[ ! "$actor_part" =~ ^t4[0-9a-z]+$ ]]; then
        echo "N/A-INVALID-FORMAT"
        return
    fi

    # Use IPC CLI to convert
    local eth_addr=$("$IPC_CLI" util f4-to-eth-addr --addr "$actor_part" 2>/dev/null | grep -o '0x[a-fA-F0-9]*' | head -1)

    if [[ -z "$eth_addr" ]]; then
        echo "N/A-CONVERSION-FAILED"
    else
        echo "$eth_addr"
    fi
}

# Check subnet registration with parent gateway
check_subnet_registration() {
    local subnet_id="$1"
    local parent_id="$2"
    local gateway_addr="$3"
    local rpc="$4"

    echo -e "    ${BLUE}Checking subnet registration with parent gateway...${NC}"

    # Get parent gateway configuration
    local parent_config=$(get_subnet_config "$parent_id")
    local parent_gateway_addr=$(echo "$parent_config" | cut -d'|' -f1)

    # Check if this subnet is using the correct parent gateway
    if [[ "$gateway_addr" != "$parent_gateway_addr" ]]; then
        echo -e "    ${RED}✗ CONFIGURATION ERROR: Subnet is using wrong gateway${NC}"
        echo -e "      ${YELLOW}Subnet gateway:  $gateway_addr${NC}"
        echo -e "      ${YELLOW}Parent gateway:  $parent_gateway_addr${NC}"
        echo -e "    ${CYAN}💡 ARCHITECTURE ISSUE: Child subnets should register with parent's gateway!${NC}"
        echo -e "      ${CYAN}Current config has subnet using its own gateway instead of parent's gateway.${NC}"
        echo -e "      ${CYAN}This suggests UI deployment created new gateway instead of using parent's gateway.${NC}"
        echo -e "    ${CYAN}💡 To fix: Update config to use parent gateway and re-register:${NC}"
        echo -e "      ${CYAN}1. Update config: Change gateway_addr to $parent_gateway_addr${NC}"
        echo -e "      ${CYAN}2. ipc-cli subnet approve --subnet $subnet_id --from <OWNER_ADDRESS>${NC}"
        echo -e "      ${CYAN}3. ipc-cli subnet join --subnet $subnet_id --collateral <AMOUNT> --from <VALIDATOR_ADDRESS>${NC}"
        return 1
    fi

    # Check if subnet is registered with the correct parent gateway
    local gateway_total=$(cast call "$parent_gateway_addr" "totalSubnets()" --rpc-url "$rpc" 2>/dev/null || echo "0x0")
    local total_count=$((gateway_total))

    if [[ $total_count -gt 0 ]]; then
        echo -e "    ${GREEN}✓ Parent gateway has $total_count subnet(s) registered${NC}"

        # Try to verify this specific subnet is registered
        local subnet_eth_addr=$(f4_to_eth "$subnet_id")
        if [[ "$subnet_eth_addr" != "N/A-"* ]]; then
            # Try to get subnet info to verify it's actually registered
            local subnet_info=$(cast call "$parent_gateway_addr" "getSubnet(address)" "$subnet_eth_addr" --rpc-url "$rpc" 2>/dev/null)
            if [[ $? -eq 0 && -n "$subnet_info" && "$subnet_info" != "0x" ]]; then
                echo -e "      ${GREEN}✓ This specific subnet is confirmed registered with parent${NC}"
            else
                echo -e "      ${YELLOW}⚠ Parent gateway has subnets but couldn't verify this specific one${NC}"
            fi
        fi
        return 0
    fi

    echo -e "    ${RED}✗ Subnet is NOT registered with parent gateway${NC}"
    echo -e "    ${CYAN}💡 To fix: Run these commands:${NC}"
    echo -e "      ${CYAN}ipc-cli subnet approve --subnet $subnet_id --from <OWNER_ADDRESS>${NC}"
    echo -e "      ${CYAN}ipc-cli subnet join --subnet $subnet_id --collateral <AMOUNT> --from <VALIDATOR_ADDRESS>${NC}"
    return 1
}

# Check validator configuration using multiple methods
check_validators() {
    local subnet_id="$1"
    local gateway_addr="$2"
    local registry_addr="$3"
    local rpc="$4"

    echo -e "    ${BLUE}Checking validator configuration...${NC}"

    local found_validators=false
    local total_validators=0

    # Method 1: Check using ipc-cli subnet list-validators
    echo -e "      ${CYAN}Method 1: IPC CLI list-validators${NC}"
    local validators_output
    local validators_exit_code

    validators_output=$("$IPC_CLI" subnet list-validators --subnet "$subnet_id" 2>&1)
    validators_exit_code=$?

    if [[ $validators_exit_code -eq 0 && -n "$validators_output" ]]; then
        local clean_output=$(echo "$validators_output" | grep -v "INFO\|WARN\|ERROR" | grep -v "^$")
        if [[ -n "$clean_output" ]]; then
            local validator_count=$(echo "$clean_output" | wc -l | tr -d ' ')
            echo -e "        ${GREEN}✓ Found $validator_count validator(s) via IPC CLI${NC}"
            echo "$clean_output" | sed 's/^/          /'
            found_validators=true
            total_validators=$((total_validators + validator_count))
        else
            echo -e "        ${YELLOW}⚠ IPC CLI returned empty validator list${NC}"
        fi
    else
        echo -e "        ${YELLOW}⚠ IPC CLI list-validators failed or returned no data${NC}"
    fi

    # Method 2: Check registry contract for validators (if it's a federated subnet)
    if [[ -n "$registry_addr" && "$registry_addr" != "<not set>" ]]; then
        echo -e "      ${CYAN}Method 2: Registry contract validators${NC}"
        check_registry_validators "$registry_addr" "$rpc"
        if [[ $? -eq 0 ]]; then
            found_validators=true
        fi
    fi

    # Method 3: Check gateway contract for subnet validators
    if [[ -n "$gateway_addr" && "$gateway_addr" != "<not set>" ]]; then
        echo -e "      ${CYAN}Method 3: Gateway contract validators${NC}"
        check_gateway_validators "$subnet_id" "$gateway_addr" "$rpc"
        if [[ $? -eq 0 ]]; then
            found_validators=true
        fi
    fi

    # Method 4: Check subnet actor contract for validator info
    local subnet_eth_addr=$(f4_to_eth "$subnet_id")
    if [[ "$subnet_eth_addr" != "N/A-"* ]]; then
        echo -e "      ${CYAN}Method 4: Subnet actor contract${NC}"
        check_subnet_actor_validators "$subnet_eth_addr" "$rpc"
        if [[ $? -eq 0 ]]; then
            found_validators=true
        fi
    fi

    if [[ "$found_validators" == "true" ]]; then
        echo -e "    ${GREEN}✓ Validators found using one or more methods${NC}"
        return 0
    else
        echo -e "    ${RED}✗ No validators found using any method${NC}"
        echo -e "    ${CYAN}💡 To fix: Run this command to join as a validator:${NC}"
        echo -e "      ${CYAN}ipc-cli subnet join --subnet $subnet_id --collateral <AMOUNT> --from <VALIDATOR_ADDRESS>${NC}"
        return 1
    fi
}

# Check validators in registry contract (for federated subnets)
check_registry_validators() {
    local registry_addr="$1"
    local rpc="$2"

    # Try to get validator count from registry
    local validator_count=$(cast call "$registry_addr" "validatorCount()" --rpc-url "$rpc" 2>/dev/null)
    if [[ $? -eq 0 && -n "$validator_count" ]]; then
        local count=$((validator_count))
        if [[ $count -gt 0 ]]; then
            echo -e "        ${GREEN}✓ Registry shows $count validator(s)${NC}"

            # Try to get individual validator addresses
            for ((i=0; i<count; i++)); do
                local validator_addr=$(cast call "$registry_addr" "validators(uint256)" "$i" --rpc-url "$rpc" 2>/dev/null)
                if [[ $? -eq 0 && -n "$validator_addr" ]]; then
                    echo -e "          [$i] $validator_addr"
                fi
            done
            return 0
        fi
    fi

    # Try alternative method: getValidators() function
    local validators_list=$(cast call "$registry_addr" "getValidators()" --rpc-url "$rpc" 2>/dev/null)
    if [[ $? -eq 0 && -n "$validators_list" && "$validators_list" != "0x" ]]; then
        echo -e "        ${GREEN}✓ Registry has validators (raw data)${NC}"
        echo -e "          Raw: $validators_list"
        return 0
    fi

    echo -e "        ${YELLOW}⚠ No validators found in registry contract${NC}"
    return 1
}

# Check validators via gateway contract
check_gateway_validators() {
    local subnet_id="$1"
    local gateway_addr="$2"
    local rpc="$3"

    # Get subnet actor address first
    local subnet_eth_addr=$(f4_to_eth "$subnet_id")
    if [[ "$subnet_eth_addr" == "N/A-"* ]]; then
        echo -e "        ${YELLOW}⚠ Cannot check gateway validators (invalid subnet address)${NC}"
        return 1
    fi

    # Try to get subnet info from gateway
    local subnet_info=$(cast call "$gateway_addr" "getSubnet(address)" "$subnet_eth_addr" --rpc-url "$rpc" 2>/dev/null)
    if [[ $? -eq 0 && -n "$subnet_info" && "$subnet_info" != "0x" ]]; then
        echo -e "        ${GREEN}✓ Gateway has subnet info${NC}"
        echo -e "          Info: $subnet_info"
        return 0
    fi

    echo -e "        ${YELLOW}⚠ No subnet info found in gateway${NC}"
    return 1
}

# Check validators in subnet actor contract
check_subnet_actor_validators() {
    local subnet_actor_addr="$1"
    local rpc="$2"

    # Try multiple validator-related functions
    local functions=("validatorCount()" "getValidators()" "validators(uint256)" "totalValidators()")

    for func in "${functions[@]}"; do
        local result=$(cast call "$subnet_actor_addr" "$func" 0 --rpc-url "$rpc" 2>/dev/null)
        if [[ $? -eq 0 && -n "$result" && "$result" != "0x" && "$result" != "0x0000000000000000000000000000000000000000000000000000000000000000" ]]; then
            echo -e "        ${GREEN}✓ Subnet actor responds to $func${NC}"
            echo -e "          Result: $result"
            return 0
        fi
    done

    echo -e "        ${YELLOW}⚠ No validator info found in subnet actor contract${NC}"
    return 1
}

# Check gateway total subnets
check_gateway_subnets() {
    local gateway_addr="$1"
    local rpc="$2"

    echo -e "    ${BLUE}Checking gateway total subnets...${NC}"

    local total_subnets=$(cast call "$gateway_addr" "totalSubnets()" --rpc-url "$rpc" 2>/dev/null || echo "error")

    if [[ "$total_subnets" != "error" ]]; then
        local count=$((total_subnets))
        echo -e "    ${GREEN}✓ Gateway has $count total subnet(s) registered${NC}"
        return 0
    else
        echo -e "    ${RED}✗ Failed to query gateway totalSubnets()${NC}"
        return 1
    fi
}

# Check subnet genesis epoch
check_genesis_epoch() {
    local subnet_id="$1"

    echo -e "    ${BLUE}Checking subnet genesis epoch...${NC}"

    # Use a more robust error handling approach
    local epoch_output
    local epoch_exit_code

    epoch_output=$("$IPC_CLI" subnet genesis-epoch --subnet "$subnet_id" 2>&1)
    epoch_exit_code=$?

    if [[ $epoch_exit_code -eq 0 && -n "$epoch_output" && "$epoch_output" != "error" ]]; then
        # Clean up the output by taking only the first line (ignore log messages)
        local clean_epoch=$(echo "$epoch_output" | head -1)
        echo -e "    ${GREEN}✓ Genesis epoch: $clean_epoch${NC}"
        return 0
    else
        echo -e "    ${RED}✗ Failed to get genesis epoch (subnet may not be operational)${NC}"
        echo -e "    ${CYAN}💡 To fix: Ensure the subnet node is running and properly configured:${NC}"
        echo -e "      ${CYAN}ipc-cli node init --subnet-id $subnet_id --parent-registry <REGISTRY_ADDR> --parent-gateway <GATEWAY_ADDR>${NC}"
        echo -e "      ${CYAN}ipc-cli node run --subnet-id $subnet_id${NC}"
        return 1
    fi
}

# Get parent subnet ID
get_parent_subnet() {
    local subnet_id="$1"

    # Extract parent by removing the last component
    echo "$subnet_id" | sed 's|/[^/]*$||'
}

# Validate a single subnet
validate_subnet() {
    local subnet_id="$1"
    local success=0
    local total=0

    echo ""
    echo -e "${YELLOW}=== Validating Subnet: $subnet_id ===${NC}"

    # Get configuration
    local config=$(get_subnet_config "$subnet_id")
    local gateway_addr=$(echo "$config" | cut -d'|' -f1)
    local registry_addr=$(echo "$config" | cut -d'|' -f2)
    local rpc_url=$(echo "$config" | cut -d'|' -f3)

    # Use provided RPC URL or default
    local effective_rpc="${rpc_url:-$RPC_URL}"

    echo -e "  ${BLUE}Configuration:${NC}"
    echo -e "    Gateway: ${gateway_addr:-<not set>}"
    echo -e "    Registry: ${registry_addr:-<not set>}"
    echo -e "    RPC URL: ${effective_rpc}"

    # Check 1: Gateway contract exists
    echo -e "  ${BLUE}Validation Results:${NC}"
    total=$((total + 1))
    if [[ -n "$gateway_addr" ]]; then
        local gateway_exists=$(check_contract_exists "$gateway_addr" "$effective_rpc")
        if [[ "$gateway_exists" == "true" ]]; then
            echo -e "    ${GREEN}✓ Gateway contract exists${NC}"
            success=$((success + 1))
        else
            echo -e "    ${RED}✗ Gateway contract does not exist${NC}"
        fi
    else
        echo -e "    ${RED}✗ No gateway address configured${NC}"
    fi

    # Check 2: Registry contract exists (if configured)
    if [[ -n "$registry_addr" ]]; then
        total=$((total + 1))
        local registry_exists=$(check_contract_exists "$registry_addr" "$effective_rpc")
        if [[ "$registry_exists" == "true" ]]; then
            echo -e "    ${GREEN}✓ Registry contract exists${NC}"
            success=$((success + 1))
        else
            echo -e "    ${RED}✗ Registry contract does not exist${NC}"
        fi
    fi

    # Check 3: Subnet actor contract exists
    total=$((total + 1))
    local subnet_eth_addr=$(f4_to_eth "$subnet_id")
    if [[ "$subnet_eth_addr" == "N/A-ROOT-SUBNET" ]]; then
        echo -e "    ${YELLOW}⚠ Root subnet - no actor contract to check${NC}"
        success=$((success + 1))  # Count as success for root subnets
    elif [[ "$subnet_eth_addr" =~ ^N/A- ]]; then
        echo -e "    ${RED}✗ Failed to convert subnet ID to Ethereum address: $subnet_eth_addr${NC}"
    elif [[ -n "$subnet_eth_addr" ]]; then
        local subnet_exists=$(check_contract_exists "$subnet_eth_addr" "$effective_rpc")
        if [[ "$subnet_exists" == "true" ]]; then
            echo -e "    ${GREEN}✓ Subnet actor contract exists at $subnet_eth_addr${NC}"
            success=$((success + 1))
        else
            echo -e "    ${RED}✗ Subnet actor contract does not exist at $subnet_eth_addr${NC}"
        fi
    else
        echo -e "    ${RED}✗ Failed to convert subnet ID to Ethereum address${NC}"
    fi

    # Check 4: Subnet registration with parent
    local parent_id=$(get_parent_subnet "$subnet_id")
    if [[ "$parent_id" != "$subnet_id" && -n "$gateway_addr" ]]; then
        total=$((total + 1))
        if check_subnet_registration "$subnet_id" "$parent_id" "$gateway_addr" "$effective_rpc"; then
            success=$((success + 1))
        fi
    fi

    # Check 5: Gateway total subnets
    if [[ -n "$gateway_addr" ]]; then
        total=$((total + 1))
        if check_gateway_subnets "$gateway_addr" "$effective_rpc"; then
            success=$((success + 1))
        fi
    fi

    # Check 6: Validator configuration
    total=$((total + 1))
    if check_validators "$subnet_id" "$gateway_addr" "$registry_addr" "$effective_rpc"; then
        success=$((success + 1))
    fi

    # Check 7: Genesis epoch
    total=$((total + 1))
    if check_genesis_epoch "$subnet_id"; then
        success=$((success + 1))
    fi

    # Summary for this subnet
    local percentage=$((success * 100 / total))
    echo -e "  ${BLUE}Summary: ${success}/${total} checks passed (${percentage}%)${NC}"

    if [[ $success -eq $total ]]; then
        echo -e "  ${GREEN}✓ Subnet appears to be fully operational${NC}"
    elif [[ $success -gt $((total / 2)) ]]; then
        echo -e "  ${YELLOW}⚠ Subnet has some issues but may be partially operational${NC}"
    else
        echo -e "  ${RED}✗ Subnet has significant issues${NC}"
    fi

    return $((total - success))
}

# Main function
main() {
    echo -e "${BLUE}IPC Subnet Deployment Validation${NC}"
    echo -e "${BLUE}=================================${NC}"

    check_dependencies
    parse_config

    local total_issues=0

    for subnet in "${subnets[@]}"; do
        validate_subnet "$subnet"
        total_issues=$((total_issues + $?))
    done

    echo ""
    echo -e "${BLUE}=== Overall Summary ===${NC}"

    if [[ $total_issues -eq 0 ]]; then
        echo -e "${GREEN}✓ All subnets passed validation!${NC}"
        exit 0
    else
        echo -e "${RED}✗ Found $total_issues total issues across all subnets${NC}"
        echo -e "${YELLOW}Please review the validation results above and fix any issues${NC}"
        echo -e "${YELLOW}See docs/troubleshooting-subnet-deployment.md for detailed troubleshooting steps${NC}"
        exit 1
    fi
}

# Parse command line options
while [[ $# -gt 0 ]]; do
    case $1 in
        --rpc-url)
            RPC_URL="$2"
            shift 2
            ;;
        --config)
            CONFIG_FILE="$2"
            shift 2
            ;;
        --ipc-cli)
            IPC_CLI="$2"
            shift 2
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --rpc-url URL    RPC URL to use (default: http://localhost:8545)"
            echo "  --config FILE    Path to IPC config file (default: ~/.ipc/config.toml)"
            echo "  --ipc-cli PATH   Path to IPC CLI binary (default: ./target/release/ipc-cli)"
            echo "  -h, --help       Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

main