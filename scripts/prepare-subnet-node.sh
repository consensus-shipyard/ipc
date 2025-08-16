#!/bin/bash

# IPC Subnet Node Preparation Script
# This script helps prepare and configure validator nodes for deployed subnets.
# It discovers validators, generates configuration, and prepares everything needed for node startup.

set -e  # Exit on any error
set -o pipefail  # Exit on pipe failures

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Configuration defaults
CONFIG_FILE="$HOME/.ipc/config.toml"
IPC_CLI="${IPC_CLI:-ipc-cli}"
DEFAULT_NODE_HOME="$HOME/.ipc-node"
DEFAULT_RPC_URL="http://localhost:8545"

# Global variables
SUBNET_ID=""
VALIDATOR_ADDRESS=""
VALIDATOR_PRIVATE_KEY=""
NODE_HOME=""
RPC_URL=""
INTERACTIVE_MODE=true
GATEWAY_ADDR=""
REGISTRY_ADDR=""
PARENT_RPC_URL=""

# Display script header
print_header() {
    echo -e "${BLUE}${BOLD}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "                     IPC Subnet Node Preparation Script"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "${NC}"
    echo -e "${CYAN}This script helps you prepare validator nodes for deployed IPC subnets.${NC}"
    echo -e "${CYAN}It will discover validators, generate configuration, and prepare for node startup.${NC}"
    echo ""
}

# Display help information
show_help() {
    cat << EOF
Usage: $0 [OPTIONS]

DESCRIPTION:
    Prepares a validator node for a deployed IPC subnet. This script automates the process
    of discovering validators, generating configuration files, and preparing everything
    needed to start a validator node.

OPTIONS:
    --subnet-id SUBNET_ID       The subnet ID (e.g., /r31337/t410f...)
    --validator-key KEY_FILE    Path to validator private key file (optional if key is in wallet)
    --validator-address ADDR    Specific validator address to prepare
    --node-home PATH           Node home directory (default: ~/.ipc-node)
    --rpc-url URL              Parent RPC URL (default: http://localhost:8545)
    --non-interactive          Run in non-interactive mode (requires all params)
    --config CONFIG_FILE       IPC config file (default: ~/.ipc/config.toml)
    --ipc-cli PATH             Path to IPC CLI binary (default: ipc-cli in PATH)
    -h, --help                 Show this help message

INTERACTIVE MODE:
    By default, the script runs interactively and will prompt for missing parameters.
    You can provide some parameters via command line and be prompted for the rest.

NON-INTERACTIVE MODE:
    Use --non-interactive to run without prompts. All required parameters must be
    provided via command line arguments.

EXAMPLES:
    # Interactive mode (will prompt for missing info)
    $0 --subnet-id /r31337/t410fuwdv5xidf356o5zqqsxchrmi3lbehpabqz46huq

    # Non-interactive mode (all parameters provided)
    $0 --subnet-id /r31337/t410fuwdv5xidf356o5zqqsxchrmi3lbehpabqz46huq \\
       --validator-key ~/.ipc/validator.sk \\
       --node-home ~/.my-node \\
       --non-interactive

    # Prepare specific validator
    $0 --subnet-id /r31337/t410fuwdv5xidf356o5zqqsxchrmi3lbehpabqz46huq \\
       --validator-address 0x742d35Cc6634C0532925a3b8d0da4a5191c6eD3

WHAT THIS SCRIPT DOES:
    1. Validates dependencies and configuration
    2. Discovers validators configured for the subnet
    3. Allows selection of which validator to prepare (if multiple)
    4. Automatically exports private key from wallet or asks for key file
    5. Generates node initialization configuration file
    6. Runs 'ipc-cli node init' to prepare the node
    7. Provides instructions for starting the node

NEXT STEPS AFTER RUNNING:
    After this script completes successfully, you can start your node with:

    ipc-cli node start --home <node-home-directory>

EOF
}

# Check if required tools are available
check_dependencies() {
    echo -e "${BLUE}Checking dependencies...${NC}"

    local missing_deps=()

    if ! command -v cast &> /dev/null; then
        missing_deps+=("cast (foundry)")
    fi

    if ! command -v jq &> /dev/null; then
        missing_deps+=("jq")
    fi

    if ! command -v python3 &> /dev/null; then
        missing_deps+=("python3")
    fi

    if ! command -v "$IPC_CLI" &> /dev/null; then
        missing_deps+=("ipc-cli (not found in PATH or at specified location)")
    fi

    if [[ ${#missing_deps[@]} -gt 0 ]]; then
        echo -e "${RED}Error: Missing required dependencies:${NC}"
        for dep in "${missing_deps[@]}"; do
            echo -e "${RED}  - $dep${NC}"
        done
        echo ""
        echo -e "${YELLOW}Please install missing dependencies:${NC}"
        echo -e "${YELLOW}  - Foundry: curl -L https://foundry.paradigm.xyz | bash${NC}"
        echo -e "${YELLOW}  - jq: apt-get install jq (or brew install jq on macOS)${NC}"
        echo -e "${YELLOW}  - python3: Usually pre-installed${NC}"
        echo -e "${YELLOW}  - ipc-cli: make build-with-ui (in IPC project root) and add to PATH${NC}"
        exit 1
    fi

    if [[ ! -f "$CONFIG_FILE" ]]; then
        echo -e "${YELLOW}Warning: IPC config file not found at $CONFIG_FILE${NC}"
        echo -e "${YELLOW}This is okay if you're providing all parameters manually.${NC}"
    fi

    echo -e "${GREEN}✓ All dependencies found${NC}"
}

# Parse command line arguments
parse_arguments() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --subnet-id)
                SUBNET_ID="$2"
                shift 2
                ;;
            --validator-key)
                VALIDATOR_PRIVATE_KEY="$2"
                shift 2
                ;;
            --validator-address)
                VALIDATOR_ADDRESS="$2"
                shift 2
                ;;
            --node-home)
                NODE_HOME="$2"
                shift 2
                ;;
            --rpc-url)
                RPC_URL="$2"
                shift 2
                ;;
            --non-interactive)
                INTERACTIVE_MODE=false
                shift
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
                show_help
                exit 0
                ;;
            *)
                echo -e "${RED}Unknown option: $1${NC}"
                echo -e "${YELLOW}Use --help for usage information${NC}"
                exit 1
                ;;
        esac
    done
}

# Get subnet configuration from IPC config file
get_subnet_config() {
    local subnet_id="$1"

    if [[ ! -f "$CONFIG_FILE" ]]; then
        return 1
    fi

    # Use awk to parse the TOML file and extract subnet configuration
    local config_data=$(awk -v subnet="$subnet_id" '
        /^\[\[subnets\]\]/ {
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

    if [[ -n "$config_data" ]]; then
        GATEWAY_ADDR=$(echo "$config_data" | cut -d'|' -f1)
        REGISTRY_ADDR=$(echo "$config_data" | cut -d'|' -f2)
        PARENT_RPC_URL=$(echo "$config_data" | cut -d'|' -f3)
        return 0
    fi

    return 1
}

# Convert f410 address to Ethereum address
f410_to_eth() {
    local f410_addr="$1"

    # Root subnets don't have actor addresses
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

# Query gateway for validators and subnet information
get_subnet_validators() {
    local gateway_addr="$1"
    local rpc="$2"

    echo -e "${BLUE}Querying subnet validators from gateway...${NC}"

    # First, try to get the subnet actor address from the subnet ID
    local subnet_actor_addr=""
    if [[ -n "$SUBNET_ID" ]]; then
        echo -e "${BLUE}Converting subnet ID to Ethereum address...${NC}"
        subnet_actor_addr=$(f410_to_eth "$SUBNET_ID" 2>/dev/null || echo "")

        if [[ -n "$subnet_actor_addr" && "$subnet_actor_addr" != N/A-* ]]; then
            echo -e "${CYAN}Converted subnet ID to Ethereum address: $subnet_actor_addr${NC}"

            # Check if this subnet actor exists and get its validators
            echo -e "${BLUE}Checking if subnet actor contract exists...${NC}"
            local contract_exists=$(check_contract_exists "$subnet_actor_addr" "$rpc" 2>/dev/null || echo "false")

            if [[ "$contract_exists" == "true" ]]; then
                echo -e "${GREEN}✓ Found subnet actor contract at: $subnet_actor_addr${NC}"

                                # Try to get active validators from the subnet actor
                echo -e "${BLUE}Querying active validators from subnet actor...${NC}"
                local active_validators=""
                active_validators=$(cast call "$subnet_actor_addr" "getActiveValidators()" --rpc-url "$rpc" 2>/dev/null || echo "")

                if [[ -n "$active_validators" && "$active_validators" != "0x" && "$active_validators" != "0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000" ]]; then
                    # The response is ABI-encoded array of addresses
                    # Use cast to properly decode it
                    local decoded_addresses=""
                    decoded_addresses=$(cast abi-decode "address[]" "$active_validators" 2>/dev/null || echo "")

                    if [[ -n "$decoded_addresses" && "$decoded_addresses" != "[]" ]]; then
                        # Extract addresses from the decoded output - format is like [0x123..., 0x456...]
                        local addresses=$(echo "$decoded_addresses" | grep -oE '0x[a-fA-F0-9]{40}' | sort -u || echo "")
                        if [[ -n "$addresses" ]]; then
                            echo -e "${GREEN}✓ Found validators in subnet actor:${NC}"
                            while read -r addr; do
                                if [[ -n "$addr" ]]; then
                                    echo -e "  ${CYAN}Validator: $addr${NC}"
                                fi
                            done <<< "$addresses"
                            echo "$addresses"
                            return 0
                        fi
                    else
                        echo -e "${YELLOW}⚠ Subnet actor exists but has no active validators yet${NC}"
                    fi
                else
                    echo -e "${YELLOW}⚠ Subnet actor exists but has no active validators yet${NC}"
                fi
            else
                echo -e "${YELLOW}⚠ Subnet actor contract does not exist yet at: $subnet_actor_addr${NC}"
            fi
        else
            echo -e "${YELLOW}⚠ Could not convert subnet ID to Ethereum address${NC}"
        fi
    fi

        # Fallback: Try to get waiting validators if no active validators found
    if [[ -n "$subnet_actor_addr" && "$subnet_actor_addr" != N/A-* ]]; then
        local contract_exists=$(check_contract_exists "$subnet_actor_addr" "$rpc" 2>/dev/null || echo "false")
        if [[ "$contract_exists" == "true" ]]; then
            echo -e "${BLUE}Checking for waiting validators as fallback...${NC}"
            local waiting_validators=""
            waiting_validators=$(cast call "$subnet_actor_addr" "getWaitingValidators()" --rpc-url "$rpc" 2>/dev/null || echo "")

            if [[ -n "$waiting_validators" && "$waiting_validators" != "0x" && "$waiting_validators" != "0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000" ]]; then
                local decoded_addresses=""
                decoded_addresses=$(cast abi-decode "address[]" "$waiting_validators" 2>/dev/null || echo "")

                if [[ -n "$decoded_addresses" && "$decoded_addresses" != "[]" ]]; then
                    local addresses=$(echo "$decoded_addresses" | grep -oE '0x[a-fA-F0-9]{40}' | sort -u || echo "")
                    if [[ -n "$addresses" ]]; then
                        echo -e "${GREEN}✓ Found waiting validators in subnet actor:${NC}"
                        while read -r addr; do
                            if [[ -n "$addr" ]]; then
                                echo -e "  ${CYAN}Waiting Validator: $addr${NC}"
                            fi
                        done <<< "$addresses"
                        echo "$addresses"
                        return 0
                    fi
                fi
            fi
        fi
    fi

    echo -e "${YELLOW}⚠ No validators found via gateway queries${NC}"
    echo -e "${CYAN}This might be normal for a newly deployed subnet that hasn't been bootstrapped yet.${NC}"
    return 1
}

# Interactive prompt for missing parameters
collect_parameters() {
    echo -e "${BLUE}Collecting configuration parameters...${NC}"

    # Subnet ID
    if [[ -z "$SUBNET_ID" ]]; then
        if [[ "$INTERACTIVE_MODE" == "true" ]]; then
            echo ""
            echo -e "${YELLOW}Enter the subnet ID you want to prepare a node for:${NC}"
            echo -e "${CYAN}(Example: /r31337/t410fuwdv5xidf356o5zqqsxchrmi3lbehpabqz46huq)${NC}"
            read -p "Subnet ID: " SUBNET_ID

            if [[ -z "$SUBNET_ID" ]]; then
                echo -e "${RED}Error: Subnet ID is required${NC}"
                exit 1
            fi
        else
            echo -e "${RED}Error: --subnet-id is required in non-interactive mode${NC}"
            exit 1
        fi
    fi

    # Try to get subnet configuration
    echo -e "${BLUE}Looking up subnet configuration...${NC}"
    if get_subnet_config "$SUBNET_ID"; then
        echo -e "${GREEN}✓ Found subnet configuration in $CONFIG_FILE${NC}"
        echo -e "  Gateway: $GATEWAY_ADDR"
        echo -e "  Registry: $REGISTRY_ADDR"
        echo -e "  Parent RPC: $PARENT_RPC_URL"
    else
        echo -e "${YELLOW}⚠ Subnet not found in configuration file${NC}"

        if [[ "$INTERACTIVE_MODE" == "true" ]]; then
            echo -e "${YELLOW}Please provide the gateway address and parent RPC URL manually:${NC}"
            read -p "Gateway address: " GATEWAY_ADDR
            read -p "Parent RPC URL: " PARENT_RPC_URL
        else
            echo -e "${RED}Error: Subnet configuration not found and running in non-interactive mode${NC}"
            exit 1
        fi
    fi

    # Set effective RPC URL
    RPC_URL="${PARENT_RPC_URL:-$DEFAULT_RPC_URL}"

    # Node home directory
    if [[ -z "$NODE_HOME" ]]; then
        if [[ "$INTERACTIVE_MODE" == "true" ]]; then
            echo ""
            echo -e "${YELLOW}Enter the node home directory (will be created if it doesn't exist):${NC}"
            read -p "Node home [$DEFAULT_NODE_HOME]: " NODE_HOME
            NODE_HOME="${NODE_HOME:-$DEFAULT_NODE_HOME}"
        else
            NODE_HOME="$DEFAULT_NODE_HOME"
        fi
    fi

    echo -e "${GREEN}✓ Configuration parameters collected${NC}"
}

# Discover and select validator
discover_validators() {
    echo ""
    echo -e "${BLUE}=== Discovering Validators for Subnet: $SUBNET_ID ===${NC}"

    # Check if this is a root subnet
    if [[ "$SUBNET_ID" =~ ^/r[0-9]+$ ]]; then
        echo -e "${CYAN}This is a root subnet. Root subnets don't have validators in the traditional sense.${NC}"
        echo -e "${CYAN}You may need to manually configure your node for this network.${NC}"
        return 0
    fi

    # Query validators from gateway
    local validators=()
    if [[ -n "$GATEWAY_ADDR" ]]; then
        echo -e "${BLUE}Querying gateway at: $GATEWAY_ADDR${NC}"

        # Use a more compatible way to read command output into array
        local validator_output=""
        # Temporarily disable exit on error for this query
        set +e
        validator_output=$(get_subnet_validators "$GATEWAY_ADDR" "$RPC_URL" 2>&1)
        local query_result=$?
        set -e

        if [[ $query_result -eq 0 && -n "$validator_output" ]]; then
            # Split output into array using newlines
            IFS=$'\n' read -d '' -r -a validators <<< "$validator_output" 2>/dev/null || true
        else
            echo -e "${YELLOW}Gateway query completed with no validators found${NC}"
        fi

        if [[ ${#validators[@]} -eq 0 ]]; then
            echo -e "${YELLOW}No validators found via gateway query.${NC}"
        fi
    else
        echo -e "${YELLOW}No gateway address available for validator discovery${NC}"
    fi

    # If no validators found or user specified a specific address
    if [[ ${#validators[@]} -eq 0 || -n "$VALIDATOR_ADDRESS" ]]; then
        if [[ -n "$VALIDATOR_ADDRESS" ]]; then
            echo -e "${CYAN}Using specified validator address: $VALIDATOR_ADDRESS${NC}"
            validators=("$VALIDATOR_ADDRESS")
        elif [[ "$INTERACTIVE_MODE" == "true" ]]; then
            echo ""
            echo -e "${YELLOW}No validators found via automatic discovery.${NC}"
            echo -e "${CYAN}This is normal for newly deployed subnets that haven't been activated yet.${NC}"
            echo -e "${CYAN}You can provide the validator address manually.${NC}"
            echo ""
            echo -e "${YELLOW}Please enter the validator address you want to prepare:${NC}"
            read -p "Validator address [0x70997970c51812dc3a010c7d01b50e0d17dc79c8]: " VALIDATOR_ADDRESS
            VALIDATOR_ADDRESS="${VALIDATOR_ADDRESS:-0x70997970c51812dc3a010c7d01b50e0d17dc79c8}"

            if [[ -z "$VALIDATOR_ADDRESS" ]]; then
                echo -e "${RED}Error: Validator address is required${NC}"
                exit 1
            fi
            validators=("$VALIDATOR_ADDRESS")
        else
            echo -e "${RED}Error: No validators found and no address specified${NC}"
            echo -e "${YELLOW}Try running with --validator-address to specify the validator manually${NC}"
            exit 1
        fi
    fi

    # If multiple validators found, let user choose
    if [[ ${#validators[@]} -gt 1 && "$INTERACTIVE_MODE" == "true" && -z "$VALIDATOR_ADDRESS" ]]; then
        echo ""
        echo -e "${YELLOW}Multiple validators found for this subnet:${NC}"
        for i in "${!validators[@]}"; do
            echo -e "${CYAN}  $((i+1)). ${validators[i]}${NC}"
        done

        echo ""
        read -p "Select validator (1-${#validators[@]}): " selection

        if [[ "$selection" =~ ^[0-9]+$ ]] && [[ "$selection" -ge 1 ]] && [[ "$selection" -le ${#validators[@]} ]]; then
            VALIDATOR_ADDRESS="${validators[$((selection-1))]}"
        else
            echo -e "${RED}Invalid selection${NC}"
            exit 1
        fi
    else
        VALIDATOR_ADDRESS="${validators[0]}"
    fi

    echo -e "${GREEN}✓ Selected validator: $VALIDATOR_ADDRESS${NC}"
}

# Get validator private key (either from wallet export or file)
get_validator_key() {
    echo ""
    echo -e "${BLUE}=== Getting Validator Private Key ===${NC}"

    # First, try to export the private key from the wallet
    echo -e "${BLUE}Attempting to export private key from wallet...${NC}"

    local exported_key=""
    set +e  # Don't exit on error for this attempt
    exported_key=$("$IPC_CLI" wallet export --wallet-type evm --address "$VALIDATOR_ADDRESS" 2>/dev/null)
    local export_result=$?
    set -e

    if [[ $export_result -eq 0 && -n "$exported_key" ]]; then
        echo -e "${GREEN}✓ Successfully exported private key from wallet${NC}"
        VALIDATOR_PRIVATE_KEY="__EXPORTED__"
        return 0
    else
        echo -e "${YELLOW}⚠ Could not export private key from wallet${NC}"
        echo -e "${CYAN}Will ask for private key file instead...${NC}"
    fi

    # Fall back to asking for private key file
    if [[ -z "$VALIDATOR_PRIVATE_KEY" ]]; then
        if [[ "$INTERACTIVE_MODE" == "true" ]]; then
            echo ""
            echo -e "${YELLOW}Enter the path to the validator private key file:${NC}"
            echo -e "${CYAN}(This should be a secp256k1 private key file for address $VALIDATOR_ADDRESS)${NC}"
            read -p "Private key file path: " VALIDATOR_PRIVATE_KEY

            if [[ -z "$VALIDATOR_PRIVATE_KEY" ]]; then
                echo -e "${RED}Error: Private key file path is required${NC}"
                exit 1
            fi
        else
            echo -e "${RED}Error: Could not export from wallet and --validator-key not provided in non-interactive mode${NC}"
            exit 1
        fi
    fi

    # Verify the key file exists (only if not exported)
    if [[ "$VALIDATOR_PRIVATE_KEY" != "__EXPORTED__" ]]; then
        if [[ ! -f "$VALIDATOR_PRIVATE_KEY" ]]; then
            echo -e "${RED}Error: Private key file does not exist: $VALIDATOR_PRIVATE_KEY${NC}"
            exit 1
        fi
        echo -e "${GREEN}✓ Private key file found: $VALIDATOR_PRIVATE_KEY${NC}"
    fi
}

# Generate node initialization configuration
generate_node_config() {
    echo ""
    echo -e "${BLUE}=== Generating Node Configuration ===${NC}"

    # Create node home directory
    mkdir -p "$NODE_HOME"

    # Determine parent subnet ID
    local parent_id
    if [[ "$SUBNET_ID" =~ ^(/r[0-9]+)/ ]]; then
        parent_id="${BASH_REMATCH[1]}"
    else
        echo -e "${RED}Error: Could not determine parent subnet from $SUBNET_ID${NC}"
        exit 1
    fi

        # Get the private key value (either from exported wallet or file)
    local private_key_value
    if [[ "$VALIDATOR_PRIVATE_KEY" == "__EXPORTED__" ]]; then
        # Re-export the private key to get the actual value and extract just the private key
        local exported_json=$("$IPC_CLI" wallet export --wallet-type evm --address "$VALIDATOR_ADDRESS" 2>/dev/null)
        if [[ -z "$exported_json" ]]; then
            echo -e "${RED}Error: Failed to re-export private key from wallet${NC}"
            exit 1
        fi

        # Extract just the private key from the JSON response
        private_key_value=$(echo "$exported_json" | jq -r '.private_key' 2>/dev/null)
        if [[ -z "$private_key_value" || "$private_key_value" == "null" ]]; then
            echo -e "${RED}Error: Could not extract private key from wallet export${NC}"
            exit 1
        fi

        echo -e "${GREEN}✓ Using private key from wallet export${NC}"
    else
        # Read from file
        if [[ -f "$VALIDATOR_PRIVATE_KEY" ]]; then
            private_key_value=$(cat "$VALIDATOR_PRIVATE_KEY" | tr -d '\n\r')
            echo -e "${GREEN}✓ Read private key from file${NC}"
        else
            echo -e "${RED}Error: Private key file not found: $VALIDATOR_PRIVATE_KEY${NC}"
            exit 1
        fi
    fi

    # Ensure the private key starts with 0x
    if [[ ! "$private_key_value" =~ ^0x ]]; then
        private_key_value="0x$private_key_value"
    fi

    # Generate the node configuration YAML
    local config_file="$NODE_HOME/node-init.yml"

    echo -e "${BLUE}Creating node initialization config at: $config_file${NC}"

    # Check if this is a newly deployed subnet that needs to be joined first
    echo -e "${BLUE}Checking if subnet needs to be joined first...${NC}"
    local needs_join="false"

    # Check if the validator address exists in the subnet
    local subnet_actor_addr=$(f410_to_eth "$SUBNET_ID" 2>/dev/null || echo "")
    if [[ -n "$subnet_actor_addr" && "$subnet_actor_addr" != N/A-* ]]; then
        local contract_exists=$(check_contract_exists "$subnet_actor_addr" "$RPC_URL" 2>/dev/null || echo "false")
        if [[ "$contract_exists" != "true" ]]; then
            needs_join="true"
            echo -e "${YELLOW}⚠ Subnet actor contract not deployed yet${NC}"
        else
            echo -e "${GREEN}✓ Subnet actor contract exists${NC}"
        fi
    else
        needs_join="true"
        echo -e "${YELLOW}⚠ Could not determine subnet actor address${NC}"
    fi

    cat > "$config_file" << EOF
# IPC Node Initialization Configuration
# Generated by prepare-subnet-node.sh

# Home directory for the node
home: "$NODE_HOME"

# Subnet to join
subnet: "$SUBNET_ID"

# Parent subnet
parent: "$parent_id"

# Validator key configuration
key:
  wallet-type: evm
  private-key: "$private_key_value"

# P2P networking configuration
p2p:
  external-ip: "127.0.0.1"
  ports:
    cometbft: 26656
    resolver: 26655

# Genesis configuration - create from parent subnet data
genesis: !create
  base-fee: "1000"
  power-scale: 3
  network-version: 21

# Join subnet configuration (for newly deployed subnets)
# Note: This will be skipped if the subnet is already bootstrapped
join:
  from: "$VALIDATOR_ADDRESS"
  collateral: 1.0
  initial-balance: 10.0

# Optional: CometBFT configuration overrides
# cometbft-overrides: |
#   [consensus]
#   timeout_commit = "5s"
#   [rpc]
#   laddr = "tcp://0.0.0.0:26657"

# Optional: Fendermint configuration overrides
# fendermint-overrides: |
#   [app]
#   max_validators = 100
EOF

    if [[ "$needs_join" == "true" ]]; then
        echo ""
        echo -e "${YELLOW}📋 Important Note:${NC}"
        echo -e "${CYAN}This appears to be a newly deployed subnet that may need additional setup.${NC}"
        echo -e "${CYAN}The node configuration includes a 'join' section to automatically join the subnet.${NC}"
        echo ""
    fi

        echo -e "${GREEN}✓ Node configuration generated${NC}"
    echo -e "${CYAN}Configuration saved to: $config_file${NC}"
    echo -e "${CYAN}You can inspect the configuration file if needed before proceeding.${NC}"
}

# Initialize the node
initialize_node() {
    echo ""
    echo -e "${BLUE}=== Initializing Node ===${NC}"

    local config_file="$NODE_HOME/node-init.yml"

    echo -e "${BLUE}Running: $IPC_CLI node init --config $config_file${NC}"

    # Temporarily disable exit on error to handle potential peer info generation failures gracefully
    set +e
    "$IPC_CLI" node init --config "$config_file"
    local init_result=$?
    set -e

    if [[ $init_result -eq 0 ]]; then
        echo -e "${GREEN}✓ Node initialization completed successfully${NC}"
    else
        # Check if the error was due to subnet already being bootstrapped
        local log_output=$("$IPC_CLI" node init --config "$config_file" 2>&1 || true)
        if echo "$log_output" | grep -q "SubnetAlreadyBootstrapped"; then
            echo -e "${YELLOW}⚠ Subnet is already bootstrapped, skipping join step${NC}"
            echo -e "${CYAN}Continuing with node initialization...${NC}"

            # Try to initialize without the join section by creating a temporary config
            local temp_config="$NODE_HOME/node-init-no-join.yml"
            grep -v -A3 "^join:" "$config_file" | grep -v "from:\|collateral:\|initial-balance:" > "$temp_config"

            set +e
            "$IPC_CLI" node init --config "$temp_config"
            local retry_result=$?
            set -e

            rm -f "$temp_config" 2>/dev/null || true

            if [[ $retry_result -eq 0 ]]; then
                echo -e "${GREEN}✓ Node initialization completed successfully (without join)${NC}"
                return 0
            fi
        fi

        # Check if most of the initialization succeeded despite other failures
        if [[ -f "$NODE_HOME/cometbft/config/genesis.json" && -f "$NODE_HOME/fendermint/config/default.toml" ]]; then
            echo -e "${YELLOW}⚠ Node initialization mostly completed with some warnings${NC}"
            echo -e "${CYAN}The core node setup is ready, though some steps may have failed.${NC}"
            echo -e "${CYAN}This may not prevent the node from starting.${NC}"
        else
            echo -e "${RED}✗ Node initialization failed${NC}"
            exit 1
        fi
    fi
}

# Display final instructions
show_completion() {
    echo ""
    echo -e "${GREEN}${BOLD}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "                          🚀 NODE PREPARATION COMPLETE! 🚀"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "${NC}"

    echo -e "${CYAN}Your validator node has been successfully prepared!${NC}"
    echo ""
    echo -e "${YELLOW}📋 Configuration Summary:${NC}"
    echo -e "   • Subnet ID: ${BOLD}$SUBNET_ID${NC}"
    echo -e "   • Validator Address: ${BOLD}$VALIDATOR_ADDRESS${NC}"
    echo -e "   • Node Home: ${BOLD}$NODE_HOME${NC}"
    echo -e "   • Gateway: ${BOLD}$GATEWAY_ADDR${NC}"
    echo -e "   • Parent RPC: ${BOLD}$RPC_URL${NC}"
    echo ""

    echo -e "${GREEN}${BOLD}🎯 NEXT STEPS:${NC}"
    echo ""

    # Check if this was a newly deployed subnet
    local subnet_actor_addr=$(f410_to_eth "$SUBNET_ID" 2>/dev/null || echo "")
    local is_new_subnet="false"
    if [[ -n "$subnet_actor_addr" && "$subnet_actor_addr" != N/A-* ]]; then
        local contract_exists=$(check_contract_exists "$subnet_actor_addr" "$RPC_URL" 2>/dev/null || echo "false")
        if [[ "$contract_exists" != "true" ]]; then
            is_new_subnet="true"
        fi
    fi

    if [[ "$is_new_subnet" == "true" ]]; then
        echo -e "${YELLOW}📋 For newly deployed subnets:${NC}"
        echo ""
        echo -e "${YELLOW}1. First, join the subnet as a validator:${NC}"
        echo -e "   ${BOLD}$IPC_CLI subnet join --subnet $SUBNET_ID --from $VALIDATOR_ADDRESS --collateral 1${NC}"
        echo ""
        echo -e "${YELLOW}2. Wait for subnet activation (this may take a few minutes)${NC}"
        echo ""
        echo -e "${YELLOW}3. Then start your validator node:${NC}"
        echo -e "   ${BOLD}$IPC_CLI node start --home $NODE_HOME${NC}"
        echo ""
        echo -e "${CYAN}💡 Alternatively, the node init process should have automatically joined the subnet.${NC}"
        echo -e "${CYAN}   You can proceed directly to starting the node if the join was successful.${NC}"
        echo ""
    else
        echo -e "${YELLOW}1. Start your validator node:${NC}"
        echo -e "   ${BOLD}$IPC_CLI node start --home $NODE_HOME${NC}"
        echo ""
    fi

    echo -e "${YELLOW}📊 Monitor your node:${NC}"
    echo -e "   • Check logs in: ${CYAN}$NODE_HOME/fendermint/data/logs/${NC}"
    echo -e "   • CometBFT RPC: ${CYAN}http://localhost:26657${NC}"
    echo -e "   • ETH JSON-RPC: ${CYAN}http://localhost:8545${NC}"
    echo ""
    echo -e "${YELLOW}🔍 Verify your node is running:${NC}"
    echo -e "   ${BOLD}curl -s http://localhost:26657/status | jq '.result.sync_info'${NC}"
    echo ""

    echo -e "${CYAN}📁 Important Files Created:${NC}"
    echo -e "   • Configuration: ${CYAN}$NODE_HOME/node-init.yml${NC}"
    echo -e "   • CometBFT config: ${CYAN}$NODE_HOME/cometbft/config/${NC}"
    echo -e "   • Fendermint config: ${CYAN}$NODE_HOME/fendermint/config/${NC}"
    echo -e "   • Genesis file: ${CYAN}$NODE_HOME/cometbft/config/genesis.json${NC}"
    echo ""

    echo -e "${GREEN}${BOLD}✅ Your node is ready to join the subnet and start validating!${NC}"
    echo ""
}

# Main execution flow
main() {
    print_header

    # Parse command line arguments
    parse_arguments "$@"

    # Check dependencies
    check_dependencies

    # Collect all required parameters
    collect_parameters

    # Discover and select validator
    discover_validators

    # Get validator private key
    get_validator_key

    # Generate node configuration
    generate_node_config

    # Initialize the node
    initialize_node

    # Show completion message
    show_completion
}

# Run the main function with all arguments
main "$@"