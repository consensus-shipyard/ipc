#!/bin/bash
# Bootstrap functions - install IPC and dependencies on fresh validator hosts

# Bootstrap a single validator host
# Usage: bootstrap_validator_host <validator_idx> [branch]
bootstrap_validator_host() {
    local validator_idx="$1"
    local branch="${2:-main}"
    local with_storage="${3:-false}"

    local name="${VALIDATORS[$validator_idx]}"
    local ip=$(get_config_value "validators[$validator_idx].ip")
    local ssh_user=$(get_config_value "validators[$validator_idx].ssh_user")
    local ipc_user=$(get_config_value "validators[$validator_idx].ipc_user")
    local ipc_repo=$(get_config_value "paths.ipc_repo")
    local ipc_repo_dir=$(dirname "$ipc_repo")

    log_info "[$name] Bootstrapping host (branch: $branch)..."

    # Step 1: Install system packages (run as ssh_user with sudo)
    log_info "[$name] Installing system packages..."
    local apt_output
    apt_output=$(ssh_run_sudo "$ip" "$ssh_user" \
        "sudo DEBIAN_FRONTEND=noninteractive apt-get update -qq && \
         sudo DEBIAN_FRONTEND=noninteractive apt-get install -y -qq \
         build-essential clang cmake pkg-config libssl-dev protobuf-compiler \
         git curl 2>&1")

    if [ $? -ne 0 ]; then
        log_error "[$name] Failed to install system packages"
        echo "$apt_output" | tail -15
        return 1
    fi
    log_success "[$name] System packages installed"

    # Step 2: Ensure ipc user exists
    log_info "[$name] Ensuring $ipc_user user exists..."
    ssh_run_sudo "$ip" "$ssh_user" \
        "id $ipc_user >/dev/null 2>&1 || sudo useradd -m -s /bin/bash $ipc_user" >/dev/null 2>&1

    if [ $? -ne 0 ]; then
        log_error "[$name] Failed to create $ipc_user user"
        return 1
    fi
    log_success "[$name] User $ipc_user ready"

    # Step 3: Install Rust as ipc user (if not already installed)
    log_info "[$name] Installing Rust..."
    local rust_check
    rust_check=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
        "command -v rustc >/dev/null 2>&1 && rustc --version || echo 'not_installed'" 2>/dev/null | tail -1)

    if [[ "$rust_check" == *"not_installed"* ]] || [ -z "$rust_check" ]; then
        ssh_exec "$ip" "$ssh_user" "$ipc_user" \
            'curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y -q && source ~/.cargo/env && rustup target add wasm32-unknown-unknown' >/dev/null 2>&1

        if [ $? -ne 0 ]; then
            log_error "[$name] Failed to install Rust"
            return 1
        fi
        log_success "[$name] Rust installed"
    else
        log_info "[$name] Rust already installed: $rust_check"
    fi

    # Step 4: Install Foundry as ipc user (if not already installed)
    log_info "[$name] Installing Foundry..."
    local forge_check
    forge_check=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
        "source ~/.bashrc 2>/dev/null; source ~/.profile 2>/dev/null; command -v forge >/dev/null 2>&1 && forge --version || echo 'not_installed'" 2>/dev/null | tail -1)

    if [[ "$forge_check" == *"not_installed"* ]] || [ -z "$forge_check" ]; then
        # Foundry installer - use yes to auto-confirm any prompts
        ssh_exec "$ip" "$ssh_user" "$ipc_user" \
            'yes | curl -sL https://foundry.paradigm.xyz | bash' >/dev/null 2>&1
        ssh_exec "$ip" "$ssh_user" "$ipc_user" \
            'export PATH="$HOME/.foundry/bin:$PATH"; foundryup' >/dev/null 2>&1

        if [ $? -ne 0 ]; then
            log_error "[$name] Failed to install Foundry"
            return 1
        fi
        log_success "[$name] Foundry installed"
    else
        log_info "[$name] Foundry already installed"
    fi

    # Step 5: Install Node.js and pnpm (needed for contracts build)
    log_info "[$name] Installing Node.js and pnpm..."
    local node_check
    node_check=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
        "command -v node >/dev/null 2>&1 && node --version || echo 'not_installed'" 2>/dev/null | tail -1)

    if [[ "$node_check" == *"not_installed"* ]] || [ -z "$node_check" ]; then
        # Install Node 20 LTS via NodeSource
        ssh_run_sudo "$ip" "$ssh_user" \
            "curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash - && \
             sudo DEBIAN_FRONTEND=noninteractive apt-get install -y -qq nodejs" >/dev/null 2>&1

        if [ $? -ne 0 ]; then
            log_warn "[$name] Node install via NodeSource failed, trying alternative..."
            # Fallback: install via apt (may be older version)
            ssh_run_sudo "$ip" "$ssh_user" \
                "sudo DEBIAN_FRONTEND=noninteractive apt-get install -y -qq nodejs npm" >/dev/null 2>&1
        fi
    fi

    # Install pnpm as ipc user (to user dir - npm -g uses /usr/lib by default which needs root)
    local pnpm_check
    pnpm_check=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
        'export PATH="$HOME/.npm-global/bin:$PATH"; (command -v pnpm >/dev/null 2>&1 && pnpm --version) || echo "not_installed"' 2>/dev/null | tail -1)

    if [[ "$pnpm_check" == *"not_installed"* ]] || [ -z "$pnpm_check" ]; then
        # Install pnpm to ~/.npm-global (writable by ipc user)
        ssh_exec "$ip" "$ssh_user" "$ipc_user" \
            'mkdir -p ~/.npm-global && npm config set prefix ~/.npm-global && export PATH="$HOME/.npm-global/bin:$PATH" && npm install -g pnpm' >/dev/null 2>&1

        # Persist PATH for future shells (idempotent - skip if already added)
        ssh_exec "$ip" "$ssh_user" "$ipc_user" \
            'grep -q "npm-global/bin" ~/.bashrc 2>/dev/null || echo "export PATH=\"\$HOME/.npm-global/bin:\$PATH\"" >> ~/.bashrc' >/dev/null 2>&1
    fi
    log_success "[$name] Node.js and pnpm ready"

    # Step 6: Clone IPC repo (or update if exists)
    log_info "[$name] Setting up IPC repository..."
    local clone_cmd
    if ssh_exec "$ip" "$ssh_user" "$ipc_user" "test -d $ipc_repo/.git 2>/dev/null" >/dev/null 2>&1; then
        log_info "[$name] Repository exists, fetching updates..."
        clone_cmd="cd $ipc_repo && git fetch origin && git checkout $branch && git pull origin $branch"
    else
        ssh_exec "$ip" "$ssh_user" "$ipc_user" "mkdir -p $ipc_repo_dir" >/dev/null 2>&1
        clone_cmd="git clone --branch $branch https://github.com/consensus-shipyard/ipc.git $ipc_repo"
    fi

    local clone_output
    clone_output=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" "$clone_cmd 2>&1")
    if [ $? -ne 0 ]; then
        log_error "[$name] Failed to clone/update repository"
        echo "$clone_output" | tail -10
        return 1
    fi
    log_success "[$name] Repository ready"

    # Step 7: Initialize submodules and build
    log_info "[$name] Building IPC (this may take several minutes)..."
    local build_cmd="cd $ipc_repo && \
        git submodule update --init --recursive && \
        source ~/.cargo/env 2>/dev/null; \
        source ~/.bashrc 2>/dev/null; \
        export PATH=\"\$HOME/.cargo/bin:\$HOME/.foundry/bin:\$HOME/.npm-global/bin:\$PATH\"; \
        make 2>&1"

    if [ "$with_storage" = "true" ]; then
        # Some branches do not expose ipc-storage on ipc-cli directly.
        # Fall back to plain ipc-cli build when the feature is unavailable.
        build_cmd="$build_cmd && \
        if rg -q \"^[[:space:]]*ipc-storage[[:space:]]*=\" ipc/cli/Cargo.toml; then \
            cargo build --release -p ipc-cli --features ipc-storage; \
        else \
            echo \"[warn] ipc-cli feature 'ipc-storage' not found; building ipc-cli without feature\"; \
            cargo build --release -p ipc-cli; \
        fi && \
        cargo build --release -p fendermint_app --features ipc-storage && \
        cargo build --release -p ipc-decentralized-storage --bin node --bin gateway 2>&1"
    fi

    local build_output
    build_output=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" "$build_cmd")

    if [ $? -ne 0 ]; then
        log_error "[$name] Build failed"
        echo "$build_output" | tail -30
        return 1
    fi

    log_success "[$name] Build completed successfully"

    # Step 8: Verify binaries
    local ipc_binary=$(get_config_value "paths.ipc_binary")
    local verify_output
    verify_output=$(ssh_exec "$ip" "$ssh_user" "$ipc_user" \
        "test -f $ipc_repo/target/release/ipc-cli && $ipc_repo/target/release/ipc-cli --version 2>&1" 2>/dev/null)

    if [ $? -eq 0 ]; then
        log_info "[$name] $verify_output"
    else
        log_warn "[$name] Could not verify ipc-cli binary"
    fi

    log_success "[$name] Bootstrap complete!"
    return 0
}

# Bootstrap all validator hosts
# Usage: bootstrap_all_hosts [branch]
bootstrap_all_hosts() {
    local branch="${1:-main}"
    local with_storage="${2:-false}"

    log_header "Bootstrap Validator Hosts"
    log_info "This will install Rust, Foundry, Node.js, and build IPC on each host."
    log_info "Branch: $branch"
    if [ "$with_storage" = "true" ]; then
        log_info "Storage build: enabled (ipc-storage feature + storage binaries)"
    fi
    log_info "Validators: ${#VALIDATORS[@]}"
    echo ""

    local all_success=true
    local results=()

    for idx in "${!VALIDATORS[@]}"; do
        local name="${VALIDATORS[$idx]}"
        log_subsection "$name"

        if bootstrap_validator_host "$idx" "$branch" "$with_storage"; then
            results[$idx]=0
        else
            results[$idx]=1
            all_success=false
        fi
        echo ""
    done

    log_section "Bootstrap Summary"
    for idx in "${!VALIDATORS[@]}"; do
        local name="${VALIDATORS[$idx]}"
        if [ ${results[$idx]:-1} -eq 0 ]; then
            log_success "✓ $name: Bootstrap successful"
        else
            log_error "✗ $name: Bootstrap failed"
        fi
    done

    if [ "$all_success" = true ]; then
        echo ""
        log_success "✓ All hosts bootstrapped successfully!"
        log_info "You can now run: $0 init --config $CONFIG_FILE"
        return 0
    else
        echo ""
        log_error "✗ Some hosts failed to bootstrap"
        return 1
    fi
}
