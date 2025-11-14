#!/bin/bash
# Configuration parsing and management

# Global variables for peer info
declare -A COMETBFT_PEERS
declare -A LIBP2P_PEERS
declare -A VALIDATOR_PUBKEYS

# Global deployment mode
DEPLOYMENT_MODE=""

# Get deployment mode (local or remote)
get_deployment_mode() {
    # Check CLI override first
    if [ -n "${CLI_MODE:-}" ]; then
        echo "$CLI_MODE"
        return
    fi

    # Check config file
    local mode=$(yq eval '.deployment.mode // "remote"' "$CONFIG_FILE" 2>/dev/null)
    if [ -z "$mode" ] || [ "$mode" = "null" ]; then
        mode="remote"
    fi
    echo "$mode"
}

# Check if running in local mode
is_local_mode() {
    [ "$DEPLOYMENT_MODE" = "local" ]
}

# Get validator port with fallback to default
# Usage: get_validator_port <validator_idx> <port_type> <default_port>
get_validator_port() {
    local validator_idx="$1"
    local port_type="$2"
    local default_port="$3"

    # Try to get validator-specific port override
    local port=$(yq eval ".validators[$validator_idx].ports.$port_type // null" "$CONFIG_FILE" 2>/dev/null)

    if [ "$port" != "null" ] && [ -n "$port" ]; then
        echo "$port"
    else
        echo "$default_port"
    fi
}

# Calculate port offset for a validator (for local mode)
# Validator 0 gets offset 0, validator 1 gets offset 100, etc.
get_validator_port_offset() {
    local validator_idx="$1"
    echo $((validator_idx * 100))
}

# Load and validate configuration
load_config() {
    if [ ! -f "$CONFIG_FILE" ]; then
        log_error "Config file not found: $CONFIG_FILE"
        exit 1
    fi

    # Clear validators array (in case of shell reuse)
    VALIDATORS=()
    COMETBFT_PEERS=()
    LIBP2P_PEERS=()
    VALIDATOR_PUBKEYS=()

    # Determine deployment mode
    DEPLOYMENT_MODE=$(get_deployment_mode)

    # Parse validators
    local validator_count=$(yq eval '.validators | length' "$CONFIG_FILE")
    for ((i=0; i<validator_count; i++)); do
        local name=$(yq eval ".validators[$i].name" "$CONFIG_FILE")
        VALIDATORS+=("$name")
    done

    log_info "Loaded configuration for ${#VALIDATORS[@]} validators (mode: $DEPLOYMENT_MODE)"
}

# Get config value with environment variable override
get_config_value() {
    local key="$1"
    local env_key=$(echo "$key" | tr '[:lower:].' '[:upper:]_' | sed 's/\[/\_/g' | sed 's/\]//g')

    # Check environment variable first
    if [ -n "${!env_key:-}" ]; then
        echo "${!env_key}"
        return
    fi

    # Fall back to config file
    yq eval ".$key" "$CONFIG_FILE"
}

# Get validator index by name
get_validator_index() {
    local name="$1"
    for idx in "${!VALIDATORS[@]}"; do
        if [ "${VALIDATORS[$idx]}" = "$name" ]; then
            echo "$idx"
            return 0
        fi
    done
    return 1
}

# Get primary validator
get_primary_validator() {
    for idx in "${!VALIDATORS[@]}"; do
        local role=$(get_config_value "validators[$idx].role")
        if [ "$role" = "primary" ]; then
            echo "$idx"
            return 0
        fi
    done

    # Default to first validator if no primary specified
    echo "0"
}

# Check configuration validity
check_config_validity() {
    log_info "Validating configuration..."

    local errors=0

    # Check subnet ID
    local subnet_id=$(get_config_value "subnet.id")
    if [ -z "$subnet_id" ] || [ "$subnet_id" = "null" ]; then
        log_error "Subnet ID not configured"
        ((errors++))
    else
        log_check "ok" "Subnet ID: $subnet_id"
    fi

    # Check validators
    if [ ${#VALIDATORS[@]} -eq 0 ]; then
        log_error "No validators configured"
        ((errors++))
    else
        log_check "ok" "Validators: ${#VALIDATORS[@]}"
    fi

    # Check required fields for each validator
    for idx in "${!VALIDATORS[@]}"; do
        local name="${VALIDATORS[$idx]}"
        local ip=$(get_config_value "validators[$idx].ip")

        if [ -z "$ip" ] || [ "$ip" = "null" ]; then
            log_error "Validator $name: IP not configured"
            ((errors++))
        fi
    done

    if [ $errors -gt 0 ]; then
        log_error "Configuration validation failed with $errors errors"
        exit 1
    fi

    log_check "ok" "Configuration valid"
}

# Check requirements (binaries, tools)
check_requirements() {
    log_info "Checking requirements..."

    local missing=0

    # Check yq
    if ! command -v yq &> /dev/null; then
        log_error "yq not found. Install with: brew install yq"
        ((missing++))
    else
        log_check "ok" "yq found"
    fi

    # Check mode-specific requirements
    if is_local_mode; then
        # Local mode: check for anvil and ipc-cli
        if ! command -v anvil &> /dev/null; then
            log_warn "anvil not found. Install Foundry for Anvil support"
            log_info "  curl -L https://foundry.paradigm.xyz | bash && foundryup"
        else
            log_check "ok" "anvil found"
        fi

        if ! command -v ipc-cli &> /dev/null; then
            log_warn "ipc-cli not in PATH. Will use path from config"
        else
            log_check "ok" "ipc-cli found"
        fi
    else
        # Remote mode: check for ssh/scp
    if ! command -v ssh &> /dev/null; then
        log_error "ssh not found"
        ((missing++))
    else
        log_check "ok" "ssh found"
    fi

    if ! command -v scp &> /dev/null; then
        log_error "scp not found"
        ((missing++))
    else
        log_check "ok" "scp found"
        fi
    fi

    if [ $missing -gt 0 ]; then
        log_error "Missing $missing required tools"
        exit 1
    fi
}

# Check SSH connectivity to all validators
check_ssh_connectivity() {
    # Skip SSH checks in local mode
    if is_local_mode; then
        log_info "SSH connectivity check skipped (local mode)"
        return 0
    fi

    if [ "$DRY_RUN" = true ]; then
        log_info "Checking SSH connectivity (skipped in dry-run mode)..."
        for idx in "${!VALIDATORS[@]}"; do
            local name="${VALIDATORS[$idx]}"
            local ip=$(get_config_value "validators[$idx].ip")
            log_check "ok" "$name ($ip) [dry-run]"
        done
        return 0
    fi

    log_info "Checking SSH connectivity..."

    local failures=0

    for idx in "${!VALIDATORS[@]}"; do
        local name="${VALIDATORS[$idx]}"
        local ip=$(get_config_value "validators[$idx].ip")
        local ssh_user=$(get_config_value "validators[$idx].ssh_user")

        if test_ssh "$ip" "$ssh_user"; then
            log_check "ok" "$name ($ip)"
        else
            log_check "fail" "$name ($ip) - SSH connection failed"
            ((failures++))
        fi
    done

    if [ $failures -gt 0 ]; then
        log_error "SSH connectivity check failed for $failures validators"
        log_error "Set up SSH keys with: ssh-copy-id $ssh_user@<validator-ip>"
        exit 1
    fi
}

# Generate node-init.yml for a validator
generate_node_init_yml() {
    local validator_idx="$1"
    local output_file="$2"
    local peer_files="${3:-}"

    # Get config values
    local subnet_id=$(get_config_value "subnet.id")
    local parent_chain_id=$(get_config_value "subnet.parent_chain_id")
    local parent_rpc=$(get_config_value "subnet.parent_rpc")

    # Read parent registry and gateway from IPC CLI config (updated by subnet init)
    local ipc_config_file=$(get_config_value "paths.ipc_config_file")
    ipc_config_file="${ipc_config_file/#\~/$HOME}"

    local parent_registry=$(get_config_value "subnet.parent_registry")
    local parent_gateway=$(get_config_value "subnet.parent_gateway")

    # If IPC config exists, try to read the actual parent addresses from it
    if [ -f "$ipc_config_file" ]; then
        local actual_parent_registry=$(grep -A 10 "id = \"$parent_chain_id\"" "$ipc_config_file" | grep "registry_addr" | head -1 | cut -d'"' -f2)
        local actual_parent_gateway=$(grep -A 10 "id = \"$parent_chain_id\"" "$ipc_config_file" | grep "gateway_addr" | head -1 | cut -d'"' -f2)

        if [ -n "$actual_parent_registry" ]; then
            parent_registry="$actual_parent_registry"
        fi
        if [ -n "$actual_parent_gateway" ]; then
            parent_gateway="$actual_parent_gateway"
        fi
    fi

    local name="${VALIDATORS[$validator_idx]}"
    local ip=$(get_config_value "validators[$validator_idx].ip")
    local private_key=$(get_config_value "validators[$validator_idx].private_key")

    # Get node home (different for local vs remote mode)
    local node_home
    if is_local_mode; then
        node_home=$(get_node_home "$validator_idx")
    else
        node_home=$(get_config_value "paths.node_home")
    fi

    # Expand tilde to absolute path (required by ipc-cli node init)
    node_home="${node_home/#\~/$HOME}"

    # Get port offset for local mode
    local port_offset=0
    if is_local_mode; then
        port_offset=$(get_validator_port_offset "$validator_idx")
    fi

    # Calculate ports with offset
    local cometbft_p2p_port=$(($(get_config_value "network.cometbft_p2p_port") + port_offset))
    local cometbft_rpc_port=$(($(get_config_value "network.cometbft_rpc_port" 2>/dev/null || echo "26657") + port_offset))
    local cometbft_abci_port=$(($(get_config_value "network.cometbft_abci_port" 2>/dev/null || echo "26658") + port_offset))
    local cometbft_prometheus_port=$(($(get_config_value "network.cometbft_prometheus_port" 2>/dev/null || echo "26660") + port_offset))
    local libp2p_port=$(($(get_config_value "network.libp2p_port") + port_offset - 1))  # -1 to match pattern
    local eth_api_port=$(($(get_config_value "network.eth_api_port") + port_offset))
    local eth_metrics_port=$(($(get_config_value "network.eth_metrics_port" 2>/dev/null || echo "9184") + port_offset))
    local fendermint_metrics_port=$(($(get_config_value "network.fendermint_metrics_port" 2>/dev/null || echo "9185") + port_offset))

    # Override with validator-specific ports if provided
    cometbft_p2p_port=$(get_validator_port "$validator_idx" "cometbft_p2p" "$cometbft_p2p_port")
    cometbft_rpc_port=$(get_validator_port "$validator_idx" "cometbft_rpc" "$cometbft_rpc_port")
    cometbft_abci_port=$(get_validator_port "$validator_idx" "cometbft_abci" "$cometbft_abci_port")
    cometbft_prometheus_port=$(get_validator_port "$validator_idx" "cometbft_prometheus" "$cometbft_prometheus_port")
    libp2p_port=$(get_validator_port "$validator_idx" "libp2p" "$libp2p_port")
    eth_api_port=$(get_validator_port "$validator_idx" "eth_api" "$eth_api_port")
    eth_metrics_port=$(get_validator_port "$validator_idx" "eth_metrics" "$eth_metrics_port")
    fendermint_metrics_port=$(get_validator_port "$validator_idx" "fendermint_metrics" "$fendermint_metrics_port")

    # Genesis config
    local base_fee=$(get_config_value "init.genesis.base_fee")
    local power_scale=$(get_config_value "init.genesis.power_scale")
    local network_version=$(get_config_value "init.genesis.network_version")

    # IPC config
    local vote_interval=$(get_config_value "init.ipc.vote_interval")
    local vote_timeout=$(get_config_value "init.ipc.vote_timeout")

    # Topdown config
    local chain_head_delay=$(get_config_value "init.topdown.chain_head_delay")
    local proposal_delay=$(get_config_value "init.topdown.proposal_delay")
    local max_proposal_range=$(get_config_value "init.topdown.max_proposal_range")
    local polling_interval=$(get_config_value "init.topdown.polling_interval")
    local exponential_back_off=$(get_config_value "init.topdown.exponential_back_off")
    local exponential_retry_limit=$(get_config_value "init.topdown.exponential_retry_limit")
    local parent_http_timeout=$(get_config_value "init.topdown.parent_http_timeout")

    # CometBFT config - core timeouts
    local timeout_commit=$(get_config_value "init.cometbft.timeout_commit")
    local timeout_propose=$(get_config_value "init.cometbft.timeout_propose")
    local timeout_prevote=$(get_config_value "init.cometbft.timeout_prevote")
    local timeout_precommit=$(get_config_value "init.cometbft.timeout_precommit")

    # CometBFT config - timeout deltas
    local timeout_propose_delta=$(get_config_value "init.cometbft.timeout_propose_delta")
    local timeout_prevote_delta=$(get_config_value "init.cometbft.timeout_prevote_delta")
    local timeout_precommit_delta=$(get_config_value "init.cometbft.timeout_precommit_delta")

    # CometBFT config - empty blocks
    local create_empty_blocks=$(get_config_value "init.cometbft.create_empty_blocks")
    local create_empty_blocks_interval=$(get_config_value "init.cometbft.create_empty_blocks_interval")

    # CometBFT config - P2P
    local send_rate=$(get_config_value "init.cometbft.send_rate")
    local recv_rate=$(get_config_value "init.cometbft.recv_rate")
    local max_packet_msg_payload_size=$(get_config_value "init.cometbft.max_packet_msg_payload_size")

    # CometBFT config - RPC
    local rpc_laddr=$(get_config_value "init.cometbft.rpc_laddr")

    cat > "$output_file" << EOF
# IPC Node Initialization Configuration
# Generated by ipc-subnet-manager

# Home directory for the node
home: "$node_home"

# Subnet to join
subnet: "$subnet_id"

# Parent subnet
parent: "$parent_chain_id"

# Validator key configuration
key:
  wallet-type: evm
  private-key: "$private_key"

# P2P networking configuration
p2p:
  external-ip: "$ip"
  ports:
    cometbft: $cometbft_p2p_port
    resolver: $libp2p_port
EOF

    # Add peer files if provided, otherwise set peers to null
    if [ -n "$peer_files" ]; then
        cat >> "$output_file" << EOF
  peers:
    peer-files:
      - "$peer_files"
EOF
    else
        cat >> "$output_file" << EOF
  peers: null
EOF
    fi

    # Get current parent chain height for genesis timestamp
    local parent_rpc=$(get_config_value "subnet.parent_rpc")
    local current_parent_height=$(curl -s -X POST -H "Content-Type: application/json" \
        --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
        "$parent_rpc" | jq -r '.result' | xargs printf "%d\n" 2>/dev/null || echo "0")

    log_info "Current parent chain height: $current_parent_height (will be used as genesis timestamp)"

    # Check if genesis files exist (created by ipc-cli subnet create-genesis)
    local ipc_config_dir=$(get_config_value "paths.ipc_config_dir")
    ipc_config_dir="${ipc_config_dir/#\~/$HOME}"
    # ipc-cli subnet create-genesis creates files with format: genesis_r31337_... (removes leading /)
    local subnet_id_no_slash="${subnet_id#/}"
    local genesis_json="$ipc_config_dir/genesis_${subnet_id_no_slash//\//_}.json"
    local genesis_sealed="$ipc_config_dir/genesis_sealed_${subnet_id_no_slash//\//_}.json"

    if [ -f "$genesis_json" ] && [ -f "$genesis_sealed" ]; then
        # Use existing genesis files
        log_info "Found existing genesis files - using !path"
        cat >> "$output_file" << EOF

# Genesis configuration - use existing genesis files
genesis: !path
  genesis: "$genesis_json"
  sealed: "$genesis_sealed"

# Join subnet configuration (for newly deployed subnets)
# Note: This will be skipped if the subnet is already bootstrapped
join: null
EOF
    else
        # Create genesis from parent subnet (requires activated subnet)
        log_info "No genesis files found - using !create (requires activated subnet)"
    cat >> "$output_file" << EOF

# Genesis configuration - create from parent subnet data
genesis: !create
  base-fee: "$base_fee"
  power-scale: $power_scale
  network-version: $network_version
  timestamp: $current_parent_height  # Use current parent height to avoid 16h lookback issue

# Join subnet configuration (for newly deployed subnets)
# Note: This will be skipped if the subnet is already bootstrapped
join: null
EOF
    fi

    cat >> "$output_file" << EOF

# Optional: CometBFT configuration overrides
cometbft-overrides: |
EOF

    # Add local mode port overrides
    if is_local_mode; then
        cat >> "$output_file" << EOF
  proxy_app = "tcp://127.0.0.1:$cometbft_abci_port"
EOF
    fi

    cat >> "$output_file" << EOF
  [consensus]
  # Core consensus timeouts
  timeout_commit = "$timeout_commit"
  timeout_propose = "$timeout_propose"
  timeout_prevote = "$timeout_prevote"
  timeout_precommit = "$timeout_precommit"

  # Timeout deltas (increase per round on failure)
  timeout_propose_delta = "$timeout_propose_delta"
  timeout_prevote_delta = "$timeout_prevote_delta"
  timeout_precommit_delta = "$timeout_precommit_delta"

  # Empty block control
  create_empty_blocks = $create_empty_blocks
  create_empty_blocks_interval = "$create_empty_blocks_interval"

  [p2p]
  # P2P performance tuning
  send_rate = $send_rate
  recv_rate = $recv_rate
  max_packet_msg_payload_size = $max_packet_msg_payload_size

  [rpc]
EOF

    # Set RPC laddr based on mode
    if is_local_mode; then
        cat >> "$output_file" << EOF
  laddr = "tcp://0.0.0.0:$cometbft_rpc_port"

  [instrumentation]
  prometheus_listen_addr = ":$cometbft_prometheus_port"
EOF
    else
        cat >> "$output_file" << EOF
  laddr = "$rpc_laddr"
EOF
    fi

    cat >> "$output_file" << EOF

# Optional: Fendermint configuration overrides
fendermint-overrides: |
EOF

    # Add local mode port overrides for fendermint
    if is_local_mode; then
        cat >> "$output_file" << EOF
  tendermint_rpc_url = "http://127.0.0.1:$cometbft_rpc_port"
  tendermint_websocket_url = "ws://127.0.0.1:$cometbft_rpc_port/websocket"

  [abci.listen]
  port = $cometbft_abci_port

  [eth.listen]
  host = "0.0.0.0"
  port = $eth_api_port

  [eth.metrics.listen]
  port = $eth_metrics_port

  [metrics.listen]
  port = $fendermint_metrics_port

EOF
    fi

    cat >> "$output_file" << EOF
  [resolver]
  enabled = true

  [ipc]
  subnet_id = "$subnet_id"
  vote_interval = $vote_interval
  vote_timeout = $vote_timeout

  [ipc.topdown]
  chain_head_delay = $chain_head_delay
  proposal_delay = $proposal_delay
  max_proposal_range = $max_proposal_range
  polling_interval = $polling_interval
  exponential_back_off = $exponential_back_off
  exponential_retry_limit = $exponential_retry_limit
  parent_http_endpoint = "$parent_rpc"
  parent_http_timeout = $parent_http_timeout
  parent_registry = "$parent_registry"
  parent_gateway = "$parent_gateway"

  [resolver.connection]
EOF

    # Set resolver listen address based on mode
    if is_local_mode; then
        cat >> "$output_file" << EOF
  listen_addr = "/ip4/127.0.0.1/tcp/$libp2p_port"
EOF
    else
        cat >> "$output_file" << EOF
  listen_addr = "/ip4/0.0.0.0/tcp/$libp2p_port"
EOF
    fi

    cat >> "$output_file" << EOF

  [resolver.network]
  local_key = "validator.sk"

  # Disable bottom-up checkpointing for federated subnets
  # (Bottom-up checkpointing posts state commitments to parent chain)
  [ipc.bottomup]
  enabled = false

  [validator_key]
  path = "validator.sk"
  # Use "ethereum" for EVM-based subnets (federated/collateral with EVM addresses)
  # Use "regular" only for native Filecoin address subnets
  kind = "ethereum"
EOF
}

# Extract peer information from a validator
extract_peer_info() {
    local validator_idx="$1"
    local name="${VALIDATORS[$validator_idx]}"

    # Get node home path (local or remote)
    local node_home
    if is_local_mode; then
        local node_home_base=$(get_config_value "paths.node_home_base")
        node_home="${node_home_base/#\~/$HOME}/$name"
    else
        node_home=$(get_config_value "paths.node_home")
    fi

    # Get CometBFT peer info
    local peer_info=$(exec_on_host "$validator_idx" "cat $node_home/peer-info.json 2>/dev/null || echo '{}'")

    if [ -z "$peer_info" ] || [ "$peer_info" = "{}" ]; then
        log_error "Failed to extract peer info from validator $validator_idx"
        return 1
    fi

    echo "$peer_info"
}

# Collect peer IDs from running CometBFT nodes via RPC
collect_peer_ids_from_running_nodes() {
    log_info "Collecting peer IDs from running CometBFT nodes..."

    for idx in "${!VALIDATORS[@]}"; do
        local name="${VALIDATORS[$idx]}"
        local ip=$(get_config_value "validators[$idx].ip")
        local ssh_user=$(get_config_value "validators[$idx].ssh_user")
        local cometbft_port=$(get_config_value "network.cometbft_p2p_port")

        # Query CometBFT RPC for node info (contains node ID)
        local node_id=$(ssh -o StrictHostKeyChecking=no "$ssh_user@$ip" \
            "curl -s http://127.0.0.1:26657/status 2>/dev/null | jq -r '.result.node_info.id // empty'" 2>/dev/null | tr -d '[:space:]')

        if [ -n "$node_id" ] && [ "$node_id" != "" ] && [ "$node_id" != "null" ]; then
            COMETBFT_PEERS[$idx]="${node_id}@${ip}:${cometbft_port}"
            log_info "$name CometBFT: ${COMETBFT_PEERS[$idx]}"
        else
            log_warn "Could not get CometBFT node ID for $name from RPC"
        fi
    done
}

# Collect all peer information
collect_all_peer_info() {
    log_info "Collecting peer information from all validators..."

    for idx in "${!VALIDATORS[@]}"; do
        local name="${VALIDATORS[$idx]}"
        local ip=$(get_config_value "validators[$idx].ip")
        local libp2p_port=$(get_config_value "network.libp2p_port")

        # Get node home path (local or remote)
        local node_home
        if is_local_mode; then
            local node_home_base=$(get_config_value "paths.node_home_base")
            node_home="${node_home_base/#\~/$HOME}/$name"
        else
            node_home=$(get_config_value "paths.node_home")
        fi

        # Get peer info from peer-info.json file for libp2p peer ID
        local peer_json=$(exec_on_host "$idx" "cat $node_home/peer-info.json 2>/dev/null || echo '{}'")

        # Parse libp2p peer ID locally (we'll reconstruct the multiaddr with correct IP)
        local libp2p_peer_id=$(echo "$peer_json" | jq -r '.fendermint.peer_id // empty' 2>/dev/null)

        if [ -n "$libp2p_peer_id" ] && [ "$libp2p_peer_id" != "null" ]; then
            # Reconstruct multiaddr using the ACTUAL public IP from config (not from peer-info.json)
            # This ensures we advertise the correct external IP even if peer-info.json has 127.0.0.1
            LIBP2P_PEERS[$idx]="/ip4/$ip/tcp/$libp2p_port/p2p/$libp2p_peer_id"
            log_info "$name libp2p: ${LIBP2P_PEERS[$idx]}"
        else
            log_warn "Could not get libp2p peer ID for $name"
        fi

        # Get validator public key from validator.pk file
        local pubkey=$(exec_on_host "$idx" \
            "cat $node_home/fendermint/validator.pk 2>/dev/null || echo ''")

        if [ -z "$pubkey" ]; then
            log_warn "Could not get validator public key for $name"
        else
            VALIDATOR_PUBKEYS[$idx]="$pubkey"
            log_info "$name pubkey: ${pubkey:0:20}..."
        fi
    done
}

# Fix listen_addr to bind to 0.0.0.0 (ipc-cli sets it to external-ip)
fix_listen_addresses() {
    log_info "Fixing resolver listen addresses to bind to 0.0.0.0..."

    local libp2p_port=$(get_config_value "network.libp2p_port")

    for idx in "${!VALIDATORS[@]}"; do
        local name="${VALIDATORS[$idx]}"

        # Get node home path (local or remote)
        local node_home
        if is_local_mode; then
            local node_home_base=$(get_config_value "paths.node_home_base")
            node_home="${node_home_base/#\~/$HOME}/$name"
        else
            node_home=$(get_config_value "paths.node_home")
        fi

        log_info "Fixing listen_addr for $name..."

        # Change listen_addr from public IP to 0.0.0.0
        local config_file="$node_home/fendermint/config/default.toml"
        exec_on_host "$idx" "sed -i.bak 's|listen_addr = .*/tcp/$libp2p_port\"|listen_addr = \"/ip4/0.0.0.0/tcp/$libp2p_port\"|' $config_file" >/dev/null 2>&1

        # Verify the change
        local listen_addr=$(exec_on_host "$idx" "grep 'listen_addr = ' $config_file | head -1" 2>/dev/null)

        if echo "$listen_addr" | grep -q "0.0.0.0"; then
            log_info "  ✓ $name now listening on 0.0.0.0:$libp2p_port"
        else
            log_warn "  ✗ Failed to update listen_addr for $name"
        fi
    done
}

# Update validator configs with full peer mesh
update_all_configs() {
    log_info "Configuring peer mesh for ${#VALIDATORS[@]} validators..."

    for idx in "${!VALIDATORS[@]}"; do
        local name="${VALIDATORS[$idx]}"
        log_subsection "$name"

        # Show what will be configured
        if [ -n "${LIBP2P_PEERS[$idx]:-}" ]; then
            log_info "  External address: ${LIBP2P_PEERS[$idx]}"
        fi

        local peer_count=0
        for peer_idx in "${!VALIDATORS[@]}"; do
            if [ "$peer_idx" != "$idx" ] && [ -n "${LIBP2P_PEERS[$peer_idx]:-}" ]; then
                peer_count=$((peer_count + 1))
            fi
        done
        log_info "  Static peers: $peer_count"

        update_validator_config "$idx"
    done
}

# Update single validator config
update_validator_config() {
    local validator_idx="$1"

    local name="${VALIDATORS[$validator_idx]}"
    local libp2p_port=$(get_config_value "network.libp2p_port")

    # Get node home path (local or remote)
    local node_home
    if is_local_mode; then
        local node_home_base=$(get_config_value "paths.node_home_base")
        node_home="${node_home_base/#\~/$HOME}/$name"
    else
        node_home=$(get_config_value "paths.node_home")
    fi

    # Build peer lists (excluding self)
    local comet_peers=""
    local libp2p_static_addrs=""

    for peer_idx in "${!VALIDATORS[@]}"; do
        if [ "$peer_idx" != "$validator_idx" ]; then
            if [ -n "${COMETBFT_PEERS[$peer_idx]:-}" ]; then
                comet_peers+="${COMETBFT_PEERS[$peer_idx]},"
            fi
            if [ -n "${LIBP2P_PEERS[$peer_idx]:-}" ]; then
                # Don't include quotes in variable, add them in sed pattern
                libp2p_static_addrs+="${LIBP2P_PEERS[$peer_idx]}, "
            fi
        fi
    done

    # Remove trailing comma/space
    comet_peers="${comet_peers%,}"
    libp2p_static_addrs="${libp2p_static_addrs%, }"

    # Update CometBFT persistent_peers
    if [ -n "$comet_peers" ]; then
        log_info "Setting CometBFT persistent_peers for $name"
        exec_on_host "$validator_idx" \
            "sed -i.bak 's|^persistent_peers = .*|persistent_peers = \"$comet_peers\"|' $node_home/cometbft/config/config.toml" >/dev/null 2>&1
    fi

    # Update Fendermint libp2p config - static_addresses (peers to connect to)
    if [ -n "$libp2p_static_addrs" ]; then
        log_info "Setting libp2p static_addresses for $name"
        # Add quotes around each multiaddr by transforming "addr1, addr2" to "\"addr1\", \"addr2\""
        local quoted_addrs=$(echo "$libp2p_static_addrs" | sed 's|/ip4/|"/ip4/|g' | sed 's|, |", |g')
        quoted_addrs="${quoted_addrs}\""  # Add trailing quote
        exec_on_host "$validator_idx" \
            "sed -i.bak '/\\[resolver.discovery\\]/,/\\[.*\\]/ s|^static_addresses = .*|static_addresses = [$quoted_addrs]|' $node_home/fendermint/config/default.toml" >/dev/null 2>&1
    fi

    # Update external_addresses (this node's advertised address)
    if [ -n "${LIBP2P_PEERS[$validator_idx]:-}" ]; then
        log_info "Setting libp2p external_addresses for $name"
        exec_on_host "$validator_idx" \
            "sed -i.bak '/\\[resolver.connection\\]/,/\\[.*\\]/ s|^external_addresses = .*|external_addresses = [\"${LIBP2P_PEERS[$validator_idx]}\"]|' $node_home/fendermint/config/default.toml" >/dev/null 2>&1
    fi

    # Ensure validator_key section exists
    exec_on_host "$validator_idx" \
        "grep -q '\\[validator_key\\]' $node_home/fendermint/config/default.toml || echo -e '\\n[validator_key]\\npath = \"validator.sk\"\\nkind = \"regular\"' >> $node_home/fendermint/config/default.toml" >/dev/null 2>&1
}

# Generate IPC CLI config file (~/.ipc/config.toml)
generate_ipc_cli_config() {
    local output_file="$1"

    # Get config values
    local keystore_path=$(get_config_value "ipc_cli.keystore_path")

    # Parent subnet config
    local parent_id=$(get_config_value "ipc_cli.parent.id")
    local parent_network_type=$(get_config_value "ipc_cli.parent.network_type")
    local parent_provider_http=$(get_config_value "ipc_cli.parent.provider_http")
    local parent_registry=$(get_config_value "ipc_cli.parent.registry_addr")
    local parent_gateway=$(get_config_value "ipc_cli.parent.gateway_addr")

    # Child subnet config
    local child_id=$(get_config_value "subnet.id")
    local child_network_type=$(get_config_value "ipc_cli.child.network_type")
    local child_provider_http=$(get_config_value "ipc_cli.child.provider_http")
    local child_gateway=$(get_config_value "ipc_cli.child.gateway_addr")
    local child_registry=$(get_config_value "ipc_cli.child.registry_addr")

    # Generate config - only include parent subnet initially
    # Child subnet will be added by subnet init command
    cat > "$output_file" << EOF
keystore_path = "$keystore_path"

[[subnets]]
id = "$parent_id"

[subnets.config]
network_type = "$parent_network_type"
provider_http = "$parent_provider_http"
registry_addr = "$parent_registry"
gateway_addr = "$parent_gateway"
EOF
}

# Ensure EVM keystore exists with validator keys from config
ensure_evm_keystore() {
    local ipc_config_dir=$(get_config_value "paths.ipc_config_dir")
    ipc_config_dir="${ipc_config_dir/#\~/$HOME}"
    local keystore_file="$ipc_config_dir/evm_keystore.json"

    # Create IPC directory if it doesn't exist
    mkdir -p "$ipc_config_dir"

    # If keystore doesn't exist, create it with validator keys from config
    if [ ! -f "$keystore_file" ]; then
        log_info "Creating EVM keystore with validator keys..."

        echo "[" > "$keystore_file"

        # Add each validator's key
        local first=true
        for idx in "${!VALIDATORS[@]}"; do
            local val_private_key=$(yq eval ".validators[$idx].private_key" "$CONFIG_FILE")
            local val_address=$(yq eval ".validators[$idx].address // null" "$CONFIG_FILE")

            # Derive address if not in config
            if [ "$val_address" = "null" ] || [ -z "$val_address" ]; then
                val_address=$(cast wallet address --private-key "$val_private_key" 2>/dev/null)
            fi

            # Remove 0x prefix from private key for storage
            val_private_key="${val_private_key#0x}"

            # Add comma if not first entry
            if [ "$first" = false ]; then
                echo "," >> "$keystore_file"
            fi
            first=false

            # Add validator entry (note: address keeps 0x prefix)
            cat >> "$keystore_file" << EOF_JSON
  {
    "address": "${val_address}",
    "private_key": "${val_private_key}"
  }
EOF_JSON
        done

        echo "]" >> "$keystore_file"

        log_success "EVM keystore created at $keystore_file"
    else
        log_info "EVM keystore already exists at $keystore_file"
    fi
}

# Update IPC CLI config on all validators
update_ipc_cli_configs() {
    log_info "Updating IPC CLI configuration on all validators..."

    for idx in "${!VALIDATORS[@]}"; do
        local name="${VALIDATORS[$idx]}"
        local ipc_config_dir=$(get_config_value "paths.ipc_config_dir")
        local ipc_config_file=$(get_config_value "paths.ipc_config_file")

        # Expand tilde in paths for local mode
        ipc_config_dir="${ipc_config_dir/#\~/$HOME}"
        ipc_config_file="${ipc_config_file/#\~/$HOME}"

        log_info "Updating IPC CLI config for $name..."

        # Generate config locally
        local temp_config="/tmp/ipc-cli-config-${name}.toml"
        generate_ipc_cli_config "$temp_config"

        # Create directory if it doesn't exist
        exec_on_host "$idx" "mkdir -p $ipc_config_dir"

        # Copy to target location
        copy_to_host "$idx" "$temp_config" "$ipc_config_file"
        rm -f "$temp_config"

        log_success "IPC CLI config updated for $name"
    done
}

