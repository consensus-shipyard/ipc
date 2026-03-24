#!/usr/bin/env bash
set -euo pipefail

# IPC Subnet Manager - Main Script
# Manages IPC validator nodes with config-driven automation

# Check bash version
if ((BASH_VERSINFO[0] < 4)); then
    echo "Error: This script requires Bash 4.0 or higher"
    echo "Your version: $BASH_VERSION"
    if [[ "$OSTYPE" == "darwin"* ]]; then
        echo "On macOS, install newer bash with: brew install bash"
        echo "Then run with: /usr/local/bin/bash $(realpath "$0") $*"
    fi
    exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CONFIG_FILE="${IPC_CONFIG_FILE:-${SCRIPT_DIR}/ipc-subnet-config.yml}"
LOCK_FILE="/tmp/ipc-subnet-manager.lock"

# Source library files
source "${SCRIPT_DIR}/lib/colors.sh"
source "${SCRIPT_DIR}/lib/ssh.sh"
source "${SCRIPT_DIR}/lib/config.sh"
source "${SCRIPT_DIR}/lib/exec.sh"
source "${SCRIPT_DIR}/lib/anvil.sh"
source "${SCRIPT_DIR}/lib/health.sh"
source "${SCRIPT_DIR}/lib/bootstrap.sh"
source "${SCRIPT_DIR}/lib/dashboard.sh"

# Global variables
VALIDATORS=()
DRY_RUN=false
DEBUG=false
CLI_MODE=""  # Can be set to "local" or "remote" to override config

# Usage information
usage() {
    cat << EOF
IPC Subnet Manager - Manage IPC validator nodes

Usage: $0 <command> [options]

Commands:
    bootstrap         Install Rust, Foundry, Node.js and build IPC on all hosts (run first on fresh hosts)
    init              Nuclear option - wipe and reinitialize all nodes
    init --resume     Continue init from where it left off (after deploy failure)
    update-config     Update existing node configs without wiping data
    update-binaries   Pull latest code, build, and install binaries on all validators
    deploy-binaries   Copy existing binaries to validators (no build)
    check             Comprehensive health check on all nodes
    diagnose [name]   Detailed diagnostics for troubleshooting (all or one validator)
    restart           Graceful restart of all nodes
    info              Show subnet information (chain ID, validators, status)
    consensus-status  Show consensus state across all validators (heights, hashes, rounds)
    voting-status     Show detailed voting info for current consensus round
    dashboard         Live monitoring dashboard with metrics and errors
    block-time        Measure block production time (default: 10s sample)
    watch-finality    Monitor parent finality progress in real-time
    watch-blocks      Monitor block production in real-time
    logs [validator]  Tail logs from specific validator
    install-systemd   Install systemd services on all validators
    start-storage     Start storage node/gateway services
    stop-storage      Stop storage node/gateway services
    storage-status    Check storage service status
    start-relayer     Start checkpoint relayer on primary validator
    stop-relayer      Stop checkpoint relayer
    relayer-status    Check relayer status and view logs

Options:
    --config FILE        Path to config file (default: ./ipc-subnet-config.yml)
    --mode MODE          Deployment mode: local or remote (overrides config)
    --dry-run            Preview actions without executing
    --yes                Skip confirmation prompts
    --debug              Show verbose debug output
    --branch NAME        For bootstrap/update-binaries: git branch to pull from (default: main)
    --compile MODE       For update-binaries: 'local' or 'remote' (default: remote)
    --git-pull           For update-binaries: fetch/pull latest changes before build (default: off)
    --with-storage       For bootstrap/update-binaries: build ipc-storage feature and storage binaries
    --duration SECONDS   For block-time: sample duration (default: 10)
    --help               Show this help message

Environment Variables:
    IPC_CONFIG_FILE          Override config file path
    IPC_SUBNET_ID            Override subnet ID
    IPC_VALIDATOR_<N>_IP     Override validator IP addresses
    IPC_PARENT_RPC           Override parent RPC endpoint

Examples:
    # Local mode (single machine, multiple validators)
    $0 init --mode local                       # Initialize local subnet
    $0 check --mode local                      # Check local validators
    $0 restart --mode local --yes              # Restart local subnet

    # Remote mode (multiple machines via SSH)
    $0 bootstrap --branch main --with-storage  # First: bootstrap fresh hosts (with storage binaries)
    $0 init                                    # Then: initialize subnet from scratch
    $0 init --resume                           # Resume after deploy (skip wipe/deploy)
    $0 init --debug                            # Initialize with verbose debug output
    $0 check                                   # Run health checks
    $0 update-binaries --branch main           # Update binaries from main branch
    $0 update-binaries -C local --branch main  # Build locally, deploy to validators
    $0 update-binaries --branch main --with-storage  # Update binaries with storage support
    $0 deploy-binaries --path ./target/release # Copy binaries to all validators
    $0 watch-finality                          # Monitor parent finality progress
    $0 watch-blocks                            # Monitor block production
    $0 logs validator-1                        # View logs from validator-1
    $0 start-relayer                           # Start checkpoint relayer on primary
    $0 restart --yes                           # Restart without confirmation

EOF
    exit 0
}

# Acquire lock to prevent concurrent executions
acquire_lock() {
    if [ -e "$LOCK_FILE" ]; then
        local lock_pid
        lock_pid=$(tr -d '[:space:]' < "$LOCK_FILE" 2>/dev/null || true)
        if [ -n "$lock_pid" ] && kill -0 "$lock_pid" 2>/dev/null; then
            log_error "Another instance is running (pid: $lock_pid). Lock file: $LOCK_FILE"
            log_error "Wait for it to finish, or stop that process before retrying."
            exit 1
        fi

        log_warn "Found stale lock file at $LOCK_FILE; removing it."
        rm -f "$LOCK_FILE"
    fi

    echo $$ > "$LOCK_FILE"
    trap 'rm -f "$LOCK_FILE"' EXIT
}

# Confirmation prompt
confirm() {
    local message="$1"
    local skip_confirm="${2:-false}"

    if [ "$skip_confirm" = true ] || [ "$DRY_RUN" = true ]; then
        if [ "$DRY_RUN" = true ]; then
            log_info "[DRY-RUN] Would confirm: $message"
        fi
        return 0
    fi

    log_warn "$message"
    read -p "Continue? (yes/no): " -r
    if [[ ! $REPLY =~ ^[Yy][Ee][Ss]$ ]]; then
        log_info "Operation cancelled."
        exit 0
    fi
}

# Bootstrap validator hosts - install deps and build IPC
cmd_bootstrap() {
    local branch="main"
    local with_storage=false

    while [[ $# -gt 0 ]]; do
        case $1 in
            --branch)
                branch="$2"
                shift 2
                ;;
            --config)
                CONFIG_FILE="$2"
                shift 2
                ;;
            --dry-run)
                DRY_RUN=true
                shift
                ;;
            --with-storage)
                with_storage=true
                shift
                ;;
            --help|-h)
                cat << EOF
Bootstrap validator hosts with IPC and dependencies

Usage: $0 bootstrap [options]

Options:
    --branch NAME    Git branch to clone (default: main)
    --with-storage   Build ipc-storage feature and storage binaries
    --config FILE    Path to config file
    --dry-run        Preview without executing

This will on each host:
  1. Install system packages (build-essential, clang, cmake, etc.)
  2. Create ipc user if needed
  3. Install Rust and wasm32 target
  4. Install Foundry
  5. Install Node.js and pnpm
  6. Clone IPC repo and build

Run this first on fresh GCP/AWS VMs before 'init'.
EOF
                exit 0
                ;;
            --yes)
                shift
                ;;
            *)
                shift
                ;;
        esac
    done

    log_header "Bootstrap Validator Hosts"

    if is_local_mode; then
        log_error "Bootstrap is only for remote mode. Use --mode remote or configure deployment.mode: remote"
        exit 1
    fi

    load_config
    check_requirements
    check_ssh_connectivity
    check_config_validity

    bootstrap_all_hosts "$branch" "$with_storage"
}

# Initialize subnet (nuclear option)
cmd_init() {
    local skip_confirm=false
    local resume=false

    # Parse command-specific options
    while [[ $# -gt 0 ]]; do
        case $1 in
            --config)
                CONFIG_FILE="$2"
                shift 2
                ;;
            --yes)
                skip_confirm=true
                shift
                ;;
            --resume)
                resume=true
                shift
                ;;
            --dry-run)
                DRY_RUN=true
                shift
                ;;
            --debug)
                DEBUG=true
                shift
                ;;
            *)
                shift
                ;;
        esac
    done

    log_header "IPC Subnet Initialization"

    if [ "$resume" = true ]; then
        log_info "Resume mode: skipping wipe, backup, and deploy steps"
    else
        confirm "This will DESTROY all existing node data and reinitialize from scratch!" "$skip_confirm"
    fi

    # Load configuration
    log_info "Loading configuration from: $CONFIG_FILE"
    load_config

    # Start Anvil if in local mode
    if is_local_mode; then
        ensure_anvil_running
    fi

    # Pre-flight checks
    log_section "Pre-flight Checks"
    check_requirements
    check_ssh_connectivity
    check_config_validity

    if [ "$resume" != true ]; then
        # Stop all nodes
        log_section "Stopping All Nodes"
        stop_all_nodes

        # Backup existing data
        log_section "Creating Backups"
        backup_all_nodes

        # Wipe node data
        log_section "Wiping Node Data"
        wipe_all_nodes

        # Ensure EVM keystore exists with validator keys
        log_section "Preparing EVM Keystore"
        ensure_evm_keystore

        # Sync existing IPC CLI config (generated by subnet init) before node init.
        log_section "Syncing IPC CLI Configuration"
        log_info "Copying local IPC config store to validators..."
        update_ipc_cli_configs
    fi

    # Determine subnet ID: from deploy output or from config (resume mode)
    local subnet_id=""
    local deploy_subnet_enabled=$(get_config_value "init.deploy_subnet")

    if [ "$resume" = true ]; then
        subnet_id=$(get_config_value "subnet.id")
        if [ -z "$subnet_id" ] || [ "$subnet_id" = "null" ]; then
            log_error "Resume requires subnet.id in config. Add the subnet ID from the previous deploy."
            exit 1
        fi
        log_info "Resume: using subnet ID from config: $subnet_id"
    elif [ "$deploy_subnet_enabled" = "true" ]; then
        log_section "Deploying Subnet and Gateway Contracts"
        local deployed_subnet_output=$(deploy_subnet)
        subnet_id=$(echo "$deployed_subnet_output" | grep "^SUBNET_ID:" | cut -d: -f2-)

        if [ -z "$subnet_id" ]; then
            log_error "Failed to extract subnet ID from deployment output"
            exit 1
        fi

        log_info "Subnet deployed with ID: $subnet_id"

        # Reload configuration to pick up updated subnet ID
        load_config
    else
        log_info "Subnet deployment disabled (deploy_subnet='$deploy_subnet_enabled')"
        subnet_id=$(get_config_value "subnet.id")
        log_info "Assuming subnet already exists with ID: $subnet_id"
    fi

    if [ -n "$subnet_id" ]; then
        # Force regeneration of node-init configuration from manager YAML.
        # ipc-cli subnet init can leave behind generated node_<subnet>.yaml templates
        # that may contain stale topdown/resolver settings from earlier runs.
        local generated_node_config
        generated_node_config=$(get_generated_node_config_path "$subnet_id")
        if [ -f "$generated_node_config" ]; then
            log_info "Removing cached generated node config: $generated_node_config"
            rm -f "$generated_node_config"
        fi

        # Update child subnet provider_http to use correct port (8546 instead of default 8545)
        # ipc-cli subnet init writes provider_http with default port, but we need the configured port
        log_section "Updating IPC CLI Configuration"
        update_child_subnet_provider "$subnet_id"

        # Update YAML config with parent chain addresses for future deployments
        # ipc-cli subnet init deploys contracts on parent chain and updates ~/.ipc/config.toml
        # We need to persist these addresses to the YAML config
        update_yaml_with_parent_addresses

        # Create genesis using ipc-cli subnet create-genesis
        # This works for both activated and non-activated subnets
        log_section "Creating Genesis"
        log_info "Creating genesis files for subnet $subnet_id..."
        if create_bootstrap_genesis "$subnet_id"; then
            log_success "Genesis created"
        else
            log_error "Failed to create genesis"
            exit 1
        fi
    fi

    # Copy genesis files to remote validators (node init runs on remote; no-op if local or no genesis)
    if [ -n "$subnet_id" ]; then
        copy_genesis_to_remotes "$subnet_id"
    fi

    # Initialize primary node
    log_section "Initializing Primary Node"
    local primary_validator=$(get_primary_validator)
    initialize_primary_node "$primary_validator"

    # Update Fendermint topdown config with correct parent contract addresses
    # This must be done AFTER node init (which creates the Fendermint config)
    # but BEFORE starting validators
    log_section "Updating Fendermint Configuration"
    update_fendermint_topdown_config

    # Extract primary peer info
    local primary_peer_info=$(extract_peer_info "$primary_validator")
    log_info "Primary peer info extracted"

    # Initialize secondary nodes
    log_section "Initializing Secondary Nodes"
    initialize_secondary_nodes "$primary_peer_info"

    # Update Fendermint config for secondary nodes (they now have config from init)
    log_section "Updating Fendermint Configuration"
    update_fendermint_topdown_config

    # Collect peer information from peer-info.json (for libp2p and validator keys)
    log_section "Collecting Peer Information"
    collect_all_peer_info

    # Start nodes temporarily to collect CometBFT node IDs
    log_section "Starting Nodes Temporarily"
    log_info "Starting nodes to collect CometBFT peer IDs..."
    start_all_nodes

    log_info "Waiting for CometBFT to start (15 seconds)..."
    sleep 15

    # Collect CometBFT peer IDs from running nodes
    log_section "Collecting CometBFT Peer IDs"
    collect_peer_ids_from_running_nodes

    # Stop nodes to update configurations
    log_info "Stopping nodes to update peer configurations..."
    stop_all_nodes
    sleep 5

    # Fix listen addresses to bind to 0.0.0.0 instead of public IP
    log_section "Fixing Listen Addresses"
    fix_listen_addresses

    # Update all configs with full mesh
    log_section "Updating Node Configurations"
    update_all_configs

    # Set federated power
    log_section "Setting Validator Power"
    set_federated_power

    # Start all nodes with complete configuration
    log_section "Starting All Nodes"
    start_all_nodes

    # Health checks
    log_section "Running Health Checks"
    log_info "Waiting 30s for nodes to fully start..."
    sleep 30
    cmd_check

    log_success "✓ Subnet initialization complete!"
}

# Update binaries on all validators
cmd_update_binaries() {
    local branch="main"
    local compile_mode="remote"
    local with_storage=false
    local git_pull=false

    # Parse options
    while [[ $# -gt 0 ]]; do
        case $1 in
            --branch)
                branch="$2"
                shift 2
                ;;
            --compile|-C)
                compile_mode="$2"
                if [[ "$compile_mode" != "local" && "$compile_mode" != "remote" ]]; then
                    log_error "Invalid --compile value: $compile_mode (use 'local' or 'remote')"
                    exit 1
                fi
                shift 2
                ;;
            --help|-h)
                cat << EOF
Update IPC binaries on all validators

Usage: $0 update-binaries [options]

Options:
    --branch NAME       Git branch to build from (default: main)
    --compile MODE      Where to build: 'local' or 'remote' (default: remote)
    -C MODE             Short for --compile
    --git-pull          Fetch/pull latest changes before build (default: off)
    --with-storage      Build ipc-storage feature and storage binaries
    --help              Show this help message

Compile modes:
  remote  Build on each validator via SSH (current behavior). Requires pnpm,
          Rust, Foundry on each host.
  local   Build on this machine and SCP binaries to validators. If you're on
          macOS and validators are Linux, cross-compiles to x86_64-unknown-linux-gnu.
          Requires: cargo-zigbuild + zig (recommended) or cross (needs Docker).

Examples:
    $0 update-binaries --branch main
    $0 update-binaries --branch main --compile local
    $0 update-binaries -C local --branch main
    $0 update-binaries --branch main --git-pull
    $0 update-binaries --branch main --with-storage
EOF
                exit 0
                ;;
            --git-pull)
                git_pull=true
                shift
                ;;
            --with-storage)
                with_storage=true
                shift
                ;;
            *)
                log_error "Unknown option: $1"
                echo "Usage: $0 update-binaries [options] (use --help for details)"
                exit 1
                ;;
        esac
    done

    # Load configuration
    load_config

    # Update binaries
    update_all_binaries "$branch" "$compile_mode" "$with_storage" "$git_pull"
}

# Deploy binaries to validators (copy only, no build)
cmd_deploy_binaries() {
    local binary_path=""
    local target_validator=""

    # Parse options
    while [[ $# -gt 0 ]]; do
        case $1 in
            --path)
                binary_path="$2"
                shift 2
                ;;
            --help|-h)
                cat << EOF
Copy ipc-cli and fendermint binaries to validators (no build)

Usage: $0 deploy-binaries [options] [validator-name]

Options:
    --path DIR    Path to directory containing ipc-cli and fendermint
    --help        Show this help message

If --path is omitted, auto-detects from local IPC repo:
  - target/release/ (native build)
  - target/x86_64-unknown-linux-gnu/release/ (cross-compiled)

Examples:
    $0 deploy-binaries --path ./target/release
    $0 deploy-binaries --path ./target/x86_64-unknown-linux-gnu/release
    $0 deploy-binaries validator-2   # Deploy to single validator (uses auto-detect path)
EOF
                exit 0
                ;;
            -*)
                log_error "Unknown option: $1"
                exit 1
                ;;
            *)
                target_validator="$1"
                shift
                ;;
        esac
    done

    load_config

    # Auto-detect path if not specified
    if [ -z "$binary_path" ]; then
        local local_repo
        local_repo=$(get_config_value "paths.local_ipc_repo" 2>/dev/null || true)
        if [ -z "$local_repo" ] || [ "$local_repo" = "null" ]; then
            local_repo=$(cd "${SCRIPT_DIR}/../.." && pwd)
        fi
        if [ -f "$local_repo/target/release/ipc-cli" ]; then
            binary_path="$local_repo/target/release"
        elif [ -f "$local_repo/target/x86_64-unknown-linux-gnu/release/ipc-cli" ]; then
            binary_path="$local_repo/target/x86_64-unknown-linux-gnu/release"
        else
            log_error "Binaries not found. Specify --path or run from IPC repo with built binaries."
            log_info "Expected: target/release/ or target/x86_64-unknown-linux-gnu/release/"
            exit 1
        fi
        log_info "Using binaries from: $binary_path"
    fi

    binary_path=$(cd "$binary_path" 2>/dev/null && pwd)
    if [ -z "$binary_path" ]; then
        log_error "Invalid path"
        exit 1
    fi

    log_header "Deploying Binaries"
    deploy_binaries_only "$binary_path" "$target_validator"
}

# Update existing node configs
cmd_update_config() {
    log_header "Updating Node Configurations"

    load_config

    log_info "Collecting current peer information..."
    collect_peer_ids_from_running_nodes
    collect_all_peer_info

    log_info "Fixing listen addresses..."
    fix_listen_addresses

    log_info "Updating node configurations..."
    update_all_configs

    log_info "Syncing IPC CLI configurations..."
    update_ipc_cli_configs

    log_info "Restarting nodes..."
    cmd_restart --yes

    log_success "✓ Configuration update complete!"
}

# Comprehensive health check
cmd_check() {
    local wait_seconds=0

    for arg in "$@"; do
        case $arg in
            --wait=*) wait_seconds="${arg#*=}" ;;
            --wait) shift; wait_seconds="${1:-30}" ;;
        esac
    done

    log_header "Health Check"

    load_config

    if [ "$wait_seconds" -gt 0 ] 2>/dev/null; then
        log_info "Waiting ${wait_seconds}s for nodes to start..."
        sleep "$wait_seconds"
    fi

    local all_healthy=true

    for validator_idx in "${!VALIDATORS[@]}"; do
        log_subsection "Checking ${VALIDATORS[$validator_idx]}"

        if ! check_validator_health "$validator_idx"; then
            all_healthy=false
        fi
    done

    echo ""
    if [ "$all_healthy" = true ]; then
        log_success "✓ All validators are healthy!"
        return 0
    else
        log_error "✗ Some validators have issues"
        log_info ""
        log_info "Troubleshooting: Run 'diagnose' for detailed diagnostics:"
        log_info "  ./ipc-manager diagnose              # All validators"
        log_info "  ./ipc-manager diagnose validator-1  # Single validator"
        log_info ""
        log_info "If nodes just started, try: ./ipc-manager check --wait 45"
        return 1
    fi
}

# Detailed diagnostics for troubleshooting health check failures
cmd_diagnose() {
    local validator_name="${1:-}"

    log_header "Validator Diagnostics"

    load_config

    if [ -n "$validator_name" ]; then
        local found=false
        for validator_idx in "${!VALIDATORS[@]}"; do
            if [ "${VALIDATORS[$validator_idx]}" = "$validator_name" ]; then
                diagnose_validator "$validator_idx"
                found=true
                break
            fi
        done
        if [ "$found" = false ]; then
            log_error "Validator not found: $validator_name"
            log_info "Available: ${VALIDATORS[*]}"
            exit 1
        fi
    else
        for validator_idx in "${!VALIDATORS[@]}"; do
            diagnose_validator "$validator_idx"
            echo ""
        done
    fi
}

# Restart all nodes
cmd_restart() {
    local skip_confirm=false

    for arg in "$@"; do
        case $arg in
            --yes) skip_confirm=true ;;
        esac
    done

    log_header "Restarting All Nodes"

    confirm "This will restart all validator nodes" "$skip_confirm"

    load_config

    log_info "Stopping all nodes..."
    stop_all_nodes

    log_info "Starting all nodes..."
    start_all_nodes

    log_success "✓ All nodes restarted"
}

# Measure block time
cmd_block_time() {
    local sample_duration=10

    for arg in "$@"; do
        case $arg in
            --duration=*) sample_duration="${arg#*=}" ;;
            --duration) shift; sample_duration="$1" ;;
        esac
    done

    load_config

    measure_all_block_times "$sample_duration"
}

# Watch parent finality progress
cmd_watch_finality() {
    local target_epoch=""
    local refresh_interval=5

    for arg in "$@"; do
        case $arg in
            --target-epoch=*) target_epoch="${arg#*=}" ;;
            --target-epoch) shift; target_epoch="$1" ;;
            --interval=*) refresh_interval="${arg#*=}" ;;
            --interval) shift; refresh_interval="$1" ;;
        esac
    done

    load_config

    watch_parent_finality "$target_epoch" "$refresh_interval"
}

# Watch block production
cmd_watch_blocks() {
    local refresh_interval=2
    local target_height=""

    for arg in "$@"; do
        case $arg in
            --target-height=*) target_height="${arg#*=}" ;;
            --target-height) shift; target_height="$1" ;;
            --interval=*) refresh_interval="${arg#*=}" ;;
            --interval) shift; refresh_interval="$1" ;;
        esac
    done

    load_config

    watch_block_production "$target_height" "$refresh_interval"
}

# Show subnet information
cmd_info() {
    load_config
    show_subnet_info
}

# Show consensus status across validators
cmd_consensus_status() {
    load_config
    show_consensus_status
}

# Show detailed voting status
cmd_voting_status() {
    load_config
    show_voting_status
}

# Live dashboard monitoring
cmd_dashboard() {
    local validator_idx=0
    local refresh_interval=3

    for arg in "$@"; do
        case $arg in
            --validator=*)
                local name="${arg#*=}"
                # Find validator index by name
                for idx in "${!VALIDATORS[@]}"; do
                    if [ "${VALIDATORS[$idx]}" = "$name" ]; then
                        validator_idx=$idx
                        break
                    fi
                done
                ;;
            --validator) shift; validator_idx="$1" ;;
            --interval=*) refresh_interval="${arg#*=}" ;;
            --interval) shift; refresh_interval="$1" ;;
        esac
    done

    run_dashboard "$validator_idx" "$refresh_interval"
}

# View logs
cmd_logs() {
    local validator_name="${1:-}"

    if [ -z "$validator_name" ]; then
        log_error "Please specify a validator name"
        log_info "Usage: $0 logs <validator-name>"
        exit 1
    fi

    load_config

    local validator_idx=$(get_validator_index "$validator_name")
    if [ -z "$validator_idx" ]; then
        log_error "Validator not found: $validator_name"
        exit 1
    fi

    log_info "Tailing logs from $validator_name..."
    log_info "Tip: Pipe locally to filter, e.g. $0 logs $validator_name | grep -i error"

    local ip=$(get_config_value "validators[$validator_idx].ip")
    local ssh_user=$(get_config_value "validators[$validator_idx].ssh_user")
    local ipc_user=$(get_config_value "validators[$validator_idx].ipc_user")
    local node_home=$(get_config_value "paths.node_home")

    # Avoid grep over SSH - nested quoting causes pipe to be interpreted as shell pipe
    ssh_exec_direct "$ip" "$ssh_user" "$ipc_user" "tail -f $node_home/node.log 2>/dev/null"
}

# Deploy binaries (stub)
cmd_deploy() {
    log_warn "Deploy command is not yet implemented"
    log_info "This will be used to deploy/update IPC binaries to validator nodes"
    exit 1
}

# Install systemd services
cmd_install_systemd() {
    local skip_confirm=false
    local install_relayer=false

    for arg in "$@"; do
        case $arg in
            --yes) skip_confirm=true ;;
            --with-relayer) install_relayer=true ;;
        esac
    done

    log_header "Installing Systemd Services"

    confirm "This will install systemd services for node management" "$skip_confirm"

    load_config

    # Install node services on all validators
    log_section "Installing Node Services"
    local success_count=0
    local fail_count=0

    for idx in "${!VALIDATORS[@]}"; do
        if install_systemd_services "$idx"; then
            success_count=$((success_count + 1))
        else
            fail_count=$((fail_count + 1))
        fi
    done

    # Install relayer service on primary validator
    if [ "$install_relayer" = true ]; then
        log_section "Installing Relayer Service"
        local primary_idx=$(get_primary_validator)
        if ! install_relayer_systemd_service "$primary_idx"; then
            log_warn "Relayer systemd service installation failed"
            fail_count=$((fail_count + 1))
        else
            success_count=$((success_count + 1))
        fi
    fi

    echo ""
    log_info "Installation Summary:"
    log_info "  ✓ Successful: $success_count"
    if [ $fail_count -gt 0 ]; then
        log_warn "  ✗ Failed: $fail_count"
        log_info ""
        log_info "Failed installations will fall back to manual process management (nohup/kill)"
        log_info "The system will continue to work, but without systemd benefits"
    fi

    if [ $success_count -gt 0 ]; then
            log_info ""
            log_success "✓ Systemd services installed on $success_count node(s)!"
            log_info ""
            log_info "Services installed to /etc/systemd/system/"
            log_info "You can now manage services with:"
            log_info "  - sudo systemctl start ipc-node"
            log_info "  - sudo systemctl stop ipc-node"
            log_info "  - sudo systemctl status ipc-node"

            if [ "$install_relayer" = true ]; then
                log_info "  - sudo systemctl start ipc-relayer"
                log_info "  - sudo systemctl stop ipc-relayer"
                log_info "  - sudo systemctl status ipc-relayer"
            fi

            log_info ""
            log_info "Or use the manager commands (they auto-detect systemd):"
            log_info "  - ./ipc-manager restart"
            log_info "  - ./ipc-manager start-relayer"
            log_info "  - ./ipc-manager stop-relayer"
        fi
}

cmd_start_storage() {
    local skip_confirm=false
    local register_operator=false

    for arg in "$@"; do
        case $arg in
            --yes) skip_confirm=true ;;
            --register-operator) register_operator=true ;;
        esac
    done

    log_header "Starting Storage Services"
    confirm "This will start storage services on configured validator(s)" "$skip_confirm"

    load_config
    if start_storage_services "$register_operator"; then
        log_success "✓ Storage services started"
    else
        log_error "✗ Failed to start storage services"
        return 1
    fi
}

cmd_stop_storage() {
    local skip_confirm=false
    for arg in "$@"; do
        case $arg in
            --yes) skip_confirm=true ;;
        esac
    done

    log_header "Stopping Storage Services"
    confirm "This will stop storage services on all validators" "$skip_confirm"

    load_config
    stop_all_storage_nodes
    log_success "✓ Storage services stopped"
}

cmd_storage_status() {
    log_header "Storage Service Status"
    load_config
    check_storage_status
}

# Main execution
main() {
    if [ $# -eq 0 ]; then
        usage
    fi

    # Check for help flag first
    if [[ "$1" == "--help" ]] || [[ "$1" == "-h" ]]; then
        usage
    fi

    # Parse global options
    while [[ $# -gt 0 ]]; do
        case $1 in
            --config)
                CONFIG_FILE="$2"
                shift 2
                ;;
            --mode)
                CLI_MODE="$2"
                shift 2
                ;;
            --dry-run)
                DRY_RUN=true
                shift
                ;;
            --debug)
                DEBUG=true
                shift
                ;;
            --help|-h)
                usage
                ;;
            *)
                break
                ;;
        esac
    done

    local command="$1"
    shift

    # Acquire lock for destructive operations
    case $command in
        init|restart|update-binaries|deploy-binaries|start-storage|stop-storage)
            acquire_lock
            ;;
    esac

    # Execute command
    case $command in
        bootstrap)
            cmd_bootstrap "$@"
            ;;
        init)
            cmd_init "$@"
            ;;
        update-config)
            cmd_update_config "$@"
            ;;
        update-binaries)
            cmd_update_binaries "$@"
            ;;
        deploy-binaries)
            cmd_deploy_binaries "$@"
            ;;
        check)
            cmd_check "$@"
            ;;
        diagnose)
            cmd_diagnose "$@"
            ;;
        restart)
            cmd_restart "$@"
            ;;
        info)
            cmd_info "$@"
            ;;
        consensus-status)
            cmd_consensus_status "$@"
            ;;
        voting-status)
            cmd_voting_status "$@"
            ;;
        dashboard|monitor)
            cmd_dashboard "$@"
            ;;
        block-time)
            cmd_block_time "$@"
            ;;
        watch-finality)
            cmd_watch_finality "$@"
            ;;
        watch-blocks)
            cmd_watch_blocks "$@"
            ;;
        logs)
            cmd_logs "$@"
            ;;
        install-systemd)
            load_config
            cmd_install_systemd "$@"
            ;;
        start-storage)
            cmd_start_storage "$@"
            ;;
        stop-storage)
            cmd_stop_storage "$@"
            ;;
        storage-status)
            cmd_storage_status "$@"
            ;;
        start-relayer)
            load_config
            start_relayer
            ;;
        stop-relayer)
            load_config
            stop_relayer
            ;;
        relayer-status)
            load_config
            check_relayer_status
            ;;
        *)
            log_error "Unknown command: $command"
            usage
            ;;
    esac
}

main "$@"

