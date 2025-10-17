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
source "${SCRIPT_DIR}/lib/health.sh"

# Global variables
VALIDATORS=()
DRY_RUN=false

# Usage information
usage() {
    cat << EOF
IPC Subnet Manager - Manage IPC validator nodes

Usage: $0 <command> [options]

Commands:
    init              Nuclear option - wipe and reinitialize all nodes
    update-config     Update existing node configs without wiping data
    check             Comprehensive health check on all nodes
    restart           Graceful restart of all nodes
    info              Show subnet information (chain ID, validators, status)
    block-time        Measure block production time (default: 10s sample)
    logs [validator]  Tail logs from specific validator
    deploy            Deploy/update binaries (STUB - not implemented)

Options:
    --config FILE        Path to config file (default: ./ipc-subnet-config.yml)
    --dry-run            Preview actions without executing
    --yes                Skip confirmation prompts
    --duration SECONDS   For block-time: sample duration (default: 10)
    --help            Show this help message

Environment Variables:
    IPC_CONFIG_FILE          Override config file path
    IPC_SUBNET_ID            Override subnet ID
    IPC_VALIDATOR_<N>_IP     Override validator IP addresses
    IPC_PARENT_RPC           Override parent RPC endpoint

Examples:
    $0 init                  # Initialize subnet from scratch
    $0 check                 # Run health checks
    $0 logs validator-1      # View logs from validator-1
    $0 restart --yes         # Restart without confirmation

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
            --yes)
                skip_confirm=true
                shift
                ;;
            --dry-run)
                DRY_RUN=true
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

    # Collect peer information (peer-info.json created during init)
    log_section "Collecting Peer Information"
    collect_all_peer_info

    # Fix listen addresses to bind to 0.0.0.0 instead of public IP
    log_section "Fixing Listen Addresses"
    fix_listen_addresses

    # Update all configs with full mesh
    log_section "Updating Node Configurations"
    update_all_configs

    # Update IPC CLI configs
    log_section "Updating IPC CLI Configuration"
    update_ipc_cli_configs

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

# Show subnet information
cmd_info() {
    load_config
    show_subnet_info
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
            --dry-run)
                DRY_RUN=true
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
        init|restart|deploy)
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
        check)
            cmd_check "$@"
            ;;
        restart)
            cmd_restart "$@"
            ;;
        info)
            cmd_info "$@"
            ;;
        block-time)
            cmd_block_time "$@"
            ;;
        logs)
            cmd_logs "$@"
            ;;
        deploy)
            cmd_deploy "$@"
            ;;
        *)
            log_error "Unknown command: $command"
            usage
            ;;
    esac
}

main "$@"

