#!/bin/bash
# Anvil management functions for local mode

# Check if Anvil is running
check_anvil_running() {
    local port=$(get_config_value "deployment.anvil.port" 2>/dev/null || echo "8545")
    local rpc_url="http://localhost:${port}"

    if curl -s -X POST -H "Content-Type: application/json" \
        --data '{"jsonrpc":"2.0","method":"net_version","params":[],"id":1}' \
        "$rpc_url" > /dev/null 2>&1; then
        return 0
    else
        return 1
    fi
}

# Get Anvil chain ID
get_anvil_chain_id() {
    local port=$(get_config_value "deployment.anvil.port" 2>/dev/null || echo "8545")
    local rpc_url="http://localhost:${port}"

    local response=$(curl -s -X POST -H "Content-Type: application/json" \
        --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
        "$rpc_url")
    echo "$response" | grep -o '"result":"[^"]*"' | cut -d'"' -f4 | xargs printf "%d" 2>/dev/null || echo "0"
}

# Start Anvil
start_anvil() {
    if check_anvil_running; then
        log_info "Anvil is already running"

        # Verify chain ID matches config
        local expected_chain_id=$(get_config_value "deployment.anvil.chain_id" 2>/dev/null || echo "31337")
        local actual_chain_id=$(get_anvil_chain_id)

        if [ "$actual_chain_id" != "$expected_chain_id" ]; then
            log_warn "Anvil chain ID mismatch (expected: $expected_chain_id, actual: $actual_chain_id)"
            log_warn "Consider stopping Anvil and letting the script restart it"
        fi

        return 0
    fi

    log_section "Starting Anvil"

    # Get Anvil config
    local port=$(get_config_value "deployment.anvil.port" 2>/dev/null || echo "8545")
    local chain_id=$(get_config_value "deployment.anvil.chain_id" 2>/dev/null || echo "31337")
    local mnemonic=$(get_config_value "deployment.anvil.mnemonic" 2>/dev/null || echo "test test test test test test test test test test test junk")

    log_info "Port: $port"
    log_info "Chain ID: $chain_id"

    # Check if anvil command exists
    if ! command -v anvil &> /dev/null; then
        log_error "anvil command not found"
        log_error "Install Foundry: curl -L https://foundry.paradigm.xyz | bash && foundryup"
        exit 1
    fi

    # Start Anvil in background
    local anvil_log="/tmp/anvil-ipc-subnet.log"

    nohup anvil \
        --host 127.0.0.1 \
        --port "$port" \
        --chain-id "$chain_id" \
        --mnemonic "$mnemonic" \
        --accounts 10 \
        --block-time 1 \
        > "$anvil_log" 2>&1 &

    local anvil_pid=$!
    echo $anvil_pid > /tmp/anvil-ipc-subnet.pid

    log_info "Anvil PID: $anvil_pid"
    log_info "Log file: $anvil_log"

    # Wait for Anvil to be ready
    log_info "Waiting for Anvil to be ready..."
    local timeout=30
    while ! check_anvil_running && [ $timeout -gt 0 ]; do
        sleep 1
        timeout=$((timeout - 1))
    done

    if [ $timeout -eq 0 ]; then
        log_error "Timeout waiting for Anvil to start"
        log_error "Check logs: $anvil_log"
        return 1
    fi

    log_success "✓ Anvil started successfully"

    # Show some account info
    log_info ""
    log_info "Anvil Accounts (first 3):"
    log_info "  0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
    log_info "  0x70997970C51812dc3A010C7d01b50e0d17dc79C8"
    log_info "  0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC"
    log_info ""
}

# Stop Anvil
stop_anvil() {
    log_info "Stopping Anvil..."

    local pid_file="/tmp/anvil-ipc-subnet.pid"

    if [ -f "$pid_file" ]; then
        local pid=$(cat "$pid_file")
        if kill -0 "$pid" 2>/dev/null; then
            kill "$pid"
            log_success "✓ Anvil stopped (PID: $pid)"
        else
            log_info "Anvil process (PID: $pid) not running"
        fi
        rm -f "$pid_file"
    else
        # Try to find and kill by process name
        pkill -f "anvil.*--port" || true
        log_info "Stopped any running Anvil processes"
    fi

    # Cleanup log file
    rm -f /tmp/anvil-ipc-subnet.log
}

# Ensure Anvil is running (start if needed)
ensure_anvil_running() {
    # Check if auto-start is enabled
    local auto_start=$(get_config_value "deployment.anvil.auto_start" 2>/dev/null || echo "true")

    if [ "$auto_start" = "false" ]; then
        log_info "Anvil auto-start disabled, skipping"
        return 0
    fi

    if ! check_anvil_running; then
        start_anvil
    else
        log_info "Anvil is already running"
    fi
}

# Show Anvil status
show_anvil_status() {
    log_subsection "Anvil Status"

    if check_anvil_running; then
        local chain_id=$(get_anvil_chain_id)
        local port=$(get_config_value "deployment.anvil.port" 2>/dev/null || echo "8545")

        log_check "ok" "Running (Chain ID: $chain_id, Port: $port)"

        # Show PID if available
        local pid_file="/tmp/anvil-ipc-subnet.pid"
        if [ -f "$pid_file" ]; then
            local pid=$(cat "$pid_file")
            if kill -0 "$pid" 2>/dev/null; then
                log_info "  PID: $pid"
            fi
        fi
    else
        log_check "fail" "Not running"
    fi
}

