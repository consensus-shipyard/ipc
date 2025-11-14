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
    init              Nuclear option - wipe and reinitialize all nodes
    update-config     Update existing node configs without wiping data
    update-binaries   Pull latest code, build, and install binaries on all validators
    check             Comprehensive health check on all nodes
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
    start-relayer     Start checkpoint relayer on primary validator
    stop-relayer      Stop checkpoint relayer
    relayer-status    Check relayer status and view logs

Options:
    --config FILE        Path to config file (default: ./ipc-subnet-config.yml)
    --mode MODE          Deployment mode: local or remote (overrides config)
    --dry-run            Preview actions without executing
    --yes                Skip confirmation prompts
    --debug              Show verbose debug output
    --branch NAME        For update-binaries: git branch to pull from (default: main)
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
    $0 init                                    # Initialize subnet from scratch
    $0 init --debug                            # Initialize with verbose debug output
    $0 check                                   # Run health checks
    $0 update-binaries --branch main           # Update binaries from main branch
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
        log_error "Another instance is running. Lock file exists: $LOCK_FILE"
        log_error "If you're sure no other instance is running, remove the lock file."
        exit 1
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

# Initialize subnet (nuclear option)
cmd_init() {
    local skip_confirm=false

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

    confirm "This will DESTROY all existing node data and reinitialize from scratch!" "$skip_confirm"

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

    # Stop all nodes
    log_section "Stopping All Nodes"
    stop_all_nodes

    # Backup existing data
    log_section "Creating Backups"
    backup_all_nodes

    # Wipe node data
    log_section "Wiping Node Data"
    wipe_all_nodes

    # Clean IPC CLI config directory to avoid corrupted files
    # Preserve the EVM keystore which contains validator keys
    log_info "Cleaning IPC CLI config directory (preserving keystore)..."
    if is_local_mode; then
        # Preserve keystore, only remove config.toml
        rm -f ~/.ipc/config.toml
    else
        for idx in "${!VALIDATORS[@]}"; do
            local ipc_config_dir=$(get_config_value "paths.ipc_config_dir")
            ipc_config_dir="${ipc_config_dir/#\~/$HOME}"
            # Preserve keystore, only remove config.toml
            exec_on_host "$idx" "rm -f $ipc_config_dir/config.toml"
        done
    fi

    # Ensure EVM keystore exists with validator keys
    log_section "Preparing EVM Keystore"
    ensure_evm_keystore

    # Update IPC CLI configs (must be done BEFORE subnet deployment)
    log_section "Deploying IPC CLI Configuration"
    log_info "Creating ~/.ipc/config.toml with parent subnet configuration..."
    update_ipc_cli_configs

    # Deploy subnet with gateway contracts if enabled
    local deploy_subnet_enabled=$(get_config_value "init.deploy_subnet")
    log_info "Checking subnet deployment flag: deploy_subnet_enabled='$deploy_subnet_enabled'"

    if [ "$deploy_subnet_enabled" = "true" ]; then
        log_section "Deploying Subnet and Gateway Contracts"
        local deployed_subnet_output=$(deploy_subnet)
        # Extract subnet ID from marker line
        local deployed_subnet_id=$(echo "$deployed_subnet_output" | grep "^SUBNET_ID:" | cut -d: -f2-)

        if [ -z "$deployed_subnet_id" ]; then
            log_error "Failed to extract subnet ID from deployment output"
            exit 1
        fi

        log_info "Subnet deployed with ID: $deployed_subnet_id"

        # Reload configuration to pick up updated subnet ID
        load_config

        # Create genesis using ipc-cli subnet create-genesis
        # This works for both activated and non-activated subnets
        log_section "Creating Genesis"
        log_info "Creating genesis files for subnet $deployed_subnet_id..."
        if create_bootstrap_genesis "$deployed_subnet_id"; then
            log_success "Genesis created"
        else
            log_error "Failed to create genesis"
            exit 1
        fi
    else
        log_info "Subnet deployment disabled (deploy_subnet='$deploy_subnet_enabled')"
        log_info "Assuming subnet already exists with ID: $(get_config_value 'subnet.id')"
    fi

    # Initialize primary node
    log_section "Initializing Primary Node"
    local primary_validator=$(get_primary_validator)
    initialize_primary_node "$primary_validator"

    # Extract primary peer info
    local primary_peer_info=$(extract_peer_info "$primary_validator")
    log_info "Primary peer info extracted"

    # Initialize secondary nodes
    log_section "Initializing Secondary Nodes"
    initialize_secondary_nodes "$primary_peer_info"

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
    sleep 10  # Give nodes time to start
    cmd_check

    log_success "✓ Subnet initialization complete!"
}

# Update binaries on all validators
cmd_update_binaries() {
    local branch="main"

    # Parse options
    while [[ $# -gt 0 ]]; do
        case $1 in
            --branch)
                branch="$2"
                shift 2
                ;;
            --help|-h)
                cat << EOF
Update IPC binaries on all validators

Usage: $0 update-binaries [options]

Options:
    --branch NAME    Git branch to pull from (default: main)
    --help           Show this help message

This command will:
  1. SSH to each validator (in parallel)
  2. Pull latest changes from the specified branch
  3. Build binaries using 'make' in the repo root
  4. Copy ipc-cli and fendermint binaries to /usr/local/bin

Examples:
    $0 update-binaries --branch main
    $0 update-binaries --branch dev
    $0 update-binaries --branch feature-xyz
EOF
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                echo "Usage: $0 update-binaries --branch <branch-name>"
                exit 1
                ;;
        esac
    done

    # Load configuration
    load_config

    # Update binaries
    update_all_binaries "$branch"
}

# Update existing node configs
cmd_update_config() {
    log_header "Updating Node Configurations"

    load_config

    log_info "Collecting current peer information..."
    collect_all_peer_info

    log_info "Fixing listen addresses..."
    fix_listen_addresses

    log_info "Updating node configurations..."
    update_all_configs

    log_info "Updating IPC CLI configurations..."
    update_ipc_cli_configs

    log_info "Restarting nodes..."
    cmd_restart --yes

    log_success "✓ Configuration update complete!"
}

# Comprehensive health check
cmd_check() {
    log_header "Health Check"

    load_config

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
        return 1
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

    local ip=$(get_config_value "validators[$validator_idx].ip")
    local ssh_user=$(get_config_value "validators[$validator_idx].ssh_user")
    local ipc_user=$(get_config_value "validators[$validator_idx].ipc_user")
    local node_home=$(get_config_value "paths.node_home")

    ssh_exec_direct "$ip" "$ssh_user" "$ipc_user" "tail -f $node_home/logs/*.log | grep --line-buffered 'ParentFinality\|ERROR\|WARN'"
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
        init|restart|update-binaries)
            acquire_lock
            ;;
    esac

    # Execute command
    case $command in
        init)
            cmd_init "$@"
            ;;
        update-config)
            cmd_update_config "$@"
            ;;
        update-binaries)
            cmd_update_binaries "$@"
            ;;
        check)
            cmd_check "$@"
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

