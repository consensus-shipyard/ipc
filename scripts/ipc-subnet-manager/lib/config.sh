#!/bin/bash
# Configuration parsing and management

# Global variables for peer info
declare -A COMETBFT_PEERS
declare -A LIBP2P_PEERS
declare -A VALIDATOR_PUBKEYS

# Load and validate configuration
load_config() {
    if [ ! -f "$CONFIG_FILE" ]; then
        log_error "Config file not found: $CONFIG_FILE"
        exit 1
    fi

    # Parse validators
    local validator_count=$(yq eval '.validators | length' "$CONFIG_FILE")
    for ((i=0; i<validator_count; i++)); do
        local name=$(yq eval ".validators[$i].name" "$CONFIG_FILE")
        VALIDATORS+=("$name")
    done

    log_info "Loaded configuration for ${#VALIDATORS[@]} validators"
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

    # Check ssh
    if ! command -v ssh &> /dev/null; then
        log_error "ssh not found"
        ((missing++))
    else
        log_check "ok" "ssh found"
    fi

    # Check scp
    if ! command -v scp &> /dev/null; then
        log_error "scp not found"
        ((missing++))
    else
        log_check "ok" "scp found"
    fi

    if [ $missing -gt 0 ]; then
        log_error "Missing $missing required tools"
        exit 1
    fi
}

# Check SSH connectivity to all validators
check_ssh_connectivity() {
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
    local peers="${3:-}"

    local subnet_id=$(get_config_value "subnet.id")
    local parent_chain_id=$(get_config_value "subnet.parent_chain_id")
    local parent_rpc=$(get_config_value "subnet.parent_rpc")
    local supply_source_kind=$(get_config_value "init.subnet_supply_source_kind")
    local permission_mode=$(get_config_value "init.permission_mode")

    local name="${VALIDATORS[$validator_idx]}"
    local ip=$(get_config_value "validators[$validator_idx].ip")
    local node_home=$(get_config_value "paths.node_home")
    local cometbft_port=$(get_config_value "network.cometbft_p2p_port")
    local libp2p_port=$(get_config_value "network.libp2p_port")

    cat > "$output_file" << EOF
home_dir: "$node_home"
subnet_id: "$subnet_id"
parent_registry: "$parent_chain_id"
parent_gateway: "$parent_chain_id"

parent:
  rpc:
    http_endpoint: "$parent_rpc"

fendermint_overrides:
  ipc:
    topdown:
      chain_head_delay: 3
      exponential_back_off:
        min: 3
        max: 60
      proposal_delay: 3
      polling_interval: 60
  resolver:
    connection:
      external_addresses:
        - "/ip4/$ip/tcp/$libp2p_port/p2p/LIBP2P_PEER_ID_PLACEHOLDER"
    discovery:
      static_addresses: []
  validator_key:
    path: "validator.sk"
    kind: "regular"
EOF

    # Add peers if provided
    if [ -n "$peers" ]; then
        echo "peers: $peers" >> "$output_file"
    fi
}

# Extract peer information from a validator
extract_peer_info() {
    local validator_idx="$1"

    local ip=$(get_config_value "validators[$validator_idx].ip")
    local ssh_user=$(get_config_value "validators[$validator_idx].ssh_user")
    local ipc_user=$(get_config_value "validators[$validator_idx].ipc_user")
    local node_home=$(get_config_value "paths.node_home")

    # Get CometBFT peer info
    local peer_info=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" "cat $node_home/peer-info.json 2>/dev/null || echo '{}'")

    if [ -z "$peer_info" ] || [ "$peer_info" = "{}" ]; then
        log_error "Failed to extract peer info from validator $validator_idx"
        return 1
    fi

    echo "$peer_info"
}

# Collect all peer information
collect_all_peer_info() {
    log_info "Collecting peer information from all validators..."

    for idx in "${!VALIDATORS[@]}"; do
        local name="${VALIDATORS[$idx]}"
        local ip=$(get_config_value "validators[$idx].ip")
        local ssh_user=$(get_config_value "validators[$idx].ssh_user")
        local ipc_user=$(get_config_value "validators[$idx].ipc_user")
        local node_home=$(get_config_value "paths.node_home")
        local cometbft_port=$(get_config_value "network.cometbft_p2p_port")
        local libp2p_port=$(get_config_value "network.libp2p_port")

        # Get CometBFT node ID
        local comet_node_id=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
            "cometbft show-node-id --home $node_home/cometbft 2>/dev/null || echo ''")

        if [ -z "$comet_node_id" ]; then
            log_warn "Could not get CometBFT node ID for $name"
        else
            COMETBFT_PEERS[$idx]="${comet_node_id}@${ip}:${cometbft_port}"
            log_info "$name CometBFT: ${COMETBFT_PEERS[$idx]}"
        fi

        # Get libp2p peer ID
        local libp2p_id=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
            "grep 'local_peer_id' $node_home/logs/*.app.log 2>/dev/null | tail -1 | grep -oP '\"local_peer_id\":\"\\K[^\"]+' || echo ''")

        if [ -z "$libp2p_id" ]; then
            log_warn "Could not get libp2p peer ID for $name"
        else
            LIBP2P_PEERS[$idx]="/ip4/${ip}/tcp/${libp2p_port}/p2p/${libp2p_id}"
            log_info "$name libp2p: ${LIBP2P_PEERS[$idx]}"
        fi

        # Get validator public key
        local pubkey=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
            "cat $node_home/fendermint/validator.sk 2>/dev/null | grep -oP '\"public_key\":\"\\K[^\"]+' || echo ''")

        if [ -z "$pubkey" ]; then
            log_warn "Could not get validator public key for $name"
        else
            VALIDATOR_PUBKEYS[$idx]="$pubkey"
            log_info "$name pubkey: ${pubkey:0:20}..."
        fi
    done
}

# Update validator configs with full peer mesh
update_all_configs() {
    for idx in "${!VALIDATORS[@]}"; do
        local name="${VALIDATORS[$idx]}"
        log_info "Updating config for $name..."

        update_validator_config "$idx"
    done
}

# Update single validator config
update_validator_config() {
    local validator_idx="$1"

    local ip=$(get_config_value "validators[$validator_idx].ip")
    local ssh_user=$(get_config_value "validators[$validator_idx].ssh_user")
    local ipc_user=$(get_config_value "validators[$validator_idx].ipc_user")
    local node_home=$(get_config_value "paths.node_home")
    local libp2p_port=$(get_config_value "network.libp2p_port")

    # Build peer lists (excluding self)
    local comet_peers=""
    local libp2p_static_addrs=""

    for peer_idx in "${!VALIDATORS[@]}"; do
        if [ "$peer_idx" != "$validator_idx" ]; then
            if [ -n "${COMETBFT_PEERS[$peer_idx]:-}" ]; then
                comet_peers+="${COMETBFT_PEERS[$peer_idx]},"
            fi
            if [ -n "${LIBP2P_PEERS[$peer_idx]:-}" ]; then
                libp2p_static_addrs+="\"${LIBP2P_PEERS[$peer_idx]}\", "
            fi
        fi
    done

    # Remove trailing comma/space
    comet_peers="${comet_peers%,}"
    libp2p_static_addrs="${libp2p_static_addrs%, }"

    # Update CometBFT persistent_peers
    if [ -n "$comet_peers" ]; then
        ssh_exec "$ip" "$ssh_user" "$ipc_user" \
            "sed -i 's|^persistent_peers = .*|persistent_peers = \"$comet_peers\"|' $node_home/cometbft/config/config.toml"
    fi

    # Update Fendermint libp2p config
    if [ -n "$libp2p_static_addrs" ]; then
        ssh_exec "$ip" "$ssh_user" "$ipc_user" \
            "sed -i 's|^static_addresses = .*|static_addresses = [$libp2p_static_addrs]|' $node_home/fendermint/config/default.toml"
    fi

    # Update external_addresses
    if [ -n "${LIBP2P_PEERS[$validator_idx]:-}" ]; then
        ssh_exec "$ip" "$ssh_user" "$ipc_user" \
            "sed -i 's|^external_addresses = .*|external_addresses = [\"${LIBP2P_PEERS[$validator_idx]}\"]|' $node_home/fendermint/config/default.toml"
    fi

    # Ensure validator_key section exists
    ssh_exec "$ip" "$ssh_user" "$ipc_user" \
        "grep -q '\[validator_key\]' $node_home/fendermint/config/default.toml || echo -e '\n[validator_key]\npath = \"validator.sk\"\nkind = \"regular\"' >> $node_home/fendermint/config/default.toml"
}

