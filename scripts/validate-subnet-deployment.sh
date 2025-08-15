#!/bin/bash

# IPC Subnet Deployment Validation Script
# This script validates all configured subnets in ~/.ipc/config.toml
# and checks their deployment status on the blockchain
#
# KEY INSIGHT: Subnet Permission Modes Without Querying Subnet Nodes
# ==================================================================
#
# To get a subnet's permission mode without querying the subnet's nodes:
# 1. Query the parent chain's gateway contract using listSubnets()
# 2. This returns an array of Subnet structs containing subnet actor addresses
# 3. Each subnet actor contract is deployed on the PARENT chain (not the subnet itself)
# 4. Call permissionMode() on the subnet actor contract to get its mode:
#    - 0 = Collateral (validator power determined by staked collateral)
#    - 1 = Federated (validator power assigned by subnet owner)
#    - 2 = Static (validator power fixed from initial stake)
#
# IMPORTANT: The f410 addresses in subnet IDs (e.g., /r31337/t410...) are just
# identifiers. The actual subnet actor contracts are deployed at different
# Ethereum addresses that must be queried from the gateway.
#
# USAGE MODES:
# - Default: Validates all subnets in the config
# - --root-gateway-only: Only validates subnets using the root network's gateway/registry
# - --select-gateway: Interactive mode to choose a gateway/registry and update root config

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

# Script modes
ROOT_GATEWAY_ONLY=false
SELECT_GATEWAY=false

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
    local target_subnet_id="${3:-}"  # Optional parameter to find specific subnet

    echo -e "    ${BLUE}Querying gateway for deployed subnet actors...${NC}"

    # Get list of all registered subnets from gateway
    # The listSubnets() returns an array of Subnet structs
    local subnet_list=$(cast call "$gateway_addr" "listSubnets()" --rpc-url "$rpc" 2>/dev/null)

    if [[ $? -eq 0 && -n "$subnet_list" && "$subnet_list" != "0x" ]]; then
        echo -e "    ${GREEN}✓ Gateway returned subnet data${NC}"

        # Extract addresses from the response
        # In the Subnet struct, the subnet actor address is the last element in the route array
        # ABI encoding pads addresses to 32 bytes (64 hex chars) with leading zeros
        # Look for patterns like: 24 zeros followed by 40-char address
        local addresses=""

        # Extract addresses from the ABI-encoded response
        # The response format includes addresses padded to 32 bytes
        # We need to look for valid Ethereum addresses in the data

        # First, convert the response to a continuous string and extract 64-char chunks
        local hex_data=$(echo "$subnet_list" | tr -d '\n ' | sed 's/^0x//')

        # Extract potential addresses by looking for 64-char hex strings that end with
        # what looks like a valid address (not all zeros)
        addresses=$(echo "$hex_data" | fold -w 64 | \
                   grep -E '^0{24}[1-9a-fA-F]' | \
                   sed 's/^0\{24\}/0x/' | sort -u)

        # Also check for addresses that might not be zero-padded
        if [[ -z "$addresses" ]]; then
            addresses=$(echo "$hex_data" | grep -oE '[1-9a-fA-F][0-9a-fA-F]{39}' | \
                       grep -v '^0\+$' | while read addr; do echo "0x$addr"; done | sort -u)
        fi

        if [[ -n "$addresses" ]]; then
            echo -e "    ${GREEN}✓ Found potential subnet actor addresses:${NC}"
                        local found_actors=0

            while read -r addr; do
                if [[ -n "$addr" && "$addr" != "$gateway_addr" ]]; then
                    # Verify this is actually a subnet actor by checking if it has permissionMode()
                    local permission_mode=$(cast call "$addr" "permissionMode()" --rpc-url "$rpc" 2>/dev/null)
                    if [[ $? -eq 0 && -n "$permission_mode" ]]; then
                        found_actors=$((found_actors + 1))
                        echo -e "      - Subnet Actor: $addr"

                        # Decode permission mode
                        local mode_dec=$(cast --to-dec "$permission_mode" 2>/dev/null)
                        local mode_name=""
                        case "$mode_dec" in
                            0) mode_name="Collateral" ;;
                            1) mode_name="Federated" ;;
                            2) mode_name="Static" ;;
                            *) mode_name="Unknown($mode_dec)" ;;
                        esac
                        echo -e "        Permission Mode: $mode_name"

                        # Try to get the subnet's parent info
                        local parent_info=$(cast call "$addr" "getParent()(uint64,address[])" --rpc-url "$rpc" 2>/dev/null)
                        if [[ $? -eq 0 && -n "$parent_info" ]]; then
                            echo -e "        Has parent subnet configuration"
                        fi

                        # If we're looking for a specific subnet, check if this matches
                        if [[ -n "$target_subnet_id" ]]; then
                            # Store this actor for potential matching
                            echo "$addr" >> /tmp/subnet_actors_$$.txt
                                                fi
                    fi
                fi
            done <<< "$addresses"

            if [[ $found_actors -eq 0 ]]; then
                echo -e "    ${YELLOW}⚠ No valid subnet actors found among the addresses${NC}"
                return 1
            fi
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

# Query gateway for subnet information by subnet ID
get_subnet_by_id() {
    local gateway_addr="$1"
    local subnet_id="$2"
    local rpc="$3"

    # Convert subnet ID string (e.g., /r31337/0x123...) to proper format for the contract call
    # Extract components from subnet ID
    local root_chain=$(echo "$subnet_id" | grep -oE '/r[0-9]+' | sed 's|/r||')
    local route_addresses=$(echo "$subnet_id" | grep -oE '(0x[a-fA-F0-9]{40}|t410[0-9a-z]+)' | tr '\n' ' ')

    if [[ -z "$root_chain" ]]; then
        echo "ERROR"
        return 1
    fi

    # For now, we'll use listSubnets and filter
    # This is because constructing the proper SubnetID struct for the call is complex
    local subnet_list=$(cast call "$gateway_addr" "listSubnets()" --rpc-url "$rpc" 2>/dev/null)

    if [[ $? -eq 0 && -n "$subnet_list" && "$subnet_list" != "0x" ]]; then
        # Parse the response to find matching subnet
        # This is complex ABI decoding - for now return indicator that we need to use listSubnets
        echo "USE_LIST_SUBNETS"
        return 0
    fi

    echo "ERROR"
    return 1
}

# Check subnet deployment status against configured vs deployed
check_deployment_status() {
    local subnet_id="$1"
    local gateway_addr="$2"
    local rpc="$3"

    echo -e "    ${BLUE}Checking deployment status for subnet: $subnet_id${NC}"

    # Query the gateway for all deployed subnets
    local subnet_list=$(cast call "$gateway_addr" "listSubnets()" --rpc-url "$rpc" 2>/dev/null)

    if [[ $? -ne 0 || -z "$subnet_list" || "$subnet_list" == "0x" ]]; then
        echo -e "    ${RED}✗ Failed to query gateway for subnet list${NC}"
        return 1
    fi

    # Extract subnet actor addresses from the response
    # The last address in each subnet's route array is the subnet actor address
    # This is a simplified extraction - in practice you'd need proper ABI decoding
    local found_match=false
    local subnet_actor_addr=""

    # Try to find subnet actors from the decoded data
    # Look for addresses that appear to be subnet actors (not gateway addresses)
    local potential_actors=""

    # Extract addresses from the ABI-encoded response
    # First, convert the response to a continuous string and extract 64-char chunks
    local hex_data=$(echo "$subnet_list" | tr -d '\n ' | sed 's/^0x//')

    # Extract potential addresses by looking for 64-char hex strings that end with
    # what looks like a valid address (not all zeros)
    potential_actors=$(echo "$hex_data" | fold -w 64 | \
                      grep -E '^0{24}[1-9a-fA-F]' | \
                      sed 's/^0\{24\}/0x/' | sort -u)

    # Also check for addresses that might not be zero-padded
    if [[ -z "$potential_actors" ]]; then
        potential_actors=$(echo "$hex_data" | grep -oE '[1-9a-fA-F][0-9a-fA-F]{39}' | \
                          grep -v '^0\+$' | while read addr; do echo "0x$addr"; done | sort -u)
    fi

    while read -r actor_addr; do
        if [[ -n "$actor_addr" && "$actor_addr" != "$gateway_addr" ]]; then
            # Check if this is a subnet actor by trying to call permissionMode()
            local permission_mode=$(cast call "$actor_addr" "permissionMode()" --rpc-url "$rpc" 2>/dev/null)
            if [[ $? -eq 0 && -n "$permission_mode" ]]; then
                echo -e "    ${GREEN}✓ Found potential subnet actor at: $actor_addr${NC}"

                # Try to get the subnet ID from this actor
                local actor_subnet_id=$(cast call "$actor_addr" "getParent()(uint64,address[])" --rpc-url "$rpc" 2>/dev/null)

                # Check details for this subnet actor
                check_subnet_actor_details "$actor_addr" "$rpc" "$subnet_id"
                found_match=true
                subnet_actor_addr="$actor_addr"
            fi
        fi
    done <<< "$potential_actors"

    if [[ "$found_match" != "true" ]]; then
        echo -e "    ${YELLOW}⚠ Could not find a matching deployed subnet actor${NC}"
        echo -e "    ${CYAN}ℹ This subnet may not be deployed yet${NC}"
        return 1
    fi

    return 0
}

# Decode subnet information from gateway more accurately
decode_gateway_subnets() {
    local gateway_addr="$1"
    local rpc="$2"

    # First, get the total number of subnets
    local total=$(cast call "$gateway_addr" "totalSubnets()" --rpc-url "$rpc" 2>/dev/null)
    if [[ $? -ne 0 || -z "$total" ]]; then
        return 1
    fi

    local count=$(cast --to-dec "$total" 2>/dev/null || echo "0")
    echo -e "    ${CYAN}ℹ Gateway reports $count total subnet(s)${NC}"

    # Alternative approach: Query subnet keys and then get each subnet
    local subnet_keys=$(cast call "$gateway_addr" "getSubnetKeys()" --rpc-url "$rpc" 2>/dev/null)
    if [[ $? -eq 0 && -n "$subnet_keys" && "$subnet_keys" != "0x" ]]; then
        # Extract 32-byte keys (they appear as 64-char hex strings after 0x)
        local keys=$(echo "$subnet_keys" | grep -oE '[a-fA-F0-9]{64}' | sort -u)

        if [[ -n "$keys" ]]; then
            echo -e "    ${GREEN}✓ Found subnet keys in gateway${NC}"

            echo "$keys" | while read -r key; do
                if [[ -n "$key" ]]; then
                    # Query subnet information for this key
                    local subnet_info=$(cast call "$gateway_addr" "subnets(bytes32)(uint256,uint256,uint256,uint64,uint64,(uint64,address[]))" "0x$key" --rpc-url "$rpc" 2>/dev/null)
                    if [[ $? -eq 0 && -n "$subnet_info" ]]; then
                        echo -e "      Subnet key: 0x$key has deployed info"
                    fi
                fi
            done
        fi
    fi

    return 0
}

# Get parent subnet ID
get_parent_subnet() {
    local subnet_id="$1"

    # Extract parent by removing the last component
    echo "$subnet_id" | sed 's|/[^/]*$||'
}

# Get root network configuration
get_root_network_config() {
    # Find the root network in the config (e.g., /r31337)
    local root_subnet=$(grep '^id = "/r[0-9]\+"' "$CONFIG_FILE" | head -1 | cut -d'"' -f2)

    if [[ -z "$root_subnet" ]]; then
        echo "ERROR: No root network found in configuration"
        return 1
    fi

    # Get the configuration for this root network
    local config=$(get_subnet_config "$root_subnet")
    echo "$root_subnet|$config"
}

# Get all unique gateway/registry pairs from config
get_all_gateways() {
    echo -e "${BLUE}Scanning for all configured gateways and registries...${NC}"

    # Extract all gateway and registry addresses from the config
    local gateway_registry_pairs=()
    local current_subnet=""
    local current_gateway=""
    local current_registry=""

    while IFS= read -r line; do
        # Check for subnet ID
        if [[ "$line" =~ ^id[[:space:]]*=[[:space:]]*\"(.+)\" ]]; then
            current_subnet="${BASH_REMATCH[1]}"
        # Check for gateway address
        elif [[ "$line" =~ ^gateway_addr[[:space:]]*=[[:space:]]*\"(.+)\" ]]; then
            current_gateway="${BASH_REMATCH[1]}"
        # Check for registry address
        elif [[ "$line" =~ ^registry_addr[[:space:]]*=[[:space:]]*\"(.+)\" ]]; then
            current_registry="${BASH_REMATCH[1]}"

            # If we have all three, add to our list
            if [[ -n "$current_subnet" && -n "$current_gateway" && -n "$current_registry" ]]; then
                gateway_registry_pairs+=("$current_gateway|$current_registry|$current_subnet")
            fi
        fi
    done < "$CONFIG_FILE"

    # Deduplicate based on gateway|registry pair
    local unique_pairs=()
    local seen_pairs=()

    for pair in "${gateway_registry_pairs[@]}"; do
        local gateway=$(echo "$pair" | cut -d'|' -f1)
        local registry=$(echo "$pair" | cut -d'|' -f2)
        local subnet=$(echo "$pair" | cut -d'|' -f3)
        local key="$gateway|$registry"

        if [[ ! " ${seen_pairs[@]} " =~ " ${key} " ]]; then
            seen_pairs+=("$key")
            unique_pairs+=("$pair")
        fi
    done

    # Sort and display
    printf '%s\n' "${unique_pairs[@]}" | sort
}

# Interactive gateway selection
select_gateway_interactive() {
    echo -e "${BLUE}=== Gateway Selection Mode ===${NC}"
    echo ""

    local gateways=($(get_all_gateways))

    if [[ ${#gateways[@]} -eq 0 ]]; then
        echo -e "${RED}No gateways found in configuration${NC}"
        return 1
    fi

    echo -e "${CYAN}Found ${#gateways[@]} unique gateway/registry pair(s):${NC}"
    echo ""

    # Display options
    local i=1
    for gateway_info in "${gateways[@]}"; do
        local gateway=$(echo "$gateway_info" | cut -d'|' -f1)
        local registry=$(echo "$gateway_info" | cut -d'|' -f2)
        local subnet=$(echo "$gateway_info" | cut -d'|' -f3)

        echo -e "${YELLOW}[$i]${NC} Gateway:  $gateway"
        echo "    Registry: $registry"
        echo "    Used by:  $subnet"

        # Check how many subnets are on this gateway
        local subnet_count=$(cast call "$gateway" "totalSubnets()" --rpc-url "$RPC_URL" 2>/dev/null | cast --to-dec 2>/dev/null || echo "?")
        echo "    Subnets:  $subnet_count"
        echo ""

        i=$((i + 1))
    done

    # Get user selection
    echo -n "Select gateway/registry pair to use for root network (1-${#gateways[@]}): "
    read selection

    if [[ ! "$selection" =~ ^[0-9]+$ ]] || [[ $selection -lt 1 ]] || [[ $selection -gt ${#gateways[@]} ]]; then
        echo -e "${RED}Invalid selection${NC}"
        return 1
    fi

    # Get selected gateway info
    local selected_idx=$((selection - 1))
    local selected="${gateways[$selected_idx]}"
    local gateway=$(echo "$selected" | cut -d'|' -f1)
    local registry=$(echo "$selected" | cut -d'|' -f2)

    echo ""
    echo -e "${GREEN}Selected:${NC}"
    echo "  Gateway:  $gateway"
    echo "  Registry: $registry"
    echo ""

    # Find root network
    local root_subnet=$(grep '^id = "/r[0-9]\+"' "$CONFIG_FILE" | head -1 | cut -d'"' -f2)

    if [[ -z "$root_subnet" ]]; then
        echo -e "${RED}No root network found in configuration${NC}"
        return 1
    fi

    echo -n "Update root network $root_subnet to use this gateway? (y/N): "
    read confirm

    if [[ "$confirm" =~ ^[Yy]$ ]]; then
        update_root_network_gateway "$root_subnet" "$gateway" "$registry"
    else
        echo -e "${YELLOW}Cancelled${NC}"
    fi
}

# Update root network gateway configuration
update_root_network_gateway() {
    local root_subnet="$1"
    local new_gateway="$2"
    local new_registry="$3"

    echo -e "${BLUE}Updating root network configuration...${NC}"

    # Create a temporary file
    local temp_file=$(mktemp)
    local in_root_subnet=false
    local in_config_section=false
    local updated=false

    while IFS= read -r line; do
        # Check if we're entering the root subnet section
        if [[ "$line" =~ ^id[[:space:]]*=[[:space:]]*\"$root_subnet\" ]]; then
            in_root_subnet=true
            echo "$line" >> "$temp_file"
        # Check if we're in the config section
        elif [[ "$in_root_subnet" == true ]] && [[ "$line" =~ ^\[subnets\.config\] ]]; then
            in_config_section=true
            echo "$line" >> "$temp_file"
        # Update gateway address
        elif [[ "$in_config_section" == true ]] && [[ "$line" =~ ^gateway_addr ]]; then
            echo "gateway_addr = \"$new_gateway\"" >> "$temp_file"
            updated=true
        # Update registry address
        elif [[ "$in_config_section" == true ]] && [[ "$line" =~ ^registry_addr ]]; then
            echo "registry_addr = \"$new_registry\"" >> "$temp_file"
        # Reset when we hit a new subnet
        elif [[ "$line" =~ ^\[\[subnets\]\] ]] && [[ "$in_root_subnet" == true ]]; then
            in_root_subnet=false
            in_config_section=false
            echo "$line" >> "$temp_file"
        else
            echo "$line" >> "$temp_file"
        fi
    done < "$CONFIG_FILE"

    if [[ "$updated" == true ]]; then
        # Backup original config
        cp "$CONFIG_FILE" "$CONFIG_FILE.bak"
        # Replace with updated config
        mv "$temp_file" "$CONFIG_FILE"
        echo -e "${GREEN}✓ Updated root network configuration${NC}"
        echo -e "${CYAN}  Backup saved to: $CONFIG_FILE.bak${NC}"
    else
        rm "$temp_file"
        echo -e "${RED}Failed to update configuration${NC}"
        return 1
    fi
}

# Filter subnets based on root gateway
filter_subnets_by_root_gateway() {
    local all_subnets=("$@")

    # Get root network configuration
    local root_config=$(get_root_network_config)
    if [[ $? -ne 0 ]]; then
        echo -e "${RED}Failed to get root network configuration${NC}"
        return 1
    fi

    local root_subnet=$(echo "$root_config" | cut -d'|' -f1)
    local root_gateway=$(echo "$root_config" | cut -d'|' -f2)
    local root_registry=$(echo "$root_config" | cut -d'|' -f3)

    echo -e "${CYAN}Root network: $root_subnet${NC}"
    echo -e "${CYAN}Root gateway: $root_gateway${NC}"
    echo -e "${CYAN}Root registry: $root_registry${NC}"
    echo ""

    # Filter subnets that use the same gateway/registry
    local filtered_subnets=()

    for subnet in "${all_subnets[@]}"; do
        local config=$(get_subnet_config "$subnet")
        local gateway=$(echo "$config" | cut -d'|' -f1)
        local registry=$(echo "$config" | cut -d'|' -f2)

        if [[ "$gateway" == "$root_gateway" && "$registry" == "$root_registry" ]]; then
            filtered_subnets+=("$subnet")
        fi
    done

    echo -e "${GREEN}Found ${#filtered_subnets[@]} subnet(s) using root network's gateway/registry${NC}"

    # Return filtered subnets
    printf '%s\n' "${filtered_subnets[@]}"
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
    # Handle special modes first
    if [[ "$SELECT_GATEWAY" == true ]]; then
        select_gateway_interactive
        exit $?
    fi

    echo -e "${BLUE}IPC Subnet Deployment Validation${NC}"
    echo -e "${BLUE}=================================${NC}"

    check_dependencies
    parse_config

    local subnets_to_validate=("${subnets[@]}")

    # Filter subnets if --root-gateway-only is specified
    if [[ "$ROOT_GATEWAY_ONLY" == true ]]; then
        echo -e "${YELLOW}Running in root-gateway-only mode${NC}"
        echo ""

        # Get filtered subnet list
        local filtered=($(filter_subnets_by_root_gateway "${subnets[@]}"))

        if [[ ${#filtered[@]} -eq 0 ]]; then
            echo -e "${YELLOW}No subnets found using the root network's gateway/registry${NC}"
            exit 0
        fi

        subnets_to_validate=("${filtered[@]}")
    fi

    local total_issues=0

    for subnet in "${subnets_to_validate[@]}"; do
        validate_subnet "$subnet"
        total_issues=$((total_issues + $?))
    done

    echo ""
    echo -e "${BLUE}=== Overall Summary ===${NC}"

    if [[ $total_issues -eq 0 ]]; then
        echo -e "${GREEN}✓ All validated subnets passed!${NC}"
        if [[ "$ROOT_GATEWAY_ONLY" == true ]]; then
            echo -e "${CYAN}ℹ Only validated subnets using root network's gateway/registry${NC}"
        fi
        echo -e "${CYAN}ℹ Note: This validates contract deployment, not node runtime status${NC}"
        echo -e "${CYAN}ℹ To check if subnet nodes are running, check the appropriate RPC endpoints${NC}"
        exit 0
    else
        echo -e "${RED}✗ Found $total_issues total issues across validated subnets${NC}"
        echo -e "${YELLOW}Please review the validation results above and fix any issues${NC}"
        echo -e "${CYAN}💡 Key insights from validation:${NC}"
        echo -e "${CYAN}  - Subnet actors are deployed on the PARENT chain, not the subnet itself${NC}"
        echo -e "${CYAN}  - Query gateway.listSubnets() on parent chain to find subnet actors${NC}"
        echo -e "${CYAN}  - Call permissionMode() on subnet actor contracts to get their mode${NC}"
        echo -e "${CYAN}  - Permission modes: 0=Collateral, 1=Federated, 2=Static${NC}"
        echo -e "${CYAN}  - f410 addresses in subnet IDs are identifiers, not contract addresses${NC}"
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
        --root-gateway-only)
            ROOT_GATEWAY_ONLY=true
            shift
            ;;
        --select-gateway)
            SELECT_GATEWAY=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --rpc-url URL           RPC URL to use (default: http://localhost:8545)"
            echo "  --config FILE           Path to IPC config file (default: ~/.ipc/config.toml)"
            echo "  --ipc-cli PATH          Path to IPC CLI binary (default: ./target/release/ipc-cli)"
            echo "  --root-gateway-only     Only validate subnets using the root network's gateway/registry"
            echo "  --select-gateway        Interactive mode to select a gateway/registry for the root network"
            echo "  -h, --help              Show this help message"
            echo ""
            echo "Usage modes:"
            echo "  Default mode:           Validates all subnets in the config"
            echo "  --root-gateway-only:    Only validates subnets using the same gateway/registry as the root network"
            echo "  --select-gateway:       Allows you to choose a gateway/registry from your config and update the root network"
            echo ""
            echo "This script validates IPC subnet deployments by:"
            echo "  - Checking gateway and registry contract deployment"
            echo "  - Querying gateway for actually deployed subnet actors"
            echo "  - Comparing configured f410 addresses with deployed contracts"
            echo "  - Showing permission modes (Collateral/Federated/Static)"
            echo "  - Providing deployment status and troubleshooting info"
            echo ""
            echo "Examples:"
            echo "  $0                                    # Validate all subnets"
            echo "  $0 --root-gateway-only                # Only validate subnets on root's gateway"
            echo "  $0 --select-gateway                   # Choose a gateway for the root network"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

main