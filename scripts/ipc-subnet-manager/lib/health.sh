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
        "nohup $ipc_binary node start --home $node_home > $node_home/node.log 2>&1 &"
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
    local primary_peer_info="$2"

    local name="${VALIDATORS[$validator_idx]}"
    local ip=$(get_config_value "validators[$validator_idx].ip")
    local ssh_user=$(get_config_value "validators[$validator_idx].ssh_user")
    local ipc_user=$(get_config_value "validators[$validator_idx].ipc_user")
    local ipc_binary=$(get_config_value "paths.ipc_binary")
    local node_init_config=$(get_config_value "paths.node_init_config")

    log_info "Initializing $name..."

    # Copy primary's peer-info.json to secondary as peer1.json
    if [ -n "$primary_peer_info" ]; then
        local temp_peer_file="/tmp/peer1-${name}.json"
        echo "$primary_peer_info" > "$temp_peer_file"
        scp_to_host "$ip" "$ssh_user" "$ipc_user" "$temp_peer_file" "/home/$ipc_user/peer1.json"
        rm -f "$temp_peer_file"
    fi

    # Generate node-init.yml with peer file reference
    local temp_config="/tmp/node-init-${name}.yml"
    local peer_file_path=""
    if [ -n "$primary_peer_info" ]; then
        peer_file_path="/home/$ipc_user/peer1.json"
    fi
    generate_node_init_yml "$validator_idx" "$temp_config" "$peer_file_path"

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
    # Trim whitespace and newlines
    process_status=$(echo "$process_status" | tr -d '\n' | xargs)
    if [ "$process_status" = "running" ]; then
        log_check "ok" "Process running"
    else
        log_check "fail" "Process not running (status: '$process_status')"
        healthy=false
    fi

    # Check ports listening
    local ports_check=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
        "netstat -tuln 2>/dev/null | grep -E \":($cometbft_port|$libp2p_port|$eth_api_port)\" | wc -l")

    if [ -n "$ports_check" ] && [ "$ports_check" -ge 2 ] 2>/dev/null; then
        log_check "ok" "Ports listening ($ports_check/3)"
    else
        log_check "fail" "Ports not listening (${ports_check:-0}/3)"
        healthy=false
    fi

    # Check CometBFT peers
    local comet_peers=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
        "curl -s http://localhost:26657/net_info 2>/dev/null | jq -r '.result.n_peers // 0' 2>/dev/null || echo 0")

    local expected_peers=$((${#VALIDATORS[@]} - 1))
    # Ensure comet_peers is a number
    comet_peers=${comet_peers:-0}
    if [ "$comet_peers" -ge "$expected_peers" ] 2>/dev/null; then
        log_check "ok" "CometBFT peers: $comet_peers/$expected_peers"
    else
        log_check "fail" "CometBFT peers: $comet_peers/$expected_peers"
        healthy=false
    fi

    # Check block height
    local block_height=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
        "curl -s http://localhost:26657/status 2>/dev/null | jq -r '.result.sync_info.latest_block_height // 0' 2>/dev/null || echo 0")

    # Ensure block_height is a number
    block_height=${block_height:-0}
    if [ "$block_height" -gt 0 ] 2>/dev/null; then
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

# Measure block time for a validator
measure_block_time() {
    local validator_idx="$1"
    local sample_duration="${2:-10}"  # Default 10 seconds

    local name="${VALIDATORS[$validator_idx]}"
    local ip=$(get_config_value "validators[$validator_idx].ip")
    local ssh_user=$(get_config_value "validators[$validator_idx].ssh_user")
    local ipc_user=$(get_config_value "validators[$validator_idx].ipc_user")

    log_info "Measuring block time for $name (sampling for ${sample_duration}s)..."

    # Get initial block height and timestamp - extract directly without intermediate JSON
    local initial_height=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
        "curl -s http://localhost:26657/status 2>/dev/null | jq -r '.result.sync_info.latest_block_height // 0' 2>/dev/null")
    local initial_time=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
        "curl -s http://localhost:26657/status 2>/dev/null | jq -r '.result.sync_info.latest_block_time // \"\"' 2>/dev/null")

    if [ -z "$initial_height" ] || [ "$initial_height" = "0" ] || [ "$initial_height" = "null" ] || [ -z "$initial_time" ] || [ "$initial_time" = "null" ]; then
        log_warn "Could not get initial block data from $name"
        return 1
    fi

    log_info "  Initial: Block #$initial_height at $initial_time"

    # Wait for the sample duration
    sleep "$sample_duration"

    # Get final block height and timestamp
    local final_height=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
        "curl -s http://localhost:26657/status 2>/dev/null | jq -r '.result.sync_info.latest_block_height // 0' 2>/dev/null")
    local final_time=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
        "curl -s http://localhost:26657/status 2>/dev/null | jq -r '.result.sync_info.latest_block_time // \"\"' 2>/dev/null")

    if [ -z "$final_height" ] || [ "$final_height" = "0" ] || [ -z "$final_time" ]; then
        log_warn "Could not get final block data from $name"
        return 1
    fi

    log_info "  Final:   Block #$final_height at $final_time"

    # Calculate blocks produced
    local blocks_produced=$((final_height - initial_height))

    if [ "$blocks_produced" -le 0 ]; then
        log_warn "No blocks produced during sampling period"
        return 1
    fi

    # Calculate time difference in seconds
    local initial_ts=$(date -j -f "%Y-%m-%dT%H:%M:%S" "${initial_time%.*}" +%s 2>/dev/null || date -d "${initial_time%.*}" +%s 2>/dev/null)
    local final_ts=$(date -j -f "%Y-%m-%dT%H:%M:%S" "${final_time%.*}" +%s 2>/dev/null || date -d "${final_time%.*}" +%s 2>/dev/null)

    local time_diff=$((final_ts - initial_ts))

    if [ "$time_diff" -le 0 ]; then
        log_warn "Invalid time difference"
        return 1
    fi

    # Calculate average block time
    local avg_block_time=$(echo "scale=3; $time_diff / $blocks_produced" | bc)
    local blocks_per_second=$(echo "scale=3; $blocks_produced / $time_diff" | bc)

    log_success "Block time statistics for $name:"
    log_info "  Blocks produced: $blocks_produced"
    log_info "  Time elapsed: ${time_diff}s"
    log_info "  Average block time: ${avg_block_time}s"
    log_info "  Blocks per second: $blocks_per_second"

    return 0
}

# Measure block time for all validators
measure_all_block_times() {
    local sample_duration="${1:-10}"

    log_header "Block Time Measurement"
    log_info "Sample duration: ${sample_duration}s"
    echo

    for idx in "${!VALIDATORS[@]}"; do
        measure_block_time "$idx" "$sample_duration"
        echo
    done
}

# Get chain ID from a validator
get_chain_id() {
    local validator_idx="${1:-0}"

    local ip=$(get_config_value "validators[$validator_idx].ip")
    local ssh_user=$(get_config_value "validators[$validator_idx].ssh_user")
    local ipc_user=$(get_config_value "validators[$validator_idx].ipc_user")
    local eth_api_port=$(get_config_value "network.eth_api_port")

    # Query eth_chainId via JSON-RPC - using simpler quoting
    local response=$(ssh -o StrictHostKeyChecking=no "$ssh_user@$ip" \
        "sudo su - $ipc_user -c \"curl -s -X POST -H 'Content-Type: application/json' --data '{\\\"jsonrpc\\\":\\\"2.0\\\",\\\"method\\\":\\\"eth_chainId\\\",\\\"params\\\":[],\\\"id\\\":1}' http://localhost:${eth_api_port}\"" 2>/dev/null)

    local chain_id=$(echo "$response" | jq -r '.result // ""' 2>/dev/null)

    echo "$chain_id"
}

# Show comprehensive subnet information
show_subnet_info() {
    log_header "Subnet Information"

    # Get config values
    local subnet_id=$(get_config_value "subnet.id")
    local parent_subnet=$(get_config_value "subnet.parent_subnet")
    local parent_registry=$(get_config_value "subnet.parent_registry")
    local parent_gateway=$(get_config_value "subnet.parent_gateway")
    local num_validators=${#VALIDATORS[@]}

    echo
    log_info "Network Configuration:"
    log_info "  Subnet ID: $subnet_id"
    log_info "  Parent Subnet: $parent_subnet"
    log_info "  Parent Registry: $parent_registry"
    log_info "  Parent Gateway: $parent_gateway"
    echo

    log_info "Validators:"
    log_info "  Total: $num_validators"
    for idx in "${!VALIDATORS[@]}"; do
        local name="${VALIDATORS[$idx]}"
        local ip=$(get_config_value "validators[$idx].ip")
        log_info "    - $name ($ip)"
    done
    echo

    # Get chain ID from first validator
    log_info "Fetching chain ID from ${VALIDATORS[0]}..."
    local chain_id=$(get_chain_id 0)

    if [ -n "$chain_id" ] && [ "$chain_id" != "null" ] && [ "$chain_id" != "" ]; then
        # Convert hex to decimal if it starts with 0x
        if [[ "$chain_id" == 0x* ]]; then
            local chain_id_dec=$((chain_id))
            log_info "  Chain ID: $chain_id (decimal: $chain_id_dec)"
        else
            log_info "  Chain ID: $chain_id"
        fi
    else
        log_warn "  Could not fetch chain ID"
    fi
    echo

    # Get current block info from first validator
    log_info "Current Block Information (from ${VALIDATORS[0]}):"
    local ip=$(get_config_value "validators[0].ip")
    local ssh_user=$(get_config_value "validators[0].ssh_user")
    local ipc_user=$(get_config_value "validators[0].ipc_user")

    local block_height=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
        "curl -s http://localhost:26657/status 2>/dev/null | jq -r '.result.sync_info.latest_block_height // \"\"' 2>/dev/null")
    local block_time=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
        "curl -s http://localhost:26657/status 2>/dev/null | jq -r '.result.sync_info.latest_block_time // \"\"' 2>/dev/null")
    local catching_up=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
        "curl -s http://localhost:26657/status 2>/dev/null | jq -r '.result.sync_info.catching_up // \"\"' 2>/dev/null")

    if [ -n "$block_height" ] && [ "$block_height" != "null" ]; then
        log_info "  Latest Block Height: $block_height"
        log_info "  Latest Block Time: $block_time"
        log_info "  Catching Up: $catching_up"
    else
        log_warn "  Could not fetch block information"
    fi
    echo

    # Get network info
    log_info "Network Status:"
    local n_peers=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
        "curl -s http://localhost:26657/net_info 2>/dev/null | jq -r '.result.n_peers // 0' 2>/dev/null")
    local listening=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
        "curl -s http://localhost:26657/net_info 2>/dev/null | jq -r '.result.listening // false' 2>/dev/null")

    log_info "  Connected Peers: $n_peers"
    log_info "  Listening: $listening"
    echo

    # Check parent finality and top-down status (critical for cross-msg fund)
    log_info "Parent Finality Status (for cross-msg fund):"

    # Check recent logs for parent finality activity using separate greps
    local parent_finality_count=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
        "grep -i 'ParentFinalityCommitted' ~/.ipc-node/logs/*.log 2>/dev/null | wc -l" 2>/dev/null | tr -d ' ')

    if [ -n "$parent_finality_count" ] && [ "$parent_finality_count" -gt 0 ] 2>/dev/null; then
        log_info "  ✓ Parent finality commits detected: $parent_finality_count total"

        # Get the most recent one
        local last_finality=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
            "grep -i 'ParentFinalityCommitted' ~/.ipc-node/logs/*.log 2>/dev/null | tail -1" 2>/dev/null)

        if [ -n "$last_finality" ]; then
            # Extract timestamp
            local timestamp=$(echo "$last_finality" | grep -oE '[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}' | head -1)
            if [ -n "$timestamp" ]; then
                log_info "    Last commit: $timestamp"
            fi
        fi

        # Check for top-down message execution
        local topdown_count=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
            "grep -i 'topdown' ~/.ipc-node/logs/*.log 2>/dev/null | grep -i 'exec\|apply\|message' | wc -l" 2>/dev/null | tr -d ' ')

        if [ -n "$topdown_count" ] && [ "$topdown_count" -gt 0 ] 2>/dev/null; then
            log_info "  ✓ Top-down message activity: $topdown_count entries"
        fi
    else
        log_warn "  No parent finality commits found"
        log_info "    This is required for cross-msg fund to work!"
    fi
    echo

    # Show validator status summary
    log_info "Validator Health Summary:"
    for idx in "${!VALIDATORS[@]}"; do
        local val_name="${VALIDATORS[$idx]}"
        local val_ip=$(get_config_value "validators[$idx].ip")
        local val_ssh_user=$(get_config_value "validators[$idx].ssh_user")
        local val_ipc_user=$(get_config_value "validators[$idx].ipc_user")

        # Quick health check
        local is_running=$(ssh_exec "$val_ip" "$val_ssh_user" "$val_ipc_user" \
            "if pgrep -f \"ipc-cli node start\" >/dev/null 2>&1; then echo running; else echo stopped; fi" 2>/dev/null | tr -d '\n' | xargs)
        local val_height=$(ssh_exec "$val_ip" "$val_ssh_user" "$val_ipc_user" \
            "curl -s http://localhost:26657/status 2>/dev/null | jq -r '.result.sync_info.latest_block_height // \"0\"' 2>/dev/null")
        local val_peers=$(ssh_exec "$val_ip" "$val_ssh_user" "$val_ipc_user" \
            "curl -s http://localhost:26657/net_info 2>/dev/null | jq -r '.result.n_peers // 0' 2>/dev/null")

        if [ "$is_running" = "running" ]; then
            log_info "  ✓ $val_name: Running | Height: $val_height | Peers: $val_peers"
        else
            log_warn "  ✗ $val_name: Not running"
        fi
    done
    echo

    # Check for recent cross-msg related activity in logs
    log_info "Recent Cross-Chain Activity (last 5 entries):"

    # Get recent topdown-related logs
    local cross_msg_logs=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
        "grep -i 'topdown' ~/.ipc-node/logs/*.log 2>/dev/null | tail -5" 2>/dev/null)

    if [ -n "$cross_msg_logs" ] && [ "$cross_msg_logs" != "" ]; then
        echo "$cross_msg_logs" | while IFS= read -r line; do
            if [ -n "$line" ]; then
                # Extract just the relevant part (timestamp + message)
                local relevant=$(echo "$line" | sed 's/^.*\([0-9]\{4\}-[0-9]\{2\}-[0-9]\{2\}T[0-9]\{2\}:[0-9]\{2\}:[0-9]\{2\}\)/\1/' | cut -c1-100)
                log_info "  $relevant"
            fi
        done
    else
        log_info "  No recent topdown activity found in logs"
    fi
    echo
}

