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

# Generate systemd service file for node
generate_node_systemd_service() {
    local validator_idx="$1"
    local output_file="$2"

    local ipc_user=$(get_config_value "validators[$validator_idx].ipc_user")
    local ipc_binary=$(get_config_value "paths.ipc_binary")
    local node_home=$(get_config_value "paths.node_home")

    # Ensure SCRIPT_DIR is set
    if [ -z "$SCRIPT_DIR" ]; then
        SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
    fi

    sed -e "s|__IPC_USER__|$ipc_user|g" \
        -e "s|__IPC_BINARY__|$ipc_binary|g" \
        -e "s|__NODE_HOME__|$node_home|g" \
        "${SCRIPT_DIR}/templates/ipc-node.service.template" > "$output_file"
}

# Generate systemd service file for relayer
generate_relayer_systemd_service() {
    local validator_idx="$1"
    local output_file="$2"

    local ipc_user=$(get_config_value "validators[$validator_idx].ipc_user")
    local ipc_binary=$(get_config_value "paths.ipc_binary")
    local node_home=$(get_config_value "paths.node_home")
    local subnet_id=$(get_config_value "subnet.id")
    local checkpoint_interval=$(get_config_value "relayer.checkpoint_interval")
    local max_parallelism=$(get_config_value "relayer.max_parallelism")
    local eth_api_port=$(get_config_value "network.eth_api_port")

    # Fendermint RPC URL is the local ETH API endpoint
    local fendermint_rpc_url="http://localhost:${eth_api_port}"

    # Get submitter address
    local submitter=$(get_validator_address_from_keystore "$validator_idx")

    if [ -z "$submitter" ]; then
        log_error "Failed to get submitter address for systemd service"
        return 1
    fi

    # Ensure SCRIPT_DIR is set
    if [ -z "$SCRIPT_DIR" ]; then
        SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
    fi

    sed -e "s|__IPC_USER__|$ipc_user|g" \
        -e "s|__IPC_BINARY__|$ipc_binary|g" \
        -e "s|__NODE_HOME__|$node_home|g" \
        -e "s|__SUBNET_ID__|$subnet_id|g" \
        -e "s|__FENDERMINT_RPC_URL__|$fendermint_rpc_url|g" \
        -e "s|__CHECKPOINT_INTERVAL__|$checkpoint_interval|g" \
        -e "s|__MAX_PARALLELISM__|$max_parallelism|g" \
        -e "s|__SUBMITTER_ADDRESS__|$submitter|g" \
        "${SCRIPT_DIR}/templates/ipc-relayer.service.template" > "$output_file"
}

# Check if systemd is available
check_systemd_available() {
    local ip="$1"
    local ssh_user="$2"

    # Check if systemd is available (just check the system one)
    local result=$(ssh -o StrictHostKeyChecking=no "$ssh_user@$ip" \
        "systemctl --version >/dev/null 2>&1 && echo 'yes' || echo 'no'" 2>/dev/null)

    echo "$result"
}

# Install systemd services on a validator
install_systemd_services() {
    local validator_idx="$1"

    local name="${VALIDATORS[$validator_idx]}"
    local ip=$(get_config_value "validators[$validator_idx].ip")
    local ssh_user=$(get_config_value "validators[$validator_idx].ssh_user")
    local ipc_user=$(get_config_value "validators[$validator_idx].ipc_user")
    local node_home=$(get_config_value "paths.node_home")

    log_info "Checking systemd availability on $name..."

    # Check if systemd is available
    local systemd_available=$(check_systemd_available "$ip" "$ssh_user")

    if [ "$systemd_available" != "yes" ]; then
        log_warn "✗ Systemd not available on $name"
        log_info "  You can still manage processes manually without systemd"
        return 1
    fi

    log_info "Installing systemd service on $name..."

    # Generate node service file
    local node_service_file="/tmp/ipc-node-${name}.service"
    generate_node_systemd_service "$validator_idx" "$node_service_file"

    if [ ! -f "$node_service_file" ]; then
        log_error "Failed to generate service file for $name"
        return 1
    fi

    # Ensure logs directory exists
    ssh_exec "$ip" "$ssh_user" "$ipc_user" "mkdir -p $node_home/logs" 2>/dev/null || true

    # Copy service file to /etc/systemd/system/ (requires sudo)
    log_info "  Copying service file to $name..."
    if ! scp -o StrictHostKeyChecking=no "$node_service_file" "$ssh_user@$ip:/tmp/ipc-node.service" >/dev/null 2>&1; then
        log_error "Failed to copy service file to $name"
        rm -f "$node_service_file"
        return 1
    fi

    log_info "  Moving to /etc/systemd/system/..."
    if ! ssh -o StrictHostKeyChecking=no "$ssh_user@$ip" \
        "sudo mv /tmp/ipc-node.service /etc/systemd/system/ipc-node.service && sudo chmod 644 /etc/systemd/system/ipc-node.service" >/dev/null 2>&1; then
        log_error "Failed to install service file on $name"
        rm -f "$node_service_file"
        return 1
    fi

    # Reload systemd
    log_info "  Reloading systemd..."
    if ! ssh -o StrictHostKeyChecking=no "$ssh_user@$ip" \
        "sudo systemctl daemon-reload" >/dev/null 2>&1; then
        log_error "Failed to reload systemd on $name"
        rm -f "$node_service_file"
        return 1
    fi

    # Enable node service
    log_info "  Enabling service..."
    ssh -o StrictHostKeyChecking=no "$ssh_user@$ip" \
        "sudo systemctl enable ipc-node.service" >/dev/null 2>&1 || true

    log_success "✓ Node service installed on $name"

    # Cleanup
    rm -f "$node_service_file"
    return 0
}

# Install relayer systemd service on primary validator
install_relayer_systemd_service() {
    local validator_idx="$1"

    local name="${VALIDATORS[$validator_idx]}"
    local ip=$(get_config_value "validators[$validator_idx].ip")
    local ssh_user=$(get_config_value "validators[$validator_idx].ssh_user")
    local ipc_user=$(get_config_value "validators[$validator_idx].ipc_user")

    # Check if systemd is available
    local systemd_available=$(check_systemd_available "$ip" "$ssh_user")

    if [ "$systemd_available" != "yes" ]; then
        log_warn "✗ Systemd not available on $name"
        log_info "  Relayer will need to be managed manually"
        return 1
    fi

    log_info "Installing relayer systemd service on $name..."

    # Generate relayer service file
    local relayer_service_file="/tmp/ipc-relayer-${name}.service"
    generate_relayer_systemd_service "$validator_idx" "$relayer_service_file"

    if [ ! -f "$relayer_service_file" ]; then
        log_error "Failed to generate relayer service file"
        return 1
    fi

    # Copy service file to /etc/systemd/system/ (requires sudo)
    log_info "  Copying relayer service file to $name..."
    if ! scp -o StrictHostKeyChecking=no "$relayer_service_file" "$ssh_user@$ip:/tmp/ipc-relayer.service" >/dev/null 2>&1; then
        log_error "Failed to copy relayer service file to $name"
        rm -f "$relayer_service_file"
        return 1
    fi

    log_info "  Moving to /etc/systemd/system/..."
    if ! ssh -o StrictHostKeyChecking=no "$ssh_user@$ip" \
        "sudo mv /tmp/ipc-relayer.service /etc/systemd/system/ipc-relayer.service && sudo chmod 644 /etc/systemd/system/ipc-relayer.service" >/dev/null 2>&1; then
        log_error "Failed to install relayer service file on $name"
        rm -f "$relayer_service_file"
        return 1
    fi

    # Reload systemd
    log_info "  Reloading systemd..."
    if ! ssh -o StrictHostKeyChecking=no "$ssh_user@$ip" \
        "sudo systemctl daemon-reload" >/dev/null 2>&1; then
        log_error "Failed to reload systemd on $name"
        rm -f "$relayer_service_file"
        return 1
    fi

    # Enable relayer service
    log_info "  Enabling relayer service..."
    ssh -o StrictHostKeyChecking=no "$ssh_user@$ip" \
        "sudo systemctl enable ipc-relayer.service" >/dev/null 2>&1 || true

    log_success "✓ Relayer service installed on $name"

    # Cleanup
    rm -f "$relayer_service_file"
    return 0
}

stop_all_nodes() {
    for idx in "${!VALIDATORS[@]}"; do
        local name="${VALIDATORS[$idx]}"
        local ip=$(get_config_value "validators[$idx].ip")
        local ssh_user=$(get_config_value "validators[$idx].ssh_user")
        local ipc_user=$(get_config_value "validators[$idx].ipc_user")

        log_info "Stopping $name..."

        # Try systemd first, fall back to manual kill
        local has_systemd=$(ssh -o StrictHostKeyChecking=no "$ssh_user@$ip" \
            "systemctl is-active ipc-node 2>/dev/null | grep -q active && echo yes || echo no" 2>/dev/null)

        if [ "$has_systemd" = "yes" ]; then
            ssh -o StrictHostKeyChecking=no "$ssh_user@$ip" "sudo systemctl stop ipc-node" >/dev/null 2>&1 || true
        else
            ssh_kill_process "$ip" "$ssh_user" "$ipc_user" "ipc-cli node start"
        fi

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

    # Try systemd first, fall back to nohup
    local has_systemd=$(ssh -o StrictHostKeyChecking=no "$ssh_user@$ip" \
        "systemctl list-unit-files ipc-node.service 2>/dev/null | grep -q ipc-node && echo yes || echo no" 2>/dev/null)

    if [ "$has_systemd" = "yes" ]; then
        ssh -o StrictHostKeyChecking=no "$ssh_user@$ip" "sudo systemctl start ipc-node" >/dev/null 2>&1 || true
    else
        # Fall back to nohup
        ssh_exec "$ip" "$ssh_user" "$ipc_user" \
            "nohup $ipc_binary node start --home $node_home > $node_home/logs/node.stdout.log 2>&1 &"
    fi
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

    # Show generated config for debugging
    if [ "${DEBUG:-false}" = true ]; then
        log_debug "Generated node-init.yml for $name:"
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        cat "$temp_config"
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    else
        log_info "Generated node-init.yml for $name (use --debug to view full config)"
    fi

    # Copy to remote
    scp_to_host "$ip" "$ssh_user" "$ipc_user" "$temp_config" "$node_init_config"
    rm -f "$temp_config"

    # Test parent chain connectivity from the remote node
    log_info "Testing parent chain connectivity from $name..."
    local parent_rpc=$(get_config_value "subnet.parent_rpc")
    local parent_test=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
        "curl -s -X POST -H 'Content-Type: application/json' --data '{\"jsonrpc\":\"2.0\",\"method\":\"eth_blockNumber\",\"params\":[],\"id\":1}' '$parent_rpc' 2>&1")

    if echo "$parent_test" | grep -q "error\|failed\|refused"; then
        log_error "Cannot reach parent chain RPC at $parent_rpc from $name"
        echo "$parent_test"
        log_info "Please verify:"
        log_info "  1. Parent RPC URL is correct: $parent_rpc"
        log_info "  2. Parent chain is running and accessible from the validator node"
        log_info "  3. No firewall blocking the connection"
        exit 1
    else
        log_success "Parent chain connectivity OK"
    fi

    # Run init with verbose logging if debug mode
    if [ "${DEBUG:-false}" = true ]; then
        log_info "Running ipc-cli node init with verbose logging..."
        local init_output=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
            "RUST_LOG=debug,ipc_cli=trace $ipc_binary node init --config $node_init_config 2>&1")
    else
        log_info "Running ipc-cli node init..."
        local init_output=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
            "$ipc_binary node init --config $node_init_config 2>&1")
    fi

    if echo "$init_output" | grep -q "Error\|error\|failed"; then
        log_error "Initialization failed for $name"

        if [ "${DEBUG:-false}" = true ]; then
            echo ""
            echo "━━━━━━━━━━━━━━━━━━━━━━━ DETAILED ERROR OUTPUT ━━━━━━━━━━━━━━━━━━━━━━━"
            echo "$init_output"
            echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
            echo ""
        else
            # Show just the error line(s)
            echo ""
            echo "Error summary:"
            echo "$init_output" | grep -i "error" | head -5
            echo ""
            log_info "Run with --debug flag to see full output"
        fi

        echo ""
        log_info "Troubleshooting tips:"
        log_info "  1. Check if parent_registry and parent_gateway addresses are correct"
        log_info "  2. Verify subnet already exists on parent chain: $parent_rpc"
        log_info "  3. Check if the subnet ID is correct: $(get_config_value 'subnet.id')"
        log_info "  4. Try querying parent chain manually:"
        log_info "     curl -X POST -H 'Content-Type: application/json' \\"
        log_info "          --data '{\"jsonrpc\":\"2.0\",\"method\":\"eth_blockNumber\",\"params\":[],\"id\":1}' \\"
        log_info "          '$parent_rpc'"
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

    # Show generated config for debugging
    if [ "${DEBUG:-false}" = true ]; then
        log_debug "Generated node-init.yml for $name:"
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        cat "$temp_config"
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    else
        log_info "Generated node-init.yml for $name (use --debug to view full config)"
    fi

    # Copy to remote
    scp_to_host "$ip" "$ssh_user" "$ipc_user" "$temp_config" "$node_init_config"
    rm -f "$temp_config"

    # Run init with verbose logging if debug mode
    if [ "${DEBUG:-false}" = true ]; then
        log_info "Running ipc-cli node init with verbose logging..."
        local init_output=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
            "RUST_LOG=debug,ipc_cli=trace $ipc_binary node init --config $node_init_config 2>&1")
    else
        log_info "Running ipc-cli node init..."
        local init_output=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
            "$ipc_binary node init --config $node_init_config 2>&1")
    fi

    if echo "$init_output" | grep -q "Error\|error\|failed"; then
        log_error "Initialization failed for $name"

        if [ "${DEBUG:-false}" = true ]; then
            echo ""
            echo "━━━━━━━━━━━━━━━━━━━━━━━ DETAILED ERROR OUTPUT ━━━━━━━━━━━━━━━━━━━━━━━"
            echo "$init_output"
            echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
            echo ""
        else
            # Show just the error line(s)
            echo ""
            echo "Error summary:"
            echo "$init_output" | grep -i "error" | head -5
            echo ""
            log_info "Run with --debug flag to see full output"
        fi

        echo ""
        log_info "Troubleshooting tips:"
        log_info "  1. Check if parent_registry and parent_gateway addresses are correct"
        log_info "  2. Verify subnet already exists on parent chain"
        log_info "  3. Check if the subnet ID is correct: $(get_config_value 'subnet.id')"
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
    for idx in "${!VALIDATOR_PUBKEYS[@]}"; do
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

# Update binaries on a single validator
update_validator_binaries() {
    local validator_idx="$1"
    local branch="$2"

    local name="${VALIDATORS[$validator_idx]}"
    local ip=$(get_config_value "validators[$validator_idx].ip")
    local ssh_user=$(get_config_value "validators[$validator_idx].ssh_user")
    local ipc_user=$(get_config_value "validators[$validator_idx].ipc_user")
    local ipc_repo=$(get_config_value "paths.ipc_repo")

    log_info "[$name] Updating binaries from branch '$branch'..."

    # Build update commands
    local update_cmd="cd $ipc_repo && \
        git fetch origin && \
        git checkout $branch && \
        git pull origin $branch && \
        make"

    # Execute build
    log_info "[$name] Pulling latest changes and building..."
    local build_output=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" "$update_cmd 2>&1")
    local build_exit=$?

    if [ $build_exit -ne 0 ]; then
        log_error "[$name] Build failed"
        echo "$build_output" | tail -20
        return 1
    fi

    log_success "[$name] Build completed successfully"

    # Copy binaries to /usr/local/bin (requires sudo)
    log_info "[$name] Installing binaries to /usr/local/bin..."
    ssh -o StrictHostKeyChecking=no "$ssh_user@$ip" \
        "sudo cp $ipc_repo/target/release/ipc-cli /usr/local/bin/ipc-cli && \
         sudo cp $ipc_repo/target/release/fendermint /usr/local/bin/fendermint && \
         sudo chmod +x /usr/local/bin/ipc-cli /usr/local/bin/fendermint" >/dev/null 2>&1

    if [ $? -ne 0 ]; then
        log_error "[$name] Failed to install binaries"
        return 1
    fi

    log_success "[$name] Binaries installed successfully"

    # Verify installation
    local ipc_version=$(ssh -o StrictHostKeyChecking=no "$ssh_user@$ip" \
        "/usr/local/bin/ipc-cli --version 2>&1 | head -1")
    log_info "[$name] ipc-cli version: $ipc_version"

    return 0
}

# Update binaries on all validators
update_all_binaries() {
    local branch="${1:-main}"

    log_header "Updating IPC Binaries"
    log_info "Branch: $branch"
    log_info "Validators: ${#VALIDATORS[@]}"
    echo ""

    # Array to track background jobs
    local pids=()
    local results=()

    # Start updates in parallel
    for idx in "${!VALIDATORS[@]}"; do
        update_validator_binaries "$idx" "$branch" &
        pids[$idx]=$!
    done

    # Wait for all jobs to complete
    log_info "Waiting for all builds to complete..."
    local all_success=true

    for idx in "${!VALIDATORS[@]}"; do
        wait ${pids[$idx]}
        results[$idx]=$?
        if [ ${results[$idx]} -ne 0 ]; then
            all_success=false
        fi
    done

    echo ""
    log_section "Update Summary"

    for idx in "${!VALIDATORS[@]}"; do
        local name="${VALIDATORS[$idx]}"
        if [ ${results[$idx]} -eq 0 ]; then
            log_success "✓ $name: Update successful"
        else
            log_error "✗ $name: Update failed"
        fi
    done

    if [ "$all_success" = true ]; then
        echo ""
        log_success "✓ All validators updated successfully"
        log_info "You may need to restart nodes for changes to take effect:"
        log_info "  $0 restart"
        return 0
    else
        echo ""
        log_error "✗ Some validators failed to update"
        return 1
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
        local ssh_user=$(get_config_value "validators[$idx].ssh_user")
        local ipc_user=$(get_config_value "validators[$idx].ipc_user")
        local node_home=$(get_config_value "paths.node_home")

        # Get validator public key
        local pubkey=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
            "cat $node_home/fendermint/validator.pk 2>/dev/null || echo ''")

        if [ -n "$pubkey" ]; then
            # Convert validator key to Ethereum address using fendermint
            local eth_address=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
                "fendermint key into-eth --secret-key $node_home/fendermint/validator.sk --name temp --out-dir /tmp 2>/dev/null && cat /tmp/temp.addr 2>/dev/null && rm -f /tmp/temp.* || echo ''")

            # Add 0x prefix if address was successfully converted
            if [ -n "$eth_address" ] && [ "$eth_address" != "" ]; then
                eth_address="0x${eth_address}"
            fi

            log_info "    - $name ($ip)"
            log_info "      Public Key: $pubkey"
            if [ -n "$eth_address" ]; then
                log_info "      Address: $eth_address"
            else
                log_warn "      Address: Unable to convert"
            fi
        else
            log_info "    - $name ($ip)"
            log_warn "      Public Key: Not found"
        fi
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

    log_info "  CometBFT Peers: $n_peers"
    log_info "  CometBFT Listening: $listening"
    echo

    # Check critical infrastructure for parent finality voting
    log_info "Libp2p Infrastructure (required for voting):"
    local libp2p_port=$(get_config_value "network.libp2p_port")

    # Check if libp2p port is listening and on correct address
    local libp2p_listening=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
        "ss -tulpn 2>/dev/null | grep ':$libp2p_port ' | head -1" 2>/dev/null)

    if [ -n "$libp2p_listening" ]; then
        if echo "$libp2p_listening" | grep -q "0.0.0.0:$libp2p_port"; then
            log_info "  ✓ Libp2p port $libp2p_port listening on 0.0.0.0 (can accept connections)"
        elif echo "$libp2p_listening" | grep -q "127.0.0.1:$libp2p_port"; then
            log_warn "  ✗ Libp2p port $libp2p_port bound to 127.0.0.1 (cannot accept external connections!)"
            log_warn "    Run: ./ipc-manager update-config to fix"
        else
            log_info "  ⚠ Libp2p port $libp2p_port listening: $(echo $libp2p_listening | awk '{print $5}')"
        fi
    else
        log_warn "  ✗ Libp2p port $libp2p_port not listening!"
    fi

    # Check if resolver is enabled in config
    local resolver_enabled=$(ssh -o StrictHostKeyChecking=no "$ssh_user@$ip" \
        "sudo su - $ipc_user -c 'grep -A3 \"\\[resolver\\]\" ~/.ipc-node/fendermint/config/default.toml | grep enabled | grep -o \"true\\|false\"'" 2>/dev/null | head -1 | tr -d '\n\r ')

    if [ "$resolver_enabled" = "true" ]; then
        log_info "  ✓ Resolver enabled in config"

        # Check if resolver service started
        local resolver_started=$(ssh -o StrictHostKeyChecking=no "$ssh_user@$ip" \
            "sudo su - $ipc_user -c 'grep \"starting the IPLD Resolver Service\" ~/.ipc-node/logs/*.log 2>/dev/null | wc -l'" 2>/dev/null | tr -d ' \n\r')

        if [ -n "$resolver_started" ] && [ "$resolver_started" -gt 0 ] 2>/dev/null; then
            log_info "  ✓ Resolver service started ($resolver_started times)"

            # Check if vote gossip loop started
            local vote_loop=$(ssh -o StrictHostKeyChecking=no "$ssh_user@$ip" \
                "sudo su - $ipc_user -c 'grep \"parent finality vote gossip loop\" ~/.ipc-node/logs/*.log 2>/dev/null | wc -l'" 2>/dev/null | tr -d ' \n\r')

            if [ -n "$vote_loop" ] && [ "$vote_loop" -gt 0 ] 2>/dev/null; then
                log_info "  ✓ Vote gossip loop active"
            else
                log_warn "  ✗ Vote gossip loop not started"
            fi
        else
            log_warn "  ✗ Resolver service did not start"
        fi
    else
        log_warn "  ✗ Resolver not enabled in config (found: '$resolver_enabled')!"
    fi

    # Check listen_addr configuration
    local listen_addr=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
        "grep 'listen_addr' ~/.ipc-node/fendermint/config/default.toml 2>/dev/null | head -1" 2>/dev/null)

    if echo "$listen_addr" | grep -q "0.0.0.0"; then
        log_info "  ✓ Listen address configured correctly (0.0.0.0)"
    elif echo "$listen_addr" | grep -q "127.0.0.1"; then
        log_warn "  ✗ Listen address misconfigured (127.0.0.1 - run update-config)"
    fi
    echo

    # Check external_addresses and static_addresses for all validators
    log_info "Libp2p Peer Configuration:"
    for idx in "${!VALIDATORS[@]}"; do
        local v_name="${VALIDATORS[$idx]}"
        local v_ip=$(get_config_value "validators[$idx].ip")
        local v_ssh_user=$(get_config_value "validators[$idx].ssh_user")
        local v_ipc_user=$(get_config_value "validators[$idx].ipc_user")
        local v_node_home=$(get_config_value "paths.node_home")

        log_info "  $v_name ($v_ip):"

        # Get external_addresses
        local ext_addrs=$(ssh -o StrictHostKeyChecking=no "$v_ssh_user@$v_ip" \
            "sudo su - $v_ipc_user -c 'grep external_addresses $v_node_home/fendermint/config/default.toml 2>/dev/null'" 2>/dev/null)

        if [ -n "$ext_addrs" ] && echo "$ext_addrs" | grep -q "/ip4/$v_ip/tcp/$libp2p_port"; then
            log_info "    ✓ external_addresses: Contains own IP ($v_ip)"
        elif [ -n "$ext_addrs" ]; then
            log_warn "    ✗ external_addresses: $(echo "$ext_addrs" | cut -c1-80)"
            log_warn "      Expected to contain: /ip4/$v_ip/tcp/$libp2p_port"
        else
            log_warn "    ✗ external_addresses: Not set or empty"
        fi

        # Get static_addresses
        local static_addrs=$(ssh -o StrictHostKeyChecking=no "$v_ssh_user@$v_ip" \
            "sudo su - $v_ipc_user -c 'grep static_addresses $v_node_home/fendermint/config/default.toml 2>/dev/null'" 2>/dev/null)

        if [ -n "$static_addrs" ]; then
            # Count how many peer IPs are in static_addresses
            local peer_count=0
            for peer_idx in "${!VALIDATORS[@]}"; do
                if [ "$peer_idx" != "$idx" ]; then
                    local peer_ip=$(get_config_value "validators[$peer_idx].ip")
                    if echo "$static_addrs" | grep -q "/ip4/$peer_ip/tcp/$libp2p_port"; then
                        peer_count=$((peer_count + 1))
                    fi
                fi
            done

            local expected_peers=$((${#VALIDATORS[@]} - 1))
            if [ "$peer_count" -eq "$expected_peers" ]; then
                log_info "    ✓ static_addresses: Contains all $expected_peers peer IPs"
            else
                log_warn "    ✗ static_addresses: Only $peer_count of $expected_peers peer IPs found"
                log_warn "      Check: $(echo "$static_addrs" | cut -c1-100)"
            fi
        else
            log_warn "    ✗ static_addresses: Not set or empty"
            log_warn "      Run: ./ipc-manager update-config to fix"
        fi

        # Check if libp2p connections are actually established
        local libp2p_connections=$(ssh -o StrictHostKeyChecking=no "$v_ssh_user@$v_ip" \
            "sudo su - $v_ipc_user -c 'ss -tn | grep :$libp2p_port | grep ESTAB | wc -l'" 2>/dev/null | tr -d ' \n\r')

        if [ -n "$libp2p_connections" ] && [ "$libp2p_connections" -gt 0 ] 2>/dev/null; then
            log_info "    ✓ Active libp2p connections: $libp2p_connections"
        else
            log_warn "    ✗ No active libp2p connections (firewall blocking port $libp2p_port?)"
        fi
    done
    echo

    # Check parent chain connectivity
    log_info "Parent Chain Connectivity:"

    # Check if parent RPC is reachable
    local parent_rpc_errors=$(ssh -o StrictHostKeyChecking=no "$ssh_user@$ip" \
        "sudo su - $ipc_user -c 'grep -i \"failed to get.*parent\\|parent.*connection.*failed\\|parent.*RPC.*error\" ~/.ipc-node/logs/*.log 2>/dev/null | wc -l'" 2>/dev/null | tr -d ' \n\r')

    if [ -n "$parent_rpc_errors" ] && [ "$parent_rpc_errors" -gt 0 ] 2>/dev/null; then
        log_warn "  ✗ Parent RPC errors detected ($parent_rpc_errors occurrences)"
        # Show a sample error
        local sample_error=$(ssh -o StrictHostKeyChecking=no "$ssh_user@$ip" \
            "sudo su - $ipc_user -c 'grep -i \"failed to get.*parent\\|parent.*connection.*failed\" ~/.ipc-node/logs/*.log 2>/dev/null | tail -1'" 2>/dev/null)
        if [ -n "$sample_error" ]; then
            log_warn "    Sample: $(echo "$sample_error" | tail -c 120)"
        fi
    else
        log_info "  ✓ No parent RPC connection errors detected"
    fi

    # Check if parent blocks are being fetched
    local parent_blocks_fetched=$(ssh -o StrictHostKeyChecking=no "$ssh_user@$ip" \
        "sudo su - $ipc_user -c 'grep -i \"parent.*block.*height\\|fetched.*parent\" ~/.ipc-node/logs/*.log 2>/dev/null | tail -1'" 2>/dev/null)

    if [ -n "$parent_blocks_fetched" ]; then
        log_info "  ✓ Parent block data being fetched"
        log_info "    Recent: $(echo "$parent_blocks_fetched" | grep -oE '[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}' | head -1)"
    else
        log_warn "  ✗ No evidence of parent block fetching"
    fi
    echo

    # Check parent finality and top-down status
    log_info "Parent Finality Status:"

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
        log_warn "  ✗ No parent finality commits found"
        log_info "    This is required for cross-msg fund to work!"
        echo ""

        # Diagnose why parent finality isn't working (simplified for speed)
        log_info "  Diagnosing parent finality issues..."

        # Check for vote-related activity (use simple grep, faster)
        local vote_sent=$(ssh -o StrictHostKeyChecking=no "$ssh_user@$ip" \
            "sudo su - $ipc_user -c 'grep -i PeerVoteReceived ~/.ipc-node/logs/*.log 2>/dev/null | wc -l'" 2>/dev/null | tr -d ' \n\r')
        if [ -n "$vote_sent" ] && [ "$vote_sent" -gt 0 ] 2>/dev/null; then
            log_info "    ✓ Found $vote_sent vote messages"
        else
            log_warn "    ✗ No votes being sent or received"
        fi

        # Check for resolver errors (common issue)
        local resolver_errors=$(ssh -o StrictHostKeyChecking=no "$ssh_user@$ip" \
            "sudo su - $ipc_user -c 'grep -i \"IPLD Resolver.*failed\\|Cannot assign requested address\" ~/.ipc-node/logs/*.log 2>/dev/null | wc -l'" 2>/dev/null | tr -d ' \n\r')
        if [ -n "$resolver_errors" ] && [ "$resolver_errors" -gt 0 ] 2>/dev/null; then
            log_warn "    ✗ Resolver binding errors detected ($resolver_errors occurrences)"
            log_warn "      This means libp2p cannot accept connections"
        fi
    fi
    echo

    # Show validator status summary with voting power
    log_info "Validator Status & Voting Power:"

    # Get validator set from CometBFT (from first validator)
    local validators_json=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
        "curl -s http://localhost:26657/validators 2>/dev/null" 2>/dev/null)

    local total_voting_power=0
    local validator_count=0
    if [ -n "$validators_json" ]; then
        # Calculate total voting power by summing individual powers
        total_voting_power=$(echo "$validators_json" | jq -r '[.result.validators[].voting_power | tonumber] | add' 2>/dev/null)
        validator_count=$(echo "$validators_json" | jq -r '.result.count // "0"' 2>/dev/null)

        # Fallback if calculation fails
        if [ -z "$total_voting_power" ] || [ "$total_voting_power" = "null" ]; then
            total_voting_power="0"
        fi
    fi

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

        # Get validator's voting power
        local val_power="?"
        local power_pct="?"
        if [ "$is_running" = "running" ]; then
            local val_info=$(ssh_exec "$val_ip" "$val_ssh_user" "$val_ipc_user" \
                "curl -s http://localhost:26657/status 2>/dev/null | jq -r '.result.validator_info.voting_power // \"0\"' 2>/dev/null")

            if [ -n "$val_info" ] && [ "$val_info" != "0" ] && [ "$val_info" != "" ]; then
                val_power="$val_info"
                if [ "$total_voting_power" != "0" ]; then
                    power_pct=$(echo "scale=2; ($val_power * 100) / $total_voting_power" | bc 2>/dev/null)
                fi
            fi
        fi

        if [ "$is_running" = "running" ]; then
            log_info "  ✓ $val_name: Running | Height: $val_height | Peers: $val_peers | Power: $val_power ($power_pct%)"
        else
            log_warn "  ✗ $val_name: Not running | Power: $val_power"
        fi
    done

    if [ "$total_voting_power" != "0" ]; then
        log_info ""
        log_info "  Total Voting Power: $total_voting_power (across $validator_count validators)"
        local quorum_needed=$(echo "scale=0; ($total_voting_power * 67) / 100 + 1" | bc 2>/dev/null)
        log_info "  Quorum Required: >67% (>= $quorum_needed power)"

        # Check if quorum is possible
        if [ "$validator_count" -ge 3 ]; then
            log_info "  ✓ Quorum is reachable with current validator set"

            # Check if voting power is too low (warning if < 10 per validator on average)
            local avg_power=$(echo "scale=0; $total_voting_power / $validator_count" | bc 2>/dev/null)
            if [ "$avg_power" -lt 10 ]; then
                log_warn "  ⚠ WARNING: Voting power is very low (avg: $avg_power per validator)"
                log_warn "    With this setup, if ANY validator goes offline, quorum cannot be reached!"
                log_warn "    Consider increasing power using: ipc-cli subnet set-federated-power"
            fi
        else
            log_warn "  ⚠ Only $validator_count validators - may not reach quorum!"
        fi
    fi
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

    # Get contract commitSHA values
    log_info "Contract Versions (commitSHA):"

    local parent_rpc=$(get_config_value "subnet.parent_rpc")
    local child_rpc=$(get_config_value "ipc_cli.child.provider_http")
    local parent_gateway_addr=$(get_config_value "subnet.parent_gateway")
    local parent_registry_addr=$(get_config_value "subnet.parent_registry")
    local child_gateway_addr=$(get_config_value "ipc_cli.child.gateway_addr")
    local child_registry_addr=$(get_config_value "ipc_cli.child.registry_addr")

    log_info "  Parent Contracts (RPC: $parent_rpc):"
    log_info "    Gateway ($parent_gateway_addr): $(get_contract_commit_sha "$parent_rpc" "$parent_gateway_addr")"
    log_info "    Registry ($parent_registry_addr): $(get_contract_commit_sha "$parent_rpc" "$parent_registry_addr")"

    log_info "  Child Contracts (RPC: $child_rpc):"
    log_info "    Gateway ($child_gateway_addr): $(get_contract_commit_sha "$child_rpc" "$child_gateway_addr")"
    log_info "    Registry ($child_registry_addr): $(get_contract_commit_sha "$child_rpc" "$child_registry_addr")"
    echo
}

# Watch parent finality progress in real-time
watch_parent_finality() {
    local target_epoch="${1:-}"
    local refresh_interval="${2:-5}"

    # Use first validator for monitoring
    local ip=$(get_config_value "validators[0].ip")
    local ssh_user=$(get_config_value "validators[0].ssh_user")
    local ipc_user=$(get_config_value "validators[0].ipc_user")
    local name="${VALIDATORS[0]}"

    # Get parent RPC endpoint for querying actual parent chain height
    local parent_rpc=$(get_config_value "subnet.parent_rpc")

    echo ""
    log_section "Parent Finality Monitor"
    echo ""

    if [ -n "$target_epoch" ]; then
        log_info "Monitoring until parent epoch: $target_epoch"
    else
        log_info "Monitoring parent finality progress (Ctrl+C to stop)"
    fi
    log_info "Refresh interval: ${refresh_interval}s"
    log_info "Source: $name"
    log_info "Parent RPC: $parent_rpc"
    echo ""
    echo "Time      | Iter | Subnet Finality | Parent Chain | Lag   | Subnet Height | Status"
    echo "----------|------|-----------------|--------------|-------|---------------|--------"

    local iteration=0
    local start_time=$(date +%s)

    while true; do
        iteration=$((iteration + 1))
        local current_time=$(date +%s)
        local elapsed=$((current_time - start_time))

        # Get subnet's parent finality height (what parent height the subnet has committed)
        local subnet_parent_finality=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
            "grep 'ParentFinalityCommitted' ~/.ipc-node/logs/*.log 2>/dev/null | tail -1" 2>/dev/null | \
            grep -oE 'parent_height: [0-9]+' | grep -oE '[0-9]+' || echo "0")

        # Get current parent chain block height
        local parent_chain_height=$(curl -s -X POST -H "Content-Type: application/json" \
            --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
            "$parent_rpc" 2>/dev/null | jq -r '.result // "0x0"' 2>/dev/null)

        # Convert hex to decimal
        if [[ "$parent_chain_height" == 0x* ]]; then
            parent_chain_height=$((16#${parent_chain_height#0x}))
        else
            parent_chain_height=0
        fi

        # Calculate lag between parent chain and subnet finality
        local lag=0
        if [ "$subnet_parent_finality" -gt 0 ] && [ "$parent_chain_height" -gt 0 ]; then
            lag=$((parent_chain_height - subnet_parent_finality))
        fi

        # Get current subnet block height
        local subnet_height=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
            "curl -s http://localhost:26657/status 2>/dev/null | jq -r '.result.sync_info.latest_block_height // 0' 2>/dev/null" || echo "0")

        # Calculate progress if target is set
        local status_msg=""
        if [ -n "$target_epoch" ] && [ "$subnet_parent_finality" -gt 0 ]; then
            local remaining=$((target_epoch - subnet_parent_finality))
            if [ "$remaining" -gt 0 ]; then
                status_msg="$remaining left"
            elif [ "$remaining" -eq 0 ]; then
                status_msg="✓ REACHED"
            else
                status_msg="✓ PAST"
            fi
        else
            status_msg="tracking"
        fi

        # Display current status on new line
        printf "%s | %-4d | %-15d | %-12d | %-5d | %-13d | %s\n" \
            "$(date +%H:%M:%S)" \
            "$iteration" \
            "$subnet_parent_finality" \
            "$parent_chain_height" \
            "$lag" \
            "$subnet_height" \
            "$status_msg"

        # Check if target reached
        if [ -n "$target_epoch" ] && [ "$subnet_parent_finality" -ge "$target_epoch" ]; then
            echo ""
            log_success "✓ Target epoch $target_epoch reached!"
            log_info "  Subnet parent finality: $subnet_parent_finality"
            log_info "  Parent chain height: $parent_chain_height"
            log_info "  Lag: $lag epochs"
            log_info "  Subnet block height: $subnet_height"
            log_info "  Total elapsed time: ${elapsed}s"
            echo ""
            break
        fi

        sleep "$refresh_interval"
    done

    if [ -z "$target_epoch" ]; then
        echo ""
        log_info "Monitoring stopped after $iteration iterations (${elapsed}s elapsed)"
    fi
}

# Watch block production in real-time
watch_block_production() {
    local target_height="${1:-}"
    local refresh_interval="${2:-2}"

    # Use first validator for monitoring
    local ip=$(get_config_value "validators[0].ip")
    local ssh_user=$(get_config_value "validators[0].ssh_user")
    local ipc_user=$(get_config_value "validators[0].ipc_user")
    local name="${VALIDATORS[0]}"

    echo ""
    log_section "Block Production Monitor"
    echo ""

    if [ -n "$target_height" ]; then
        log_info "Monitoring until block height: $target_height"
    else
        log_info "Monitoring block production (Ctrl+C to stop)"
    fi
    log_info "Refresh interval: ${refresh_interval}s"
    log_info "Source: $name"
    echo ""
    echo "Time      | Iter | Height  | Δ Blocks | Block Time | Blocks/s | Avg Time | Status"
    echo "----------|------|---------|----------|------------|----------|----------|--------"

    local iteration=0
    local start_time=$(date +%s)
    local prev_height=0
    local prev_time=0
    local total_blocks=0
    local cumulative_time=0

    # Get initial height
    prev_height=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
        "curl -s http://localhost:26657/status 2>/dev/null | jq -r '.result.sync_info.latest_block_height // 0' 2>/dev/null" || echo "0")
    prev_time=$(date +%s)

    while true; do
        sleep "$refresh_interval"

        iteration=$((iteration + 1))
        local current_time=$(date +%s)
        local elapsed=$((current_time - start_time))

        # Get current block height
        local current_height=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
            "curl -s http://localhost:26657/status 2>/dev/null | jq -r '.result.sync_info.latest_block_height // 0' 2>/dev/null" || echo "0")

        # Calculate metrics
        local delta_blocks=$((current_height - prev_height))
        local delta_time=$((current_time - prev_time))

        # Avoid division by zero
        if [ "$delta_time" -eq 0 ]; then
            delta_time=1
        fi

        # Calculate block time and blocks per second
        local block_time="N/A"
        local blocks_per_sec="0.00"
        if [ "$delta_blocks" -gt 0 ]; then
            block_time=$(echo "scale=2; $delta_time / $delta_blocks" | bc 2>/dev/null || echo "N/A")
            blocks_per_sec=$(echo "scale=2; $delta_blocks / $delta_time" | bc 2>/dev/null || echo "0.00")

            # Update cumulative stats
            total_blocks=$((total_blocks + delta_blocks))
            cumulative_time=$((cumulative_time + delta_time))
        fi

        # Calculate average block time
        local avg_block_time="N/A"
        if [ "$total_blocks" -gt 0 ] && [ "$cumulative_time" -gt 0 ]; then
            avg_block_time=$(echo "scale=2; $cumulative_time / $total_blocks" | bc 2>/dev/null || echo "N/A")
        fi

        # Calculate progress if target is set
        local status_msg=""
        if [ -n "$target_height" ] && [ "$current_height" -gt 0 ]; then
            local remaining=$((target_height - current_height))
            if [ "$remaining" -gt 0 ]; then
                status_msg="$remaining left"
            elif [ "$remaining" -eq 0 ]; then
                status_msg="✓ REACHED"
            else
                status_msg="✓ PAST"
            fi
        else
            if [ "$delta_blocks" -eq 0 ]; then
                status_msg="stalled"
            elif [ "$delta_blocks" -lt 0 ]; then
                status_msg="reorg?"
            else
                status_msg="producing"
            fi
        fi

        # Display current status on new line
        printf "%s | %-4d | %-7d | %-8d | %-10s | %-8s | %-8s | %s\n" \
            "$(date +%H:%M:%S)" \
            "$iteration" \
            "$current_height" \
            "$delta_blocks" \
            "${block_time}s" \
            "$blocks_per_sec" \
            "${avg_block_time}s" \
            "$status_msg"

        # Check if target reached
        if [ -n "$target_height" ] && [ "$current_height" -ge "$target_height" ]; then
            echo ""
            log_success "✓ Target height $target_height reached!"
            log_info "  Current height: $current_height"
            log_info "  Total blocks produced: $total_blocks"
            log_info "  Average block time: ${avg_block_time}s"
            log_info "  Total elapsed time: ${elapsed}s"
            echo ""
            break
        fi

        # Update previous values for next iteration
        prev_height=$current_height
        prev_time=$current_time
    done

    if [ -z "$target_height" ]; then
        echo ""
        log_info "Monitoring stopped after $iteration iterations (${elapsed}s elapsed)"
        log_info "  Total blocks observed: $total_blocks"
        if [ "$total_blocks" -gt 0 ]; then
            log_info "  Average block time: ${avg_block_time}s"
            local overall_blocks_per_sec=$(echo "scale=2; $total_blocks / $elapsed" | bc 2>/dev/null || echo "0.00")
            log_info "  Overall blocks/second: $overall_blocks_per_sec"
        fi
    fi
}

# Show consensus status across all validators
show_consensus_status() {
    echo ""
    log_section "Consensus Status"
    echo ""

    log_info "Checking consensus state across all validators..."
    echo ""
    echo "Validator      | Height | Block Hash                                                       | App Hash                                                         | Round | Step"
    echo "---------------|--------|------------------------------------------------------------------|------------------------------------------------------------------|-------|-------------"

    for idx in "${!VALIDATORS[@]}"; do
        local name="${VALIDATORS[$idx]}"
        local ip=$(get_config_value "validators[$idx].ip")
        local ssh_user=$(get_config_value "validators[$idx].ssh_user")
        local ipc_user=$(get_config_value "validators[$idx].ipc_user")

        # Get status from CometBFT
        local status=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
            "curl -s http://localhost:26657/status 2>/dev/null" || echo '{}')

        local height=$(echo "$status" | jq -r '.result.sync_info.latest_block_height // "?"' 2>/dev/null || echo "?")
        local block_hash=$(echo "$status" | jq -r '.result.sync_info.latest_block_hash // "?"' 2>/dev/null || echo "?")
        local app_hash=$(echo "$status" | jq -r '.result.sync_info.latest_app_hash // "?"' 2>/dev/null || echo "?")

        # Get consensus state
        local consensus=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
            "curl -s http://localhost:26657/consensus_state 2>/dev/null" || echo '{}')

        local round=$(echo "$consensus" | jq -r '.result.round_state.height_round_step // "?"' 2>/dev/null | cut -d'/' -f2 || echo "?")
        local step=$(echo "$consensus" | jq -r '.result.round_state.height_round_step // "?"' 2>/dev/null | cut -d'/' -f3 || echo "?")

        # Truncate hashes for display
        local block_hash_short="${block_hash:0:64}"
        local app_hash_short="${app_hash:0:64}"

        printf "%-14s | %-6s | %-64s | %-64s | %-5s | %s\n" \
            "$name" "$height" "$block_hash_short" "$app_hash_short" "$round" "$step"
    done

    echo ""

    # Check for divergence
    log_info "Checking for state divergence..."

    # Get heights and hashes
    declare -A heights
    declare -A block_hashes
    declare -A app_hashes

    for idx in "${!VALIDATORS[@]}"; do
        local name="${VALIDATORS[$idx]}"
        local ip=$(get_config_value "validators[$idx].ip")
        local ssh_user=$(get_config_value "validators[$idx].ssh_user")
        local ipc_user=$(get_config_value "validators[$idx].ipc_user")

        local status=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
            "curl -s http://localhost:26657/status 2>/dev/null" || echo '{}')

        heights[$name]=$(echo "$status" | jq -r '.result.sync_info.latest_block_height // "0"' 2>/dev/null)
        block_hashes[$name]=$(echo "$status" | jq -r '.result.sync_info.latest_block_hash // ""' 2>/dev/null)
        app_hashes[$name]=$(echo "$status" | jq -r '.result.sync_info.latest_app_hash // ""' 2>/dev/null)
    done

    # Check height divergence
    local min_height=999999999
    local max_height=0
    for height in "${heights[@]}"; do
        if [ "$height" != "0" ] && [ "$height" -lt "$min_height" ]; then
            min_height=$height
        fi
        if [ "$height" -gt "$max_height" ]; then
            max_height=$height
        fi
    done

    local height_diff=$((max_height - min_height))

    if [ "$height_diff" -gt 10 ]; then
        log_warn "⚠ Height divergence detected: $height_diff blocks apart"
        log_warn "  Min: $min_height, Max: $max_height"
    elif [ "$height_diff" -gt 0 ]; then
        log_info "  Small height difference: $height_diff blocks (normal during sync)"
    else
        log_success "  ✓ All validators at same height: $max_height"
    fi

    # Check app hash divergence at same height
    declare -A height_app_hashes
    for name in "${!heights[@]}"; do
        local h="${heights[$name]}"
        local ah="${app_hashes[$name]}"
        if [ -n "$ah" ] && [ "$ah" != "null" ]; then
            if [ -z "${height_app_hashes[$h]:-}" ]; then
                height_app_hashes[$h]="$ah"
            elif [ "${height_app_hashes[$h]}" != "$ah" ]; then
                log_error "✗ CRITICAL: App hash divergence at height $h!"
                log_error "  This indicates state machine divergence between validators"
                log_error "  One or more validators have corrupted state"
                return 1
            fi
        fi
    done

    log_success "  ✓ No app hash divergence detected"
    echo ""
}

# Show detailed voting status for current consensus round
show_voting_status() {
    echo ""
    log_section "Voting Status"
    echo ""

    log_info "Checking current consensus round voting..."
    echo ""

    # Use first validator as reference
    local ip=$(get_config_value "validators[0].ip")
    local ssh_user=$(get_config_value "validators[0].ssh_user")
    local ipc_user=$(get_config_value "validators[0].ipc_user")
    local name="${VALIDATORS[0]}"

    log_info "Source: $name"
    echo ""

    # Get consensus state
    local consensus=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
        "curl -s http://localhost:26657/consensus_state 2>/dev/null" || echo '{}')

    local height_round_step=$(echo "$consensus" | jq -r '.result.round_state.height_round_step // "?"' 2>/dev/null)
    local height=$(echo "$height_round_step" | cut -d'/' -f1)
    local round=$(echo "$height_round_step" | cut -d'/' -f2)
    local step=$(echo "$height_round_step" | cut -d'/' -f3)

    log_info "Current consensus: Height $height, Round $round, Step $step"
    echo ""

    # Get validators
    local validators=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
        "curl -s http://localhost:26657/validators 2>/dev/null" || echo '{}')

    local total_voting_power=$(echo "$validators" | jq -r '[.result.validators[].voting_power | tonumber] | add // 0' 2>/dev/null)

    log_info "Total voting power: $total_voting_power"
    log_info "Quorum required: $((total_voting_power * 2 / 3 + 1)) (>2/3)"
    echo ""

    # Get prevote and precommit info
    local prevotes=$(echo "$consensus" | jq -r '.result.round_state.height_vote_set[0].prevotes_bit_array // "?"' 2>/dev/null)
    local precommits=$(echo "$consensus" | jq -r '.result.round_state.height_vote_set[0].precommits_bit_array // "?"' 2>/dev/null)

    log_info "Prevotes:   $prevotes"
    log_info "Precommits: $precommits"
    echo ""

    # Parse vote participation
    local prevote_sum=$(echo "$prevotes" | grep -oE '[0-9]+/' | cut -d'/' -f1 || echo "0")
    local prevote_total=$(echo "$prevotes" | grep -oE '/[0-9]+ =' | tr -d '/ =' || echo "0")
    local precommit_sum=$(echo "$precommits" | grep -oE '[0-9]+/' | cut -d'/' -f1 || echo "0")
    local precommit_total=$(echo "$precommits" | grep -oE '/[0-9]+ =' | tr -d '/ =' || echo "0")

    if [ "$prevote_total" -gt 0 ]; then
        local prevote_pct=$((prevote_sum * 100 / prevote_total))
        log_info "Prevote participation: $prevote_sum/$prevote_total validators ($prevote_pct%)"
    fi

    if [ "$precommit_total" -gt 0 ]; then
        local precommit_pct=$((precommit_sum * 100 / precommit_total))
        log_info "Precommit participation: $precommit_sum/$precommit_total validators ($precommit_pct%)"
    fi

    echo ""

    # Check if consensus is stuck
    if [ "$step" = "RoundStepPrevote" ] || [ "$step" = "RoundStepPrecommit" ]; then
        log_warn "⚠ Consensus is in voting phase"
        if [ "$prevote_sum" -lt "$((prevote_total * 2 / 3))" ]; then
            log_warn "  Not enough prevotes for quorum (need $((prevote_total * 2 / 3 + 1)))"
        fi
        if [ "$precommit_sum" -lt "$((precommit_total * 2 / 3))" ]; then
            log_warn "  Not enough precommits for quorum (need $((precommit_total * 2 / 3 + 1)))"
        fi
    elif [ "$step" = "RoundStepNewHeight" ] || [ "$step" = "RoundStepPropose" ]; then
        log_success "  ✓ Consensus progressing normally"
    else
        log_info "  Step: $step"
    fi

    echo ""

    # Check recent consensus logs for issues
    log_info "Recent consensus activity (last 20 lines):"
    echo ""

    ssh_exec "$ip" "$ssh_user" "$ipc_user" \
        "tail -20 ~/.ipc-node/logs/2025-10-20.consensus.log 2>/dev/null | grep -v 'received complete proposal' | tail -10" || true

    echo ""
}

# Get address from keystore for a validator
get_validator_address_from_keystore() {
    local validator_idx="$1"

    local ip=$(get_config_value "validators[$validator_idx].ip")
    local ssh_user=$(get_config_value "validators[$validator_idx].ssh_user")
    local ipc_user=$(get_config_value "validators[$validator_idx].ipc_user")
    local ipc_config_dir=$(get_config_value "paths.ipc_config_dir")

    # Try to get address from evm_keystore.json
    # First check if it's an array or object
    local keystore_content=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
        "cat $ipc_config_dir/evm_keystore.json 2>/dev/null" 2>/dev/null)

    if [ -z "$keystore_content" ]; then
        log_warn "Could not read keystore file"
        return 1
    fi

    # Try as array first (most common), then as object
    local address=$(echo "$keystore_content" | jq -r '
        if type == "array" then
            .[0].address // .[0].Address // empty
        else
            .address // .Address // empty
        end
    ' 2>/dev/null)

    if [ -n "$address" ] && [ "$address" != "null" ]; then
        # Add 0x prefix if not present
        if [[ ! "$address" =~ ^0x ]]; then
            address="0x${address}"
        fi
        echo "$address"
        return 0
    fi

    log_warn "Could not extract address from keystore"
    return 1
}

# Start checkpoint relayer on primary validator
start_relayer() {
    log_header "Starting Checkpoint Relayer"

    # Get primary validator
    local primary_idx=$(get_primary_validator)
    local name="${VALIDATORS[$primary_idx]}"

    log_info "Starting relayer on $name (primary validator)..."

    local ip=$(get_config_value "validators[$primary_idx].ip")
    local ssh_user=$(get_config_value "validators[$primary_idx].ssh_user")
    local ipc_user=$(get_config_value "validators[$primary_idx].ipc_user")
    local node_home=$(get_config_value "paths.node_home")
    local subnet_id=$(get_config_value "subnet.id")
    local checkpoint_interval=$(get_config_value "relayer.checkpoint_interval")
    local max_parallelism=$(get_config_value "relayer.max_parallelism")

    log_info "  Subnet: $subnet_id"
    log_info "  Checkpoint interval: ${checkpoint_interval}s"
    log_info "  Max parallelism: $max_parallelism"

    # Try systemd first, fall back to nohup
    local has_systemd=$(ssh -o StrictHostKeyChecking=no "$ssh_user@$ip" \
        "systemctl list-unit-files ipc-relayer.service 2>/dev/null | grep -q ipc-relayer && echo yes || echo no" 2>/dev/null)

    if [ "$has_systemd" = "yes" ]; then
        log_info "Using systemd to start relayer..."
        ssh -o StrictHostKeyChecking=no "$ssh_user@$ip" "sudo systemctl start ipc-relayer" >/dev/null 2>&1 || true
        sleep 2

        # Check status
        local is_active=$(ssh -o StrictHostKeyChecking=no "$ssh_user@$ip" \
            "systemctl is-active ipc-relayer 2>/dev/null" | tr -d ' \n\r')

        if [ "$is_active" = "active" ]; then
            log_success "✓ Relayer started successfully via systemd"
            log_info "View logs: sudo journalctl -u ipc-relayer -f"
            log_info "Or: tail -f $node_home/logs/relayer.log"
            return 0
        else
            log_error "✗ Failed to start relayer via systemd"
            log_info "Check status: sudo systemctl status ipc-relayer"
            return 1
        fi
    else
        # Fall back to nohup
        log_info "Systemd service not found, using nohup..."

        # Get submitter address from keystore
        log_info "Extracting submitter address from keystore..."
        local submitter=$(get_validator_address_from_keystore "$primary_idx")

        if [ -z "$submitter" ]; then
            log_error "Failed to get submitter address from keystore"
            return 1
        fi

        log_info "Submitter address: $submitter"

        local ipc_binary=$(get_config_value "paths.ipc_binary")
        local relayer_log="$node_home/logs/relayer.log"

        # Ensure logs directory exists
        ssh_exec "$ip" "$ssh_user" "$ipc_user" "mkdir -p $node_home/logs" || true

        ssh_exec "$ip" "$ssh_user" "$ipc_user" \
            "nohup $ipc_binary checkpoint relayer \
            --subnet $subnet_id \
            --checkpoint-interval-sec $checkpoint_interval \
            --max-parallelism $max_parallelism \
            --submitter $submitter \
            > $relayer_log 2>&1 &"

        sleep 2

        # Verify it started
        local relayer_pid=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
            "ps aux | grep '[i]pc-cli checkpoint relayer' | grep -v grep | awk '{print \$2}' | head -1" 2>/dev/null | tr -d ' \n\r')

        if [ -n "$relayer_pid" ]; then
            log_success "✓ Relayer started successfully (PID: $relayer_pid)"
            log_info "Log file: $relayer_log"
            return 0
        else
            log_error "✗ Failed to start relayer"
            return 1
        fi
    fi
}

# Stop checkpoint relayer
stop_relayer() {
    log_header "Stopping Checkpoint Relayer"

    local primary_idx=$(get_primary_validator)
    local name="${VALIDATORS[$primary_idx]}"

    log_info "Stopping relayer on $name..."

    local ip=$(get_config_value "validators[$primary_idx].ip")
    local ssh_user=$(get_config_value "validators[$primary_idx].ssh_user")
    local ipc_user=$(get_config_value "validators[$primary_idx].ipc_user")

    # Try systemd first, fall back to manual kill
    local has_systemd=$(ssh -o StrictHostKeyChecking=no "$ssh_user@$ip" \
        "systemctl list-unit-files ipc-relayer.service 2>/dev/null | grep -q ipc-relayer && echo yes || echo no" 2>/dev/null)

    if [ "$has_systemd" = "yes" ]; then
        log_info "Using systemd to stop relayer..."
        ssh -o StrictHostKeyChecking=no "$ssh_user@$ip" "sudo systemctl stop ipc-relayer" >/dev/null 2>&1 || true
    else
        # Find and kill the relayer process by PID
        local pids=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
            "ps aux | grep '[i]pc-cli checkpoint relayer' | grep -v grep | awk '{print \$2}'" 2>/dev/null | tr '\n' ' ')

        if [ -n "$pids" ]; then
            log_info "Killing relayer process(es): $pids"
            ssh_exec "$ip" "$ssh_user" "$ipc_user" "kill $pids 2>/dev/null || true" || true
            sleep 1
            # Force kill if still running
            ssh_exec "$ip" "$ssh_user" "$ipc_user" "kill -9 $pids 2>/dev/null || true" || true
        else
            log_info "No relayer processes found"
        fi
    fi

    log_success "✓ Relayer stopped"
}

# Check relayer status
check_relayer_status() {
    log_header "Checkpoint Relayer Status"

    local primary_idx=$(get_primary_validator)
    local name="${VALIDATORS[$primary_idx]}"

    local ip=$(get_config_value "validators[$primary_idx].ip")
    local ssh_user=$(get_config_value "validators[$primary_idx].ssh_user")
    local ipc_user=$(get_config_value "validators[$primary_idx].ipc_user")

    log_info "Checking relayer on $name..."

    local node_home=$(get_config_value "paths.node_home")
    local relayer_log="$node_home/logs/relayer.log"

    # Check systemd first
    local has_systemd=$(ssh -o StrictHostKeyChecking=no "$ssh_user@$ip" \
        "systemctl list-unit-files ipc-relayer.service 2>/dev/null | grep -q ipc-relayer && echo yes || echo no" 2>/dev/null)

    if [ "$has_systemd" = "yes" ]; then
        local is_active=$(ssh -o StrictHostKeyChecking=no "$ssh_user@$ip" \
            "systemctl is-active ipc-relayer 2>/dev/null" | tr -d ' \n\r')

        if [ "$is_active" = "active" ]; then
            log_success "✓ Relayer is running (systemd)"
            log_info "Check status: sudo systemctl status ipc-relayer"
            log_info "View logs: sudo journalctl -u ipc-relayer -f"
        else
            log_warn "✗ Relayer is not running (systemd service exists but inactive)"
            log_info "Status: $is_active"
            log_info "Check with: sudo systemctl status ipc-relayer"
        fi

        # Show recent journal logs
        log_info "Recent relayer activity (from journal):"
        ssh -o StrictHostKeyChecking=no "$ssh_user@$ip" \
            "sudo journalctl -u ipc-relayer -n 20 --no-pager 2>/dev/null || echo 'No journal logs found'"
    else
        # Check for relayer process using ps
        local relayer_pid=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
            "ps aux | grep '[i]pc-cli checkpoint relayer' | grep -v grep | awk '{print \$2}' | head -1" 2>/dev/null | tr -d ' \n\r')

        if [ -n "$relayer_pid" ]; then
            log_success "✓ Relayer is running (PID: $relayer_pid)"
            log_info "Log file: $relayer_log"

            # Show recent log lines
            log_info "Recent relayer activity:"
            ssh_exec "$ip" "$ssh_user" "$ipc_user" \
                "tail -20 $relayer_log 2>/dev/null || echo 'No logs found'"
        else
            log_warn "✗ Relayer is not running"

            # Check if log file exists with any content
            local log_exists=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
                "test -f $relayer_log && echo 'yes' || echo 'no'" 2>/dev/null)

            if [ "$log_exists" = "yes" ]; then
                log_info "Last relayer output from $relayer_log:"
                ssh_exec "$ip" "$ssh_user" "$ipc_user" \
                    "tail -20 $relayer_log 2>/dev/null || echo 'Could not read log'"
            fi
        fi
    fi
}

# Get commitSHA from contract
get_contract_commit_sha() {
    local rpc_url="$1"
    local contract_address="$2"

    # Call the commitSHA() function (selector: 0x66a9f38a)
    local result=$(curl -s -X POST -H "Content-Type: application/json" \
        --data "{\"jsonrpc\":\"2.0\",\"method\":\"eth_call\",\"params\":[{\"to\":\"$contract_address\",\"data\":\"0x66a9f38a\"},\"latest\"],\"id\":1}" \
        "$rpc_url" 2>/dev/null | jq -r '.result // empty')

    if [ -n "$result" ] && [ "$result" != "null" ] && [ "$result" != "0x" ]; then
        # Decode the bytes32 result to a string
        # Remove 0x prefix and trailing zeros
        result="${result#0x}"
        # Convert hex to ASCII
        local decoded=$(echo "$result" | xxd -r -p 2>/dev/null | tr -d '\0' | strings)
        if [ -n "$decoded" ]; then
            echo "$decoded"
        else
            echo "$result"
        fi
    else
        echo "N/A"
    fi
}

