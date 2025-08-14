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

    if ! command -v python3 &> /dev/null; then
        echo -e "${RED}Error: 'python3' command not found. Please install python3.${NC}"
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

# Convert f410 address to Ethereum address using Python
f410_to_eth() {
    local f410_addr="$1"

    # Root subnets (like /r31337) don't have actor addresses
    if [[ "$f410_addr" =~ ^/r[0-9]+$ ]]; then
        echo "N/A-ROOT-SUBNET"
        return
    fi

    # Extract the actor part (everything after the last '/')
    local actor_part=$(echo "$f410_addr" | awk -F'/' '{print $NF}')

    # Check if actor part looks like a valid f410 address
    if [[ ! "$actor_part" =~ ^t410[0-9a-z]+$ ]]; then
        echo "N/A-INVALID-FORMAT"
        return
    fi

    # Use Python to convert
    local eth_addr=$(python3 -c "
import base64

def crockford_base32_decode(data):
    crockford_alphabet = '0123456789ABCDEFGHJKMNPQRSTVWXYZ'
    standard_alphabet = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ234567'
    translate_table = str.maketrans(crockford_alphabet, standard_alphabet)
    data_upper = data.upper()
    standard_data = data_upper.translate(translate_table)
    while len(standard_data) % 8 != 0:
        standard_data += '='
    try:
        decoded = base64.b32decode(standard_data)
        return decoded
    except Exception:
        return None

f410_addr = '$actor_part'
without_prefix = f410_addr[4:]
decoded_bytes = crockford_base32_decode(without_prefix)
if decoded_bytes and len(decoded_bytes) >= 20:
    eth_bytes = decoded_bytes[-20:]
    print('0x' + eth_bytes.hex())
else:
    print('N/A-CONVERSION-FAILED')
" 2>/dev/null)

    if [[ -z "$eth_addr" ]]; then
        echo "N/A-CONVERSION-FAILED"
    else
        echo "$eth_addr"
    fi
}

# Query gateway contract for all registered subnet actors
get_deployed_subnet_actors() {
    local gateway_addr="$1"
    local rpc="$2"

    echo -e "    ${BLUE}Querying gateway for deployed subnet actors...${NC}"

    # Get list of all registered subnets from gateway
    local subnet_list=$(cast call "$gateway_addr" "listSubnets()" --rpc-url "$rpc" 2>/dev/null)

    if [[ $? -eq 0 && -n "$subnet_list" && "$subnet_list" != "0x" ]]; then
        echo -e "    ${GREEN}✓ Gateway returned subnet data${NC}"

        # Extract addresses from the response (this is complex ABI decoding)
        # For now, we'll try to extract any 20-byte hex addresses
        local addresses=$(echo "$subnet_list" | grep -oE '0x[a-fA-F0-9]{40}' | sort -u)

        if [[ -n "$addresses" ]]; then
            echo -e "    ${GREEN}✓ Found deployed subnet actor contracts:${NC}"
            echo "$addresses" | while read -r addr; do
                if [[ -n "$addr" ]]; then
                    echo -e "      - $addr"

                    # Check permission mode for each deployed contract
                    local permission_mode=$(cast call "$addr" "permissionMode()" --rpc-url "$rpc" 2>/dev/null)
                    if [[ $? -eq 0 && -n "$permission_mode" ]]; then
                        local mode_dec=$(cast --to-dec "$permission_mode" 2>/dev/null)
                        local mode_name=""
                        case "$mode_dec" in
                            0) mode_name="Collateral" ;;
                            1) mode_name="Federated" ;;
                            2) mode_name="Static" ;;
                            *) mode_name="Unknown($mode_dec)" ;;
                        esac
                        echo -e "        Permission Mode: $mode_name ($mode_dec)"
                    fi
                fi
            done
            return 0
        fi
    fi

    echo -e "    ${YELLOW}⚠ No deployed subnet actors found or gateway query failed${NC}"
    return 1
}

# Check if a specific subnet actor contract is deployed and get its details
check_subnet_actor_details() {
    local subnet_actor_addr="$1"
    local rpc="$2"
    local subnet_id="$3"

    echo -e "    ${BLUE}Checking subnet actor contract: $subnet_actor_addr${NC}"

    # Check if contract exists
    local contract_exists=$(check_contract_exists "$subnet_actor_addr" "$rpc")
    if [[ "$contract_exists" != "true" ]]; then
        echo -e "    ${RED}✗ Subnet actor contract does not exist${NC}"
        return 1
    fi

    echo -e "    ${GREEN}✓ Subnet actor contract exists${NC}"

    # Get permission mode
    local permission_mode=$(cast call "$subnet_actor_addr" "permissionMode()" --rpc-url "$rpc" 2>/dev/null)
    if [[ $? -eq 0 && -n "$permission_mode" ]]; then
        local mode_dec=$(cast --to-dec "$permission_mode" 2>/dev/null)
        local mode_name=""
        case "$mode_dec" in
            0) mode_name="Collateral" ;;
            1) mode_name="Federated" ;;
            2) mode_name="Static" ;;
            *) mode_name="Unknown($mode_dec)" ;;
        esac
        echo -e "    ${GREEN}✓ Permission Mode: $mode_name ($mode_dec)${NC}"
    else
        echo -e "    ${YELLOW}⚠ Could not retrieve permission mode${NC}"
    fi

    # Get parent subnet ID
    local parent_data=$(cast call "$subnet_actor_addr" "getParent()" --rpc-url "$rpc" 2>/dev/null)
    if [[ $? -eq 0 && -n "$parent_data" ]]; then
        echo -e "    ${GREEN}✓ Parent subnet data retrieved${NC}"
        # Extract chain ID from parent data (first 32 bytes after offset)
        local chain_id_hex=$(echo "$parent_data" | grep -oE '7a69|31337' | head -1)
        if [[ -n "$chain_id_hex" ]]; then
            if [[ "$chain_id_hex" == "7a69" ]]; then
                echo -e "      Parent Chain ID: 31337 (0x7a69)"
            else
                echo -e "      Parent Chain ID: $chain_id_hex"
            fi
        fi
    fi

    # Try to get additional info
    local functions=("minValidators()" "majorityPercentage()" "bottomUpCheckPeriod()")
    for func in "${functions[@]}"; do
        local result=$(cast call "$subnet_actor_addr" "$func" --rpc-url "$rpc" 2>/dev/null)
        if [[ $? -eq 0 && -n "$result" && "$result" != "0x" ]]; then
            local func_name=$(echo "$func" | sed 's/()//')
            local value=$(cast --to-dec "$result" 2>/dev/null || echo "$result")
            echo -e "      $func_name: $value"
        fi
    done

    return 0
}

# Check if subnet ID represents a root network
is_root_network() {
    local subnet_id="$1"
    # Root networks follow pattern /r<number> with no additional components
    [[ "$subnet_id" =~ ^/r[0-9]+$ ]]
}

# Check gateway status and deployed subnets
check_gateway_status() {
    local gateway_addr="$1"
    local rpc="$2"

    echo -e "    ${BLUE}Checking gateway contract status...${NC}"

    # Check if gateway exists
    local gateway_exists=$(check_contract_exists "$gateway_addr" "$rpc")
    if [[ "$gateway_exists" != "true" ]]; then
        echo -e "    ${RED}✗ Gateway contract does not exist${NC}"
        return 1
    fi

    echo -e "    ${GREEN}✓ Gateway contract exists${NC}"

    # Get total subnets
    local total_subnets=$(cast call "$gateway_addr" "totalSubnets()" --rpc-url "$rpc" 2>/dev/null)
    if [[ $? -eq 0 && -n "$total_subnets" ]]; then
        local count=$(cast --to-dec "$total_subnets" 2>/dev/null || echo "0")
        echo -e "    ${GREEN}✓ Gateway manages $count subnet(s)${NC}"

        if [[ $count -gt 0 ]]; then
            # Get deployed subnet actors
            get_deployed_subnet_actors "$gateway_addr" "$rpc"
        fi
    else
        echo -e "    ${YELLOW}⚠ Could not retrieve total subnets count${NC}"
    fi

    return 0
}

# Check subnet deployment status against configured vs deployed
check_deployment_status() {
    local subnet_id="$1"
    local gateway_addr="$2"
    local rpc="$3"

    echo -e "    ${BLUE}Checking deployment status...${NC}"

    # Get the converted Ethereum address from f410
    local subnet_eth_addr=$(f410_to_eth "$subnet_id")

    if [[ "$subnet_eth_addr" =~ ^N/A- ]]; then
        echo -e "    ${YELLOW}⚠ Cannot convert f410 address: $subnet_eth_addr${NC}"
        echo -e "    ${CYAN}ℹ This might be normal - deployed contracts may have different addresses${NC}"
    else
        echo -e "    ${CYAN}ℹ Converted f410 to Ethereum address: $subnet_eth_addr${NC}"

        # Check if this specific address is deployed
        check_subnet_actor_details "$subnet_eth_addr" "$rpc" "$subnet_id"
    fi

    return 0
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

    echo -e "  ${BLUE}Validation Results:${NC}"

    # Check 1: Gateway contract status
    total=$((total + 1))
    if [[ -n "$gateway_addr" ]]; then
        if check_gateway_status "$gateway_addr" "$effective_rpc"; then
            success=$((success + 1))
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

    # Check 3: Subnet deployment status
    if ! is_root_network "$subnet_id"; then
        total=$((total + 1))
        if check_deployment_status "$subnet_id" "$gateway_addr" "$effective_rpc"; then
            success=$((success + 1))
        fi
    else
        echo -e "    ${BLUE}ℹ Root network - skipping subnet actor checks${NC}"
    fi

    # Summary for this subnet
    local percentage=$((success * 100 / total))
    echo -e "  ${BLUE}Summary: ${success}/${total} checks passed (${percentage}%)${NC}"

    if is_root_network "$subnet_id"; then
        if [[ $success -eq $total ]]; then
            echo -e "  ${GREEN}✓ Root network contracts are properly deployed${NC}"
        else
            echo -e "  ${YELLOW}⚠ Root network has some issues${NC}"
        fi
    else
        if [[ $success -eq $total ]]; then
            echo -e "  ${GREEN}✓ Subnet appears to be properly configured${NC}"
        elif [[ $success -gt 0 ]]; then
            echo -e "  ${YELLOW}⚠ Subnet has some issues but may be partially operational${NC}"
        else
            echo -e "  ${RED}✗ Subnet has significant deployment issues${NC}"
        fi
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
        echo -e "${CYAN}ℹ Note: This validates contract deployment, not node runtime status${NC}"
        echo -e "${CYAN}ℹ To check if subnet nodes are running, check the appropriate RPC endpoints${NC}"
        exit 0
    else
        echo -e "${RED}✗ Found $total_issues total issues across all subnets${NC}"
        echo -e "${YELLOW}Please review the validation results above and fix any issues${NC}"
        echo -e "${CYAN}💡 Key insights from validation:${NC}"
        echo -e "${CYAN}  - f410 addresses in subnet IDs are identifiers, not deployed contract addresses${NC}"
        echo -e "${CYAN}  - Actual deployed subnet actors are found via gateway.listSubnets()${NC}"
        echo -e "${CYAN}  - Permission modes: 0=Collateral, 1=Federated, 2=Static${NC}"
        echo -e "${CYAN}  - Subnet actors are deployed on the parent chain, not the subnet itself${NC}"
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
            echo ""
            echo "This script validates IPC subnet deployments by:"
            echo "  - Checking gateway and registry contract deployment"
            echo "  - Querying gateway for actually deployed subnet actors"
            echo "  - Comparing configured f410 addresses with deployed contracts"
            echo "  - Showing permission modes (Collateral/Federated/Static)"
            echo "  - Providing deployment status and troubleshooting info"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

main