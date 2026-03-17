#!/bin/bash
# Health check functions

# Initialize, backup, wipe, and start functions

backup_all_nodes() {
    for idx in "${!VALIDATORS[@]}"; do
        local name="${VALIDATORS[$idx]}"
        local node_home=$(get_node_home "$idx")

        local timestamp=$(date +%Y%m%d%H%M%S)
        local backup_path="${node_home}.backup.${timestamp}"

        log_info "Creating backup for $name at $backup_path..."
        exec_on_host "$idx" \
            "if [ -d $node_home ]; then cp -r $node_home $backup_path; fi"
    done
}

wipe_all_nodes() {
    for idx in "${!VALIDATORS[@]}"; do
        local name="${VALIDATORS[$idx]}"
        local node_home=$(get_node_home "$idx")

        log_info "Wiping $name..."
        exec_on_host "$idx" "rm -rf $node_home"
    done
}

stop_all_nodes() {
    for idx in "${!VALIDATORS[@]}"; do
        local name="${VALIDATORS[$idx]}"

        log_info "Stopping $name..."
        kill_process "$idx" "ipc-cli node start"

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
    local ipc_binary=$(get_config_value "paths.ipc_binary")
    local node_home=$(get_node_home "$validator_idx")
    local resolver_port=$(get_resolver_port_for_validator "$validator_idx")
    local subnet_id=$(get_config_value "subnet.id")

    log_info "Starting $name..."

    # Use wrapper script to set env vars reliably (avoids SSH quoting issues with sudo su -c '...').
    # resolver_enabled() requires: !listen_addr.is_empty() && subnet_id != UNDEF
    local resolver_listen="/ip4/0.0.0.0/tcp/$resolver_port"
    local start_script="$node_home/start-node.sh"

    if is_local_mode; then
        # Local: run directly with env vars
        (
            export FM_RESOLVER__CONNECTION__LISTEN_ADDR="$resolver_listen"
            [ -n "$subnet_id" ] && export FM_IPC__SUBNET_ID="$subnet_id"
            nohup "$ipc_binary" node start --home "$node_home" > "$node_home/node.log" 2>&1 &
        )
    else
        # Remote: create script, copy, run (avoids ssh -c quoting of multiaddr/subnet_id)
        local tmp_script=$(mktemp)
        cat > "$tmp_script" << EOF
#!/bin/bash
export FM_RESOLVER__CONNECTION__LISTEN_ADDR="$resolver_listen"
EOF
        [ -n "$subnet_id" ] && echo "export FM_IPC__SUBNET_ID=\"$subnet_id\"" >> "$tmp_script"
        cat >> "$tmp_script" << EOF

nohup $ipc_binary node start --home $node_home > $node_home/node.log 2>&1 &
EOF
        copy_to_host "$validator_idx" "$tmp_script" "$start_script"
        rm -f "$tmp_script"
        exec_on_host "$validator_idx" "chmod +x $start_script && $start_script"
    fi
}

initialize_primary_node() {
    local validator_idx="$1"

    local name="${VALIDATORS[$validator_idx]}"
    local ipc_binary=$(get_config_value "paths.ipc_binary")
    local node_init_config=$(get_config_value "paths.node_init_config")

    log_info "Initializing $name (primary)..."

    # Generate node-init.yml
    local temp_config="/tmp/node-init-${name}.yml"
    generate_node_init_yml "$validator_idx" "$temp_config" ""

    # Copy to target location (handles local/remote automatically)
    copy_to_host "$validator_idx" "$temp_config" "$node_init_config"
    rm -f "$temp_config"

    # Run init
    local init_output=$(exec_on_host "$validator_idx" \
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
    local ipc_user=$(get_config_value "validators[$validator_idx].ipc_user")
    local ipc_binary=$(get_config_value "paths.ipc_binary")
    local node_init_config=$(get_config_value "paths.node_init_config")

    log_info "Initializing $name..."

    # Copy primary's peer-info.json to secondary as peer1.json
    if [ -n "$primary_peer_info" ]; then
        local temp_peer_file="/tmp/peer1-${name}.json"
        echo "$primary_peer_info" > "$temp_peer_file"
        copy_to_host "$validator_idx" "$temp_peer_file" "/home/$ipc_user/peer1.json"
        rm -f "$temp_peer_file"
    fi

    # Generate node-init.yml with peer file reference
    local temp_config="/tmp/node-init-${name}.yml"
    local peer_file_path=""
    if [ -n "$primary_peer_info" ]; then
        peer_file_path="/home/$ipc_user/peer1.json"
    fi
    generate_node_init_yml "$validator_idx" "$temp_config" "$peer_file_path"

    # Copy to target location (handles local/remote automatically)
    copy_to_host "$validator_idx" "$temp_config" "$node_init_config"
    rm -f "$temp_config"

    # Run init
    local init_output=$(exec_on_host "$validator_idx" \
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

    local output=$(exec_on_host "$primary_idx" "$cmd 2>&1")

    if echo "$output" | grep -q "Error\|error\|failed"; then
        log_error "Failed to set federated power"
        echo "$output"
    else
        log_success "Federated power configured"
    fi
}

# Deploy subnet with gateway contracts using ipc-cli subnet init
deploy_subnet() {
    # All logs go to stderr, only subnet ID goes to stdout for capture
    log_info "Deploying subnet with gateway contracts..." >&2

    local ipc_binary=$(get_config_value "paths.ipc_binary")
    local ipc_binary_expanded="${ipc_binary/#\~/$HOME}"
    # Deploy runs locally - use ipc-cli from PATH if config path is for remote host
    if ! is_local_mode && [[ "$ipc_binary_expanded" == /home/* ]] && [ ! -x "$ipc_binary_expanded" ]; then
        ipc_binary_expanded=$(command -v ipc-cli 2>/dev/null || echo "")
        if [ -z "$ipc_binary_expanded" ] || [ ! -x "$ipc_binary_expanded" ]; then
            log_error "ipc-cli not found. Install locally: cd ipc && cargo build --release, or add to PATH. Config paths.ipc_binary is for remote hosts." >&2
            exit 1
        fi
        log_info "Using local ipc-cli: $ipc_binary_expanded" >&2
    fi
    local parent_rpc=$(get_config_value "subnet.parent_rpc")
    local parent_chain_id=$(get_config_value "subnet.parent_chain_id")

    # Get validator information
    local validator_count=${#VALIDATORS[@]}
    local primary_validator_idx=$(get_primary_validator)
    local primary_private_key=$(get_config_value "validators[$primary_validator_idx].private_key")

    # Extract Ethereum address from private key
    local from_address=$(yq eval ".validators[$primary_validator_idx].address // null" "$CONFIG_FILE")

    # If no address in config, derive from known Anvil keys or use cast
    if [ "$from_address" = "null" ] || [ -z "$from_address" ]; then
        case "$primary_private_key" in
            "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80")
                from_address="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
                ;;
            "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d")
                from_address="0x70997970C51812dc3A010C7d01b50e0d17dc79C8"
                ;;
            "0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a")
                from_address="0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC"
                ;;
            *)
                from_address=$(cast wallet address --private-key "$primary_private_key" 2>/dev/null)
                if [ -z "$from_address" ]; then
                    log_error "Cannot derive address from private key. Install Foundry (cast) or add 'address' field to validator config." >&2
                    exit 1
                fi
                log_info "Derived address from private key: $from_address" >&2
                ;;
        esac
    fi

    log_info "Generating subnet-init.yaml configuration..." >&2

    # Get configuration values
    local permission_mode=$(get_config_value "init.permission_mode")
    local supply_source=$(get_config_value "init.subnet_supply_source_kind")
    local min_validators=$(get_config_value "init.min_validators" 2>/dev/null)
    if [ -z "$min_validators" ] || [ "$min_validators" = "null" ]; then
        min_validators=$validator_count
    fi
    local activate_subnet=$(get_config_value "init.activate_subnet" 2>/dev/null || echo "true")

    # Get subnet chain ID from config, or generate a unique one
    local subnet_chain_id=$(get_config_value "subnet.chain_id" 2>/dev/null)
    if [ -z "$subnet_chain_id" ] || [ "$subnet_chain_id" = "null" ]; then
        # Generate unique chain ID based on timestamp (milliseconds since epoch mod 2^32)
        local parent_num=$(echo "$parent_chain_id" | sed 's/\/r//')
        subnet_chain_id=$((parent_num + 1000 + ($(date +%s) % 10000)))
        log_warn "No subnet.chain_id configured, generated: $subnet_chain_id" >&2
    else
        log_info "Using configured subnet chain ID: $subnet_chain_id" >&2
    fi

    # Create subnet-init.yaml
    local subnet_init_config="/tmp/subnet-init-$$.yaml"

    cat > "$subnet_init_config" << EOF
import-wallets:
  - wallet-type: evm
    private-key: $primary_private_key

deploy:
  enabled: true
  url: $parent_rpc
  from: $from_address
  chain-id: $(echo "$parent_chain_id" | sed 's/\/r//')

create:
  parent: $parent_chain_id
  from: $from_address
  chain-id: $subnet_chain_id
  min-validator-stake: 1.0
  min-validators: $min_validators
  bottomup-check-period: 50
  permission-mode: $permission_mode
  supply-source-kind: $supply_source
  min-cross-msg-fee: 0.000001
  genesis-subnet-ipc-contracts-owner: $from_address
EOF

    # Add activation section if enabled
    if [ "$activate_subnet" = "true" ]; then
        cat >> "$subnet_init_config" << EOF

activate:
  mode: $permission_mode
  from: $from_address
EOF

        # Add validator configuration based on permission mode
        if [ "$permission_mode" = "collateral" ]; then
            cat >> "$subnet_init_config" << EOF
  validators:
EOF
            for idx in "${!VALIDATORS[@]}"; do
                local val_address=$(yq eval ".validators[$idx].address // null" "$CONFIG_FILE")
                local val_private_key=$(yq eval ".validators[$idx].private_key" "$CONFIG_FILE")

                if [ "$val_address" = "null" ] || [ -z "$val_address" ]; then
                    case "$val_private_key" in
                        "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80")
                            val_address="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
                            ;;
                        "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d")
                            val_address="0x70997970C51812dc3A010C7d01b50e0d17dc79C8"
                            ;;
                        "0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a")
                            val_address="0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC"
                            ;;
                        *)
                            val_address=$(cast wallet address --private-key "$val_private_key" 2>/dev/null)
                            ;;
                    esac
                fi

                cat >> "$subnet_init_config" << EOF
    - from: "$val_address"
      collateral: 1.0
      initial-balance: 10.0
EOF
            done
        else
            # For federated/static mode, derive public keys
            local pubkeys=()
            local powers=()

            for idx in "${!VALIDATORS[@]}"; do
                local val_private_key=$(yq eval ".validators[$idx].private_key" "$CONFIG_FILE")
                local pubkey_raw=$(cast wallet pubkey --private-key "$val_private_key" 2>/dev/null)

                if [ -z "$pubkey_raw" ]; then
                    log_error "Failed to derive public key from private key for validator $idx" >&2
                    exit 1
                fi

                local pubkey="0x04${pubkey_raw#0x}"
                pubkeys+=("$pubkey")
                powers+=(100)
            done

            cat >> "$subnet_init_config" << EOF
  validator-pubkeys:
EOF
            for pubkey in "${pubkeys[@]}"; do
                cat >> "$subnet_init_config" << EOF
    - "$pubkey"
EOF
            done

            cat >> "$subnet_init_config" << EOF
  validator-power:
EOF
            for power in "${powers[@]}"; do
                cat >> "$subnet_init_config" << EOF
    - $power
EOF
            done
        fi
    fi

    # Run subnet init
    log_info "Running ipc-cli subnet init..." >&2
    log_info "This will deploy gateway contracts, create the subnet, and generate genesis files..." >&2

    local init_output
    init_output=$($ipc_binary_expanded subnet init --config "$subnet_init_config" 2>&1)
    local exit_code=$?

    if [ $exit_code -ne 0 ]; then
        log_error "Subnet deployment failed" >&2
        echo "" >&2
        echo "Error output:" >&2
        echo "$init_output" >&2
        echo "" >&2
        log_info "Troubleshooting: For Calibration, fund $from_address with test FIL. Ensure ipc-cli is in PATH." >&2
        rm -f "$subnet_init_config"
        exit 1
    fi

    # Extract subnet ID from config (use local path when deploy runs locally)
    local ipc_config_dir
    ipc_config_dir=$(get_local_ipc_config_dir)
    local ipc_config_file="$ipc_config_dir/config.toml"

    local subnet_id=$(grep '^id = ' "$ipc_config_file" | cut -d'"' -f2 | grep -E "^$parent_chain_id/t[a-z0-9]+" | head -1)

    if [ -z "$subnet_id" ]; then
        log_error "Could not extract subnet ID from IPC config at $ipc_config_file" >&2
        log_info "Full CLI output:" >&2
        echo "$init_output" >&2
        rm -f "$subnet_init_config"
        exit 1
    fi

    log_success "Subnet deployed successfully: $subnet_id" >&2

    # Update config with new subnet ID
    log_info "Updating configuration with new subnet ID..." >&2
    yq eval ".subnet.id = \"$subnet_id\"" -i "$CONFIG_FILE"

    log_info "✅ Subnet deployment complete!" >&2
    log_info "   Subnet ID: $subnet_id" >&2
    log_info "   Genesis files generated in ~/.ipc/" >&2
    log_info "   IPC config updated at ~/.ipc/config.toml" >&2

    # Clean up
    rm -f "$subnet_init_config"

    # Return subnet ID with marker
    echo "SUBNET_ID:$subnet_id"
}

# Create bootstrap genesis for non-activated subnets (Anvil/local development)
# When ipc-cli subnet create-genesis fails (e.g. FunctionNotFound on Calibration),
# fall back to fendermint genesis commands directly (no parent chain fetch).
create_bootstrap_genesis() {
    local subnet_id="$1"

    log_info "Creating bootstrap genesis for non-activated subnet..."

    local ipc_config_dir
    ipc_config_dir=$(get_local_ipc_config_dir)

    local ipc_binary=$(get_config_value "paths.ipc_binary")
    local ipc_binary_expanded="${ipc_binary/#\~/$HOME}"
    if ! is_local_mode && [[ "$ipc_binary_expanded" == /home/* ]] && [ ! -x "$ipc_binary_expanded" ]; then
        ipc_binary_expanded=$(command -v ipc-cli 2>/dev/null || echo "")
    fi
    if [ -z "$ipc_binary_expanded" ] || [ ! -x "$ipc_binary_expanded" ]; then
        log_error "ipc-cli not found. Install: cd ipc && cargo build --release"
        return 1
    fi

    # Try ipc-cli subnet create-genesis first (fetches from parent chain)
    log_info "Generating genesis files..."
    local genesis_output
    genesis_output=$($ipc_binary_expanded subnet create-genesis --subnet "$subnet_id" 2>&1)
    local exit_code=$?

    if [ $exit_code -eq 0 ]; then
        log_success "Genesis files created successfully"
        return 0
    fi

    # Fallback: parent chain fetch failed (e.g. subnet not activated, FunctionNotFound on Calibration)
    # Use fendermint genesis commands directly - no parent chain required
    if echo "$genesis_output" | grep -qi "FunctionNotFound\|failed to create\|reverted with error\|does not exist"; then
        log_warn "Parent chain fetch failed, using fendermint bootstrap genesis (no parent chain)"
        if create_bootstrap_genesis_fendermint "$subnet_id"; then
            log_success "Bootstrap genesis created successfully"
            return 0
        fi
    fi

    log_error "Genesis creation failed"
    echo "$genesis_output"
    return 1
}

# Create genesis using fendermint directly (no parent chain - for Calibration FunctionNotFound workaround)
create_bootstrap_genesis_fendermint() {
    local subnet_id="$1"

    # Subnet IDs with t-prefix addresses (e.g. /t410...) are Filecoin testnet; fendermint must use --network testnet
    local fendermint_network=""
    if [[ "$subnet_id" == *"/t"* ]]; then
        fendermint_network="--network testnet"
    fi

    local fendermint_bin
    fendermint_bin=$(command -v fendermint 2>/dev/null || echo "")
    if [ -z "$fendermint_bin" ]; then
        log_error "fendermint not found in PATH. Install with: cd ipc && cargo build --release"
        return 1
    fi

    local ipc_config_dir
    ipc_config_dir=$(get_local_ipc_config_dir)
    local subnet_id_no_slash="${subnet_id#/}"
    local genesis_json="$ipc_config_dir/genesis_${subnet_id_no_slash//\//_}.json"
    local genesis_sealed="$ipc_config_dir/genesis_sealed_${subnet_id_no_slash//\//_}.json"

    local base_fee=$(get_config_value_with_default "init.genesis.base_fee" "1000")
    local power_scale=$(get_config_value_with_default "init.genesis.power_scale" "3")
    local network_version=$(get_config_value_with_default "init.genesis.network_version" "21")

    # Get primary validator address for ipc-contracts-owner
    local primary_idx
    primary_idx=$(get_primary_validator)
    local from_address
    from_address=$(yq eval ".validators[$primary_idx].address // null" "$CONFIG_FILE")
    local primary_private_key
    primary_private_key=$(get_config_value "validators[$primary_idx].private_key")
    if [ "$from_address" = "null" ] || [ -z "$from_address" ]; then
        from_address=$(cast wallet address --private-key "$primary_private_key" 2>/dev/null || echo "0x0000000000000000000000000000000000000000")
    fi

    local chain_name="${subnet_id_no_slash//\//_}"
    local timestamp
    timestamp=$(date +%s)

    mkdir -p "$ipc_config_dir"

    log_info "Creating genesis with fendermint..."
    $fendermint_bin $fendermint_network genesis --genesis-file "$genesis_json" new \
        --timestamp "$timestamp" \
        --chain-name "$chain_name" \
        --network-version "$network_version" \
        --base-fee "$base_fee" \
        --power-scale "$power_scale" \
        --ipc-contracts-owner "$from_address" 2>&1 || return 1

    if [ ! -f "$genesis_json" ]; then
        log_error "Failed to create genesis file"
        return 1
    fi

    # Set IPC gateway params (bottom-up disabled for federated - use large period)
    $fendermint_bin $fendermint_network genesis --genesis-file "$genesis_json" ipc gateway \
        --subnet-id "$subnet_id" \
        --bottom-up-check-period 10000 \
        --msg-fee 1000 \
        --majority-percentage 51 2>&1 || return 1

    # Add validators and accounts
    for idx in "${!VALIDATORS[@]}"; do
        local val_private_key
        val_private_key=$(yq eval ".validators[$idx].private_key" "$CONFIG_FILE")
        local val_address
        val_address=$(yq eval ".validators[$idx].address // null" "$CONFIG_FILE")
        if [ "$val_address" = "null" ] || [ -z "$val_address" ]; then
            val_address=$(cast wallet address --private-key "$val_private_key" 2>/dev/null)
        fi

        local pubkey_raw
        pubkey_raw=$(cast wallet pubkey --private-key "$val_private_key" 2>/dev/null)
        local pubkey_hex="04${pubkey_raw#0x}"
        local pubkey_file="/tmp/validator_${idx}_pubkey_b64.txt"
        echo -n "$pubkey_hex" | xxd -r -p | base64 | tr -d '\n' > "$pubkey_file"

        log_info "Adding validator ${VALIDATORS[$idx]}..."
        $fendermint_bin $fendermint_network genesis --genesis-file "$genesis_json" add-validator \
            --public-key "$pubkey_file" \
            --power 100 2>&1 || true

        $fendermint_bin $fendermint_network genesis --genesis-file "$genesis_json" add-account \
            --public-key "$pubkey_file" \
            --balance "1000" \
            --kind ethereum 2>&1 || true

        rm -f "$pubkey_file" 2>/dev/null
    done

    # Seal genesis
    log_info "Sealing genesis..."
    $fendermint_bin $fendermint_network genesis --genesis-file "$genesis_json" ipc seal-genesis \
        --output-path "$genesis_sealed" 2>&1 || return 1

    if [ ! -f "$genesis_sealed" ]; then
        log_error "Failed to seal genesis"
        return 1
    fi

    return 0
}

# Copy genesis files from local to all remote validators (no-op for local mode)
copy_genesis_to_remotes() {
    local subnet_id="$1"

    if is_local_mode; then
        return 0
    fi

    local local_ipc_dir
    local_ipc_dir=$(get_local_ipc_config_dir)
    local subnet_id_no_slash="${subnet_id#/}"
    local genesis_json="$local_ipc_dir/genesis_${subnet_id_no_slash//\//_}.json"
    local genesis_sealed="$local_ipc_dir/genesis_sealed_${subnet_id_no_slash//\//_}.json"

    if [ ! -f "$genesis_json" ] || [ ! -f "$genesis_sealed" ]; then
        return 0
    fi

    log_info "Copying genesis files to remote validators..."
    local remote_ipc_dir
    remote_ipc_dir=$(get_config_value "paths.ipc_config_dir")

    for idx in "${!VALIDATORS[@]}"; do
        local name="${VALIDATORS[$idx]}"
        log_info "Copying genesis to $name..."
        exec_on_host "$idx" "mkdir -p $remote_ipc_dir"
        copy_to_host "$idx" "$genesis_json" "$remote_ipc_dir/$(basename "$genesis_json")"
        copy_to_host "$idx" "$genesis_sealed" "$remote_ipc_dir/$(basename "$genesis_sealed")"
        log_success "Genesis copied to $name"
    done
}

# Detailed diagnostics for a validator (troubleshooting health check failures)
diagnose_validator() {
    local validator_idx="$1"

    local name="${VALIDATORS[$validator_idx]}"
    local node_home=$(get_node_home "$validator_idx")
    local cometbft_port=$(get_config_value "network.cometbft_p2p_port")
    local libp2p_port=$(get_resolver_port_for_validator "$validator_idx")
    local eth_api_port=$(get_config_value "network.eth_api_port")

    log_header "Diagnostics: $name"
    echo ""
    log_info "Expected ports: cometbft_p2p=$cometbft_port, libp2p=$libp2p_port, eth_api=$eth_api_port"
    log_info "CometBFT RPC (for status/net_info): 26657"
    echo ""

    log_subsection "Listening ports"
    exec_on_host_simple "$validator_idx" \
        "netstat -an 2>/dev/null | grep LISTEN | head -25 || ss -tuln 2>/dev/null | head -25 || echo 'netstat/ss not found'" || true
    echo ""

    log_subsection "Expected ports check"
    local ports_found
    ports_found=$(exec_on_host_simple "$validator_idx" \
        "netstat -an 2>/dev/null | grep LISTEN | grep -E \"[\.:]$cometbft_port|[\.:]$libp2p_port|[\.:]$eth_api_port\" || true")
    if [ -n "$ports_found" ]; then
        echo "$ports_found"
    else
        log_warn "None of the expected ports ($cometbft_port, $libp2p_port, $eth_api_port) found listening"
        log_info "Trying 'ss' as fallback..."
        exec_on_host_simple "$validator_idx" \
            "ss -tuln 2>/dev/null | grep -E \"26656|26657|26654|26655|8545\" || true" || true
    fi
    echo ""

    log_subsection "CometBFT RPC (localhost:26657)"
    local status
    status=$(exec_on_host_simple "$validator_idx" \
        "curl -s --max-time 5 http://localhost:26657/status 2>/dev/null" || echo "{}")
    if echo "$status" | jq -e '.result.sync_info' >/dev/null 2>&1; then
        log_info "Block height: $(echo "$status" | jq -r '.result.sync_info.latest_block_height // "?"')"
        log_info "Catching up: $(echo "$status" | jq -r '.result.sync_info.catching_up // "?"')"
    else
        log_warn "CometBFT RPC not responding - node may still be starting"
        echo "$status" | head -3
    fi
    echo ""

    log_subsection "CometBFT peers (net_info)"
    local net_info
    net_info=$(exec_on_host_simple "$validator_idx" \
        "curl -s --max-time 5 http://localhost:26657/net_info 2>/dev/null" || echo "{}")
    if echo "$net_info" | jq -e '.result.n_peers' >/dev/null 2>&1; then
        log_info "Peers: $(echo "$net_info" | jq -r '.result.n_peers // "?"')"
    else
        log_warn "Could not get net_info"
    fi
    echo ""

    log_subsection "Recent node log (last 15 lines)"
    exec_on_host_simple "$validator_idx" \
        "tail -15 $node_home/node.log 2>/dev/null || tail -15 $node_home/logs/*.log 2>/dev/null || echo 'No logs found'" || true
    echo ""

    log_subsection "Fendermint config (ABCI port 26658)"
    local abci_config
    abci_config=$(exec_on_host_simple "$validator_idx" \
        "grep -r 26658 $node_home/fendermint/config/ 2>/dev/null || echo \"Config not found\"" || true)
    if echo "$abci_config" | grep -q "26658"; then
        log_info "ABCI port 26658 configured"
    else
        log_warn "ABCI port 26658 may not be configured - Fendermint must listen here for CometBFT"
        echo "$abci_config" | head -5
    fi
    echo ""

    log_info "Root cause: CometBFT cannot connect to Fendermint ABCI at 127.0.0.1:26658"
    log_info "  Fendermint provides the ABCI app; CometBFT needs it before starting RPC."
    log_info ""
    log_info "Suggestions:"
    log_info "  1. Run update-config to fix Fendermint/CometBFT config: ./ipc-manager update-config"
    log_info "  2. If nodes were initialized before recent fixes, re-run init: ./ipc-manager init"
    log_info "  3. Check full logs for Fendermint errors: ./ipc-manager logs $name"
    log_info "  4. Restart after config fix: ./ipc-manager restart --yes"
}

# Health check for single validator
check_validator_health() {
    local validator_idx="$1"

    local name="${VALIDATORS[$validator_idx]}"
    local node_home=$(get_node_home "$validator_idx")
    local cometbft_port=$(get_config_value "network.cometbft_p2p_port")
    local libp2p_port=$(get_resolver_port_for_validator "$validator_idx")
    local eth_api_port=$(get_config_value "network.eth_api_port")

    local healthy=true

    # Check process running
    if check_process_running "$validator_idx" "ipc-cli node start"; then
        log_check "ok" "Process running"
    else
        log_check "fail" "Process not running"
        healthy=false
    fi

    # Check ports listening
    # Note: macOS netstat uses . as separator (e.g., *.8546), Linux uses : (e.g., *:8546)
    local ports_check=$(exec_on_host_simple "$validator_idx" \
        "netstat -an 2>/dev/null | grep LISTEN | grep -E \"[\.:]$cometbft_port|[\.:]$libp2p_port|[\.:]$eth_api_port\" | wc -l")

    if [ -n "$ports_check" ] && [ "$ports_check" -ge 2 ] 2>/dev/null; then
        log_check "ok" "Ports listening ($ports_check/3)"
    else
        log_check "fail" "Ports not listening (${ports_check:-0}/3) [expected: $cometbft_port, $libp2p_port, $eth_api_port]"
        [ "$DEBUG" = true ] && log_info "  Run 'diagnose' for details: ./ipc-manager diagnose $name"
        healthy=false
    fi

    # Check CometBFT peers
    local comet_peers=$(exec_on_host_simple "$validator_idx" \
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
    local block_height=$(exec_on_host_simple "$validator_idx" \
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
    local recent_errors=$(exec_on_host_simple "$validator_idx" \
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

    log_info "Measuring block time for $name (sampling for ${sample_duration}s)..."

    # Get initial block height and timestamp
    local initial_height=$(exec_on_host "$validator_idx" \
        "curl -s http://localhost:26657/status 2>/dev/null | jq -r '.result.sync_info.latest_block_height // 0' 2>/dev/null")
    local initial_time=$(exec_on_host "$validator_idx" \
        "curl -s http://localhost:26657/status 2>/dev/null | jq -r '.result.sync_info.latest_block_time // \"\"' 2>/dev/null")

    if [ -z "$initial_height" ] || [ "$initial_height" = "0" ] || [ "$initial_height" = "null" ] || [ -z "$initial_time" ] || [ "$initial_time" = "null" ]; then
        log_warn "Could not get initial block data from $name"
        return 1
    fi

    log_info "  Initial: Block #$initial_height at $initial_time"

    # Wait for the sample duration
    sleep "$sample_duration"

    # Get final block height and timestamp
    local final_height=$(exec_on_host "$validator_idx" \
        "curl -s http://localhost:26657/status 2>/dev/null | jq -r '.result.sync_info.latest_block_height // 0' 2>/dev/null")
    local final_time=$(exec_on_host "$validator_idx" \
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
# In remote mode: curl directly to validator IP (eth API may not be reachable via SSH/localhost)
# In local mode: curl localhost via exec_on_host
get_chain_id() {
    local validator_idx="${1:-0}"

    local eth_api_port=$(get_config_value "network.eth_api_port")
    local rpc_url
    local response

    if is_local_mode; then
        # Local mode: curl localhost on the validator
        response=$(exec_on_host "$validator_idx" \
            "curl -s -X POST -H 'Content-Type: application/json' --data '{\"jsonrpc\":\"2.0\",\"method\":\"eth_chainId\",\"params\":[],\"id\":1}' http://localhost:${eth_api_port}" 2>/dev/null)
    else
        # Remote mode: curl directly to validator's external IP (same path cast/wallets use)
        local ip=$(get_config_value "validators[$validator_idx].ip")
        rpc_url="http://${ip}:${eth_api_port}"
        response=$(curl -s --max-time 5 -X POST -H "Content-Type: application/json" \
            --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
            "$rpc_url" 2>/dev/null)
    fi

    local chain_id=$(echo "$response" | jq -r '.result // ""' 2>/dev/null)

    echo "$chain_id"
}

# Show comprehensive subnet information
show_subnet_info() {
    log_header "Subnet Information"

    # Get config values
    local subnet_id=$(get_config_value "subnet.id")
    local parent_chain_id=$(get_config_value "subnet.parent_chain_id")
    local parent_registry=$(get_config_value "subnet.parent_registry")
    local parent_gateway=$(get_config_value "subnet.parent_gateway")
    local num_validators=${#VALIDATORS[@]}

    echo
    log_info "Network Configuration:"
    log_info "  Subnet ID: $subnet_id"
    log_info "  Parent Chain: $parent_chain_id"
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

    # Get chain IDs
    log_info "Chain IDs:"

    # Parent chain ID (from config)
    if [ -n "$parent_chain_id" ] && [ "$parent_chain_id" != "null" ]; then
        # Extract numeric chain ID from /r<number> format
        local parent_chain_num=$(echo "$parent_chain_id" | sed 's/\/r//')
        log_info "  Parent Chain ID: $parent_chain_num (from config: $parent_chain_id)"

        # Query parent chain's actual eth_chainId
        local parent_rpc=$(get_config_value "subnet.parent_rpc")
        if [ -n "$parent_rpc" ]; then
            local parent_eth_chain_id=$(curl -s -X POST -H "Content-Type: application/json" \
                --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
                "$parent_rpc" 2>/dev/null | jq -r '.result // ""' 2>/dev/null)

            if [ -n "$parent_eth_chain_id" ] && [ "$parent_eth_chain_id" != "null" ]; then
                if [[ "$parent_eth_chain_id" == 0x* ]]; then
                    local parent_eth_chain_id_dec=$((parent_eth_chain_id))
                    log_info "  Parent eth_chainId (via RPC): $parent_eth_chain_id (decimal: $parent_eth_chain_id_dec)"
                fi
            fi
        fi
    fi

    # Subnet's eth_chainId (from querying the subnet's RPC)
    local eth_api_port=$(get_config_value "network.eth_api_port")
    log_info "  Querying subnet's eth_chainId from ${VALIDATORS[0]} (port $eth_api_port)..."
    local subnet_chain_id=$(get_chain_id 0)

    if [ -n "$subnet_chain_id" ] && [ "$subnet_chain_id" != "null" ] && [ "$subnet_chain_id" != "" ]; then
        # Convert hex to decimal if it starts with 0x
        if [[ "$subnet_chain_id" == 0x* ]]; then
            local subnet_chain_id_dec=$((subnet_chain_id))
            log_info "  Subnet eth_chainId (via RPC): $subnet_chain_id (decimal: $subnet_chain_id_dec)"

            # Warn if they're the same
            if [ "$subnet_chain_id_dec" = "$parent_chain_num" ]; then
                log_warn "  ⚠ Subnet and parent have the same eth_chainId ($subnet_chain_id_dec)"
                log_warn "    This is common in local dev but may cause issues in production"
            fi
        else
            log_info "  Subnet eth_chainId (via RPC): $subnet_chain_id"
        fi
    else
        log_warn "  Could not fetch subnet eth_chainId"
    fi
    echo

    # Get current block info from first validator
    log_info "Current Block Information (from ${VALIDATORS[0]}):"

    local block_height=$(exec_on_host 0 \
        "curl -s http://localhost:26657/status 2>/dev/null | jq -r '.result.sync_info.latest_block_height // \"\"' 2>/dev/null")
    local block_time=$(exec_on_host 0 \
        "curl -s http://localhost:26657/status 2>/dev/null | jq -r '.result.sync_info.latest_block_time // \"\"' 2>/dev/null")
    local catching_up=$(exec_on_host 0 \
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
    local n_peers=$(exec_on_host 0 \
        "curl -s http://localhost:26657/net_info 2>/dev/null | jq -r '.result.n_peers // 0' 2>/dev/null")
    local listening=$(exec_on_host 0 \
        "curl -s http://localhost:26657/net_info 2>/dev/null | jq -r '.result.listening // false' 2>/dev/null")

    log_info "  CometBFT Peers: $n_peers"
    log_info "  CometBFT Listening: $listening"
    echo

    # Check critical infrastructure for parent finality voting
    log_info "Libp2p Infrastructure (required for voting):"
    # Use get_resolver_port_for_validator - node init uses libp2p_port-1 for resolver (port_offset pattern)
    local libp2p_port=$(get_resolver_port_for_validator 0)

    # Check if libp2p port is listening and on correct address
    local libp2p_listening=$(exec_on_host 0 \
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
    local node_home=$(get_node_home 0)
    local resolver_enabled=$(exec_on_host 0 \
        "grep -A3 \"\\[resolver\\]\" $node_home/fendermint/config/default.toml | grep enabled | grep -o \"true\\|false\"" 2>/dev/null | head -1 | tr -d '\n\r ')

    if [ "$resolver_enabled" = "true" ]; then
        log_info "  ✓ Resolver enabled in config"

        # Check if resolver service started
        local resolver_started=$(exec_on_host 0 \
            "grep \"starting the IPLD Resolver Service\" $node_home/logs/*.log 2>/dev/null | wc -l" 2>/dev/null | tr -d ' \n\r')

        if [ -n "$resolver_started" ] && [ "$resolver_started" -gt 0 ] 2>/dev/null; then
            log_info "  ✓ Resolver service started ($resolver_started times)"

            # Check if vote gossip loop started
            local vote_loop=$(exec_on_host 0 \
                "grep \"parent finality vote gossip loop\" $node_home/logs/*.log 2>/dev/null | wc -l" 2>/dev/null | tr -d ' \n\r')

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
    local listen_addr=$(exec_on_host 0 \
        "grep 'listen_addr' $node_home/fendermint/config/default.toml 2>/dev/null | head -1" 2>/dev/null)

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
        local v_peer_ip=$(get_peer_ip "$idx")
        local v_resolver_port=$(get_resolver_port_for_validator "$idx")
        local v_node_home=$(get_node_home "$idx")

        log_info "  $v_name ($v_ip):"

        # Get external_addresses (config uses peer_ip/internal_ip for VPC, resolver_port for actual port)
        local ext_addrs=$(exec_on_host "$idx" \
            "grep external_addresses $v_node_home/fendermint/config/default.toml 2>/dev/null" 2>/dev/null)

        if [ -n "$ext_addrs" ] && echo "$ext_addrs" | grep -q "/ip4/$v_peer_ip/tcp/$v_resolver_port"; then
            log_info "    ✓ external_addresses: Contains peer address ($v_peer_ip:$v_resolver_port)"
        elif [ -n "$ext_addrs" ]; then
            log_warn "    ✗ external_addresses: $(echo "$ext_addrs" | cut -c1-80)"
            log_warn "      Expected to contain: /ip4/$v_peer_ip/tcp/$v_resolver_port"
        else
            log_warn "    ✗ external_addresses: Not set or empty"
        fi

        # Get static_addresses
        local static_addrs=$(exec_on_host "$idx" \
            "grep static_addresses $v_node_home/fendermint/config/default.toml 2>/dev/null" 2>/dev/null)

        if [ -n "$static_addrs" ]; then
            # Count how many peer IPs are in static_addresses (use get_peer_ip for VPC internal IPs)
            local peer_count=0
            for peer_idx in "${!VALIDATORS[@]}"; do
                if [ "$peer_idx" != "$idx" ]; then
                    local peer_ip=$(get_peer_ip "$peer_idx")
                    if echo "$static_addrs" | grep -q "/ip4/$peer_ip/tcp/$v_resolver_port"; then
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
        local libp2p_connections=$(exec_on_host "$idx" \
            "ss -tn | grep :$v_resolver_port | grep ESTAB | wc -l" 2>/dev/null | tr -d ' \n\r')

        if [ -n "$libp2p_connections" ] && [ "$libp2p_connections" -gt 0 ] 2>/dev/null; then
            log_info "    ✓ Active libp2p connections: $libp2p_connections"
        else
            log_warn "    ✗ No active libp2p connections (firewall blocking port $v_resolver_port?)"
        fi
    done
    echo

    # Check parent chain connectivity
    log_info "Parent Chain Connectivity:"

    # Check if parent RPC is reachable
    local parent_rpc_errors=$(exec_on_host 0 \
        "grep -i \"failed to get.*parent\\|parent.*connection.*failed\\|parent.*RPC.*error\" $node_home/logs/*.log 2>/dev/null | wc -l" 2>/dev/null | tr -d ' \n\r')

    if [ -n "$parent_rpc_errors" ] && [ "$parent_rpc_errors" -gt 0 ] 2>/dev/null; then
        log_warn "  ✗ Parent RPC errors detected ($parent_rpc_errors occurrences)"
        # Show a sample error
        local sample_error=$(exec_on_host 0 \
            "grep -i \"failed to get.*parent\\|parent.*connection.*failed\" $node_home/logs/*.log 2>/dev/null | tail -1" 2>/dev/null)
        if [ -n "$sample_error" ]; then
            log_warn "    Sample: $(echo "$sample_error" | tail -c 120)"
        fi
    else
        log_info "  ✓ No parent RPC connection errors detected"
    fi

    # Check if parent blocks are being fetched
    local parent_blocks_fetched=$(exec_on_host 0 \
        "grep -i \"parent.*block.*height\\|fetched.*parent\" $node_home/logs/*.log 2>/dev/null | tail -1" 2>/dev/null)

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
    local parent_finality_count=$(exec_on_host 0 \
        "grep -i 'ParentFinalityCommitted' $node_home/logs/*.log 2>/dev/null | wc -l" 2>/dev/null | tr -d ' ')

    if [ -n "$parent_finality_count" ] && [ "$parent_finality_count" -gt 0 ] 2>/dev/null; then
        log_info "  ✓ Parent finality commits detected: $parent_finality_count total"

        # Get the most recent one
        local last_finality=$(exec_on_host 0 \
            "grep -i 'ParentFinalityCommitted' $node_home/logs/*.log 2>/dev/null | tail -1" 2>/dev/null)

        if [ -n "$last_finality" ]; then
            # Extract timestamp
            local timestamp=$(echo "$last_finality" | grep -oE '[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}' | head -1)
            if [ -n "$timestamp" ]; then
                log_info "    Last commit: $timestamp"
            fi
        fi

        # Check for top-down message execution
        local topdown_count=$(exec_on_host 0 \
            "grep -i 'topdown' $node_home/logs/*.log 2>/dev/null | grep -i 'exec\|apply\|message' | wc -l" 2>/dev/null | tr -d ' ')

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
        local vote_sent=$(exec_on_host 0 \
            "grep -i PeerVoteReceived $node_home/logs/*.log 2>/dev/null | wc -l" 2>/dev/null | tr -d ' \n\r')
        if [ -n "$vote_sent" ] && [ "$vote_sent" -gt 0 ] 2>/dev/null; then
            log_info "    ✓ Found $vote_sent vote messages"
        else
            log_warn "    ✗ No votes being sent or received"
        fi

        # Check for resolver errors (common issue)
        local resolver_errors=$(exec_on_host 0 \
            "grep -i \"IPLD Resolver.*failed\\|Cannot assign requested address\" $node_home/logs/*.log 2>/dev/null | wc -l" 2>/dev/null | tr -d ' \n\r')
        if [ -n "$resolver_errors" ] && [ "$resolver_errors" -gt 0 ] 2>/dev/null; then
            log_warn "    ✗ Resolver binding errors detected ($resolver_errors occurrences)"
            log_warn "      This means libp2p cannot accept connections"
        fi
    fi
    echo

    # Show validator status summary with voting power
    log_info "Validator Status & Voting Power:"

    # Get validator set from CometBFT (from first validator)
    local validators_json=$(exec_on_host 0 \
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

        # Quick health check
        local is_running=$(exec_on_host "$idx" \
            "if pgrep -f \"ipc-cli node start\" >/dev/null 2>&1; then echo running; else echo stopped; fi" 2>/dev/null | tr -d '\n' | xargs)
        local val_height=$(exec_on_host "$idx" \
            "curl -s http://localhost:26657/status 2>/dev/null | jq -r '.result.sync_info.latest_block_height // \"0\"' 2>/dev/null")
        local val_peers=$(exec_on_host "$idx" \
            "curl -s http://localhost:26657/net_info 2>/dev/null | jq -r '.result.n_peers // 0' 2>/dev/null")

        # Get validator's voting power
        local val_power="?"
        local power_pct="?"
        if [ "$is_running" = "running" ]; then
            local val_info=$(exec_on_host "$idx" \
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
    local cross_msg_logs=$(exec_on_host 0 \
        "grep -i 'topdown' $node_home/logs/*.log 2>/dev/null | tail -5" 2>/dev/null)

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

# Watch parent finality progress in real-time
watch_parent_finality() {
    local target_epoch="${1:-}"
    local refresh_interval="${2:-5}"

    # Use first validator for monitoring
    local validator_idx=0
    local name="${VALIDATORS[0]}"
    local node_home=$(get_node_home 0)

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
        local subnet_parent_finality=$(exec_on_host 0 \
            "grep 'ParentFinalityCommitted' $node_home/logs/*.log 2>/dev/null | tail -1" 2>/dev/null | \
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
        local subnet_height=$(exec_on_host 0 \
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
    local validator_idx=0
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
    prev_height=$(exec_on_host 0 \
        "curl -s http://localhost:26657/status 2>/dev/null | jq -r '.result.sync_info.latest_block_height // 0' 2>/dev/null" || echo "0")
    prev_time=$(date +%s)

    while true; do
        sleep "$refresh_interval"

        iteration=$((iteration + 1))
        local current_time=$(date +%s)
        local elapsed=$((current_time - start_time))

        # Get current block height
        local current_height=$(exec_on_host 0 \
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

        # Get status from CometBFT
        local status=$(exec_on_host "$idx" \
            "curl -s http://localhost:26657/status 2>/dev/null" || echo '{}')

        local height=$(echo "$status" | jq -r '.result.sync_info.latest_block_height // "?"' 2>/dev/null || echo "?")
        local block_hash=$(echo "$status" | jq -r '.result.sync_info.latest_block_hash // "?"' 2>/dev/null || echo "?")
        local app_hash=$(echo "$status" | jq -r '.result.sync_info.latest_app_hash // "?"' 2>/dev/null || echo "?")

        # Get consensus state
        local consensus=$(exec_on_host "$idx" \
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

        local status=$(exec_on_host "$idx" \
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
    local validator_idx=0
    local name="${VALIDATORS[0]}"

    log_info "Source: $name"
    echo ""

    # Get consensus state
    local consensus=$(exec_on_host 0 \
        "curl -s http://localhost:26657/consensus_state 2>/dev/null" || echo '{}')

    local height_round_step=$(echo "$consensus" | jq -r '.result.round_state.height_round_step // "?"' 2>/dev/null)
    local height=$(echo "$height_round_step" | cut -d'/' -f1)
    local round=$(echo "$height_round_step" | cut -d'/' -f2)
    local step=$(echo "$height_round_step" | cut -d'/' -f3)

    log_info "Current consensus: Height $height, Round $round, Step $step"
    echo ""

    # Get validators
    local validators=$(exec_on_host 0 \
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

    local node_home=$(get_node_home 0)
    exec_on_host 0 \
        "tail -20 $node_home/logs/*.consensus.log 2>/dev/null | grep -v 'received complete proposal' | tail -10" || true

    echo ""
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

    local update_cmd="cd $ipc_repo && \
        git fetch origin && \
        git checkout $branch && \
        git pull origin $branch && \
        cargo clean && \
        make"

    log_info "[$name] Pulling latest changes and building... (this may take 10-15 min for full rebuild)"
    if ! ssh_exec_long "$ip" "$ssh_user" "$ipc_user" "$update_cmd"; then
        log_error "[$name] Build failed"
        return 1
    fi

    log_success "[$name] Build completed successfully"

    log_info "[$name] Verifying binaries..."
    local ipc_version
    ipc_version=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
        "test -f $ipc_repo/target/release/ipc-cli && $ipc_repo/target/release/ipc-cli --version 2>&1" 2>/dev/null | head -1)
    if [ -n "$ipc_version" ]; then
        log_info "[$name] $ipc_version"
    fi

    log_success "[$name] Binaries updated successfully"
    return 0
}

# Update binaries on all validators
update_all_binaries() {
    local branch="${1:-main}"

    log_header "Updating IPC Binaries"
    log_info "Branch: $branch"
    log_info "Validators: ${#VALIDATORS[@]}"
    echo ""

    local all_success=true
    local results=()

    for idx in "${!VALIDATORS[@]}"; do
        update_validator_binaries "$idx" "$branch"
        results[$idx]=$?
        [ ${results[$idx]} -ne 0 ] && all_success=false
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

