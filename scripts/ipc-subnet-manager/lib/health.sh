#!/bin/bash
# Health check functions

# Initialize, backup, wipe, and start functions

backup_all_nodes() {
    for idx in "${!VALIDATORS[@]}"; do
        local name="${VALIDATORS[$idx]}"
        local ip=$(get_config_value "validators[$idx].ip")
        local ssh_user=$(get_config_value "validators[$idx].ssh_user")
        local ipc_user=$(get_config_value "validators[$idx].ipc_user")
        local node_home=$(get_config_value "paths.node_home")

        local timestamp=$(date +%Y%m%d%H%M%S)
        local backup_path="${node_home}.backup.${timestamp}"

        log_info "Creating backup for $name at $backup_path..."
        ssh_exec "$ip" "$ssh_user" "$ipc_user" \
            "if [ -d $node_home ]; then cp -r $node_home $backup_path; fi"
    done
}

wipe_all_nodes() {
    for idx in "${!VALIDATORS[@]}"; do
        local name="${VALIDATORS[$idx]}"
        local ip=$(get_config_value "validators[$idx].ip")
        local ssh_user=$(get_config_value "validators[$idx].ssh_user")
        local ipc_user=$(get_config_value "validators[$idx].ipc_user")
        local node_home=$(get_config_value "paths.node_home")

        log_info "Wiping $name..."
        ssh_exec "$ip" "$ssh_user" "$ipc_user" "rm -rf $node_home"
    done
}

stop_all_nodes() {
    for idx in "${!VALIDATORS[@]}"; do
        local name="${VALIDATORS[$idx]}"
        local ip=$(get_config_value "validators[$idx].ip")
        local ssh_user=$(get_config_value "validators[$idx].ssh_user")
        local ipc_user=$(get_config_value "validators[$idx].ipc_user")

        log_info "Stopping $name..."
        ssh_kill_process "$ip" "$ssh_user" "$ipc_user" "ipc-cli node start"

        # Wait a moment for graceful shutdown
        sleep 2
    done
}

start_all_nodes() {
    # Start primary first
    local primary_idx=$(get_primary_validator)
    start_validator_node "$primary_idx"

    # Wait a bit for primary to initialize
    sleep 5

    # Start secondaries
    for idx in "${!VALIDATORS[@]}"; do
        if [ "$idx" != "$primary_idx" ]; then
            start_validator_node "$idx"
            sleep 2
        fi
    done
}

start_validator_node() {
    local validator_idx="$1"

    local name="${VALIDATORS[$validator_idx]}"
    local ip=$(get_config_value "validators[$validator_idx].ip")
    local ssh_user=$(get_config_value "validators[$validator_idx].ssh_user")
    local ipc_user=$(get_config_value "validators[$validator_idx].ipc_user")
    local ipc_binary=$(get_config_value "paths.ipc_binary")
    local node_home=$(get_config_value "paths.node_home")

    log_info "Starting $name..."

    # Start node in background
    ssh_exec "$ip" "$ssh_user" "$ipc_user" \
        "nohup $ipc_binary node start > $node_home/node.log 2>&1 &"
}

initialize_primary_node() {
    local validator_idx="$1"

    local name="${VALIDATORS[$validator_idx]}"
    local ip=$(get_config_value "validators[$validator_idx].ip")
    local ssh_user=$(get_config_value "validators[$validator_idx].ssh_user")
    local ipc_user=$(get_config_value "validators[$validator_idx].ipc_user")
    local ipc_binary=$(get_config_value "paths.ipc_binary")
    local node_init_config=$(get_config_value "paths.node_init_config")

    log_info "Initializing $name (primary)..."

    # Generate node-init.yml
    local temp_config="/tmp/node-init-${name}.yml"
    generate_node_init_yml "$validator_idx" "$temp_config" ""

    # Copy to remote
    scp_to_host "$ip" "$ssh_user" "$ipc_user" "$temp_config" "$node_init_config"
    rm -f "$temp_config"

    # Run init
    local init_output=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
        "$ipc_binary node init --config $node_init_config 2>&1")

    if echo "$init_output" | grep -q "Error\|error\|failed"; then
        log_error "Initialization failed for $name"
        echo "$init_output"
        exit 1
    fi

    log_success "$name initialized successfully"
}

initialize_secondary_nodes() {
    local primary_peer_info="$1"

    for idx in "${!VALIDATORS[@]}"; do
        local role=$(get_config_value "validators[$idx].role")
        if [ "$role" = "secondary" ]; then
            initialize_secondary_node "$idx" "$primary_peer_info"
        fi
    done
}

initialize_secondary_node() {
    local validator_idx="$1"
    local peers="$2"

    local name="${VALIDATORS[$validator_idx]}"
    local ip=$(get_config_value "validators[$validator_idx].ip")
    local ssh_user=$(get_config_value "validators[$validator_idx].ssh_user")
    local ipc_user=$(get_config_value "validators[$validator_idx].ipc_user")
    local ipc_binary=$(get_config_value "paths.ipc_binary")
    local node_init_config=$(get_config_value "paths.node_init_config")

    log_info "Initializing $name..."

    # Generate node-init.yml with peers
    local temp_config="/tmp/node-init-${name}.yml"
    generate_node_init_yml "$validator_idx" "$temp_config" "$peers"

    # Copy to remote
    scp_to_host "$ip" "$ssh_user" "$ipc_user" "$temp_config" "$node_init_config"
    rm -f "$temp_config"

    # Run init
    local init_output=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
        "$ipc_binary node init --config $node_init_config 2>&1")

    if echo "$init_output" | grep -q "Error\|error\|failed"; then
        log_error "Initialization failed for $name"
        echo "$init_output"
        exit 1
    fi

    log_success "$name initialized successfully"
}

set_federated_power() {
    local primary_idx=$(get_primary_validator)
    local name="${VALIDATORS[$primary_idx]}"
    local ip=$(get_config_value "validators[$primary_idx].ip")
    local ssh_user=$(get_config_value "validators[$primary_idx].ssh_user")
    local ipc_user=$(get_config_value "validators[$primary_idx].ipc_user")
    local ipc_binary=$(get_config_value "paths.ipc_binary")
    local subnet_id=$(get_config_value "subnet.id")
    local validator_power=$(get_config_value "init.validator_power")

    # Collect all validator public keys (without 0x prefix)
    local pubkeys=""
    for idx in "${!VALIDATORS[@]}"; do
        if [ -n "${VALIDATOR_PUBKEYS[$idx]:-}" ]; then
            local clean_pubkey="${VALIDATOR_PUBKEYS[$idx]#0x}"
            pubkeys+="${clean_pubkey},"
        fi
    done
    pubkeys="${pubkeys%,}"

    if [ -z "$pubkeys" ]; then
        log_warn "No validator public keys found, skipping federated power setup"
        return
    fi

    log_info "Setting federated power for ${#VALIDATOR_PUBKEYS[@]} validators..."
    log_info "Power per validator: $validator_power"

    # Run set-federated-power from primary node
    local cmd="$ipc_binary subnet set-federated-power --subnet $subnet_id --validator-pubkeys $pubkeys --validator-power $validator_power --from t1d4gxuxytb6vg7cxzvxqk3cvbx4hv7vrtd6oa2mi"

    local output=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" "$cmd 2>&1")

    if echo "$output" | grep -q "Error\|error\|failed"; then
        log_error "Failed to set federated power"
        echo "$output"
    else
        log_success "Federated power configured"
    fi
}

# Health check for single validator
check_validator_health() {
    local validator_idx="$1"

    local name="${VALIDATORS[$validator_idx]}"
    local ip=$(get_config_value "validators[$validator_idx].ip")
    local ssh_user=$(get_config_value "validators[$validator_idx].ssh_user")
    local ipc_user=$(get_config_value "validators[$validator_idx].ipc_user")
    local node_home=$(get_config_value "paths.node_home")
    local cometbft_port=$(get_config_value "network.cometbft_p2p_port")
    local libp2p_port=$(get_config_value "network.libp2p_port")
    local eth_api_port=$(get_config_value "network.eth_api_port")

    local healthy=true

    # Check process running
    local process_status=$(ssh_check_process "$ip" "$ssh_user" "$ipc_user" "ipc-cli node start")
    if [ "$process_status" = "running" ]; then
        log_check "ok" "Process running"
    else
        log_check "fail" "Process not running"
        healthy=false
    fi

    # Check ports listening
    local ports_check=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
        "netstat -tuln 2>/dev/null | grep -E ':(${cometbft_port}|${libp2p_port}|${eth_api_port})' | wc -l")

    if [ "$ports_check" -ge 2 ]; then
        log_check "ok" "Ports listening ($ports_check/3)"
    else
        log_check "fail" "Ports not listening ($ports_check/3)"
        healthy=false
    fi

    # Check CometBFT peers
    local comet_peers=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
        "curl -s http://localhost:26657/net_info 2>/dev/null | grep -o '\"n_peers\":\"[0-9]*\"' | grep -o '[0-9]*' || echo 0")

    local expected_peers=$((${#VALIDATORS[@]} - 1))
    if [ "$comet_peers" -ge "$expected_peers" ]; then
        log_check "ok" "CometBFT peers: $comet_peers/$expected_peers"
    else
        log_check "fail" "CometBFT peers: $comet_peers/$expected_peers"
        healthy=false
    fi

    # Check block height
    local block_height=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
        "curl -s http://localhost:26657/status 2>/dev/null | grep -o '\"latest_block_height\":\"[0-9]*\"' | grep -o '[0-9]*' || echo 0")

    if [ "$block_height" -gt 0 ]; then
        log_check "ok" "Block height: $block_height"
    else
        log_check "fail" "Block height: $block_height (chain not producing blocks)"
        healthy=false
    fi

    # Check for recent errors in logs
    local recent_errors=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
        "tail -100 $node_home/logs/*.log 2>/dev/null | grep -i 'ERROR' | tail -5 || echo ''")

    if [ -z "$recent_errors" ]; then
        log_check "ok" "No recent errors"
    else
        log_check "fail" "Recent errors found"
        echo "$recent_errors" | head -3
        healthy=false
    fi

    if [ "$healthy" = true ]; then
        return 0
    else
        return 1
    fi
}

