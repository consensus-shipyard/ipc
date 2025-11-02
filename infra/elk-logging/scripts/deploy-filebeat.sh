#!/bin/bash
# Deploy Filebeat to IPC Validator Nodes
# This script installs and configures Filebeat on all validator nodes

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ELK_DIR="$(dirname "$SCRIPT_DIR")"
IPC_CONFIG="${IPC_CONFIG:-$HOME/github/ipc/scripts/ipc-subnet-manager/ipc-subnet-config.yml}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."

    # Check if yq is installed (for YAML parsing)
    if ! command -v yq &> /dev/null; then
        log_error "yq is not installed. Please install it first:"
        log_info "  macOS: brew install yq"
        log_info "  Linux: snap install yq"
        exit 1
    fi

    # Check if IPC config file exists
    if [ ! -f "$IPC_CONFIG" ]; then
        log_error "IPC subnet config not found: $IPC_CONFIG"
        log_info "Please set IPC_CONFIG environment variable or ensure file exists"
        exit 1
    fi

    # Check if .env file exists
    if [ ! -f "$ELK_DIR/.env" ]; then
        log_error ".env file not found. Please run setup-central-server.sh first"
        exit 1
    fi

    log_success "Prerequisites checked"
}

# Load configuration
load_config() {
    log_info "Loading configuration..."

    # Source environment variables
    source "$ELK_DIR/.env"

    # Read subnet config
    SUBNET_ID=$(yq eval '.subnet.id' "$IPC_CONFIG")
    PARENT_RPC=$(yq eval '.subnet.parent_rpc' "$IPC_CONFIG")
    PARENT_CHAIN_ID=$(yq eval '.subnet.parent_chain_id' "$IPC_CONFIG")
    NODE_HOME=$(yq eval '.paths.node_home' "$IPC_CONFIG")

    log_success "Configuration loaded"
    log_info "  Subnet ID: $SUBNET_ID"
    log_info "  Logstash: ${SERVER_IP}:5044"
}

# Get validator count
get_validator_count() {
    yq eval '.validators | length' "$IPC_CONFIG"
}

# Get validator info
get_validator_info() {
    local idx=$1
    local field=$2
    yq eval ".validators[$idx].$field" "$IPC_CONFIG"
}

# Download Filebeat binary
download_filebeat() {
    local validator_ip=$1
    local ssh_user=$2

    log_info "Downloading Filebeat on $validator_ip..."

    ssh -o StrictHostKeyChecking=no "$ssh_user@$validator_ip" bash <<'ENDSSH'
set -e

# Determine architecture
ARCH=$(uname -m)
if [ "$ARCH" = "x86_64" ]; then
    FILEBEAT_ARCH="amd64"
elif [ "$ARCH" = "aarch64" ]; then
    FILEBEAT_ARCH="arm64"
else
    echo "Unsupported architecture: $ARCH"
    exit 1
fi

FILEBEAT_VERSION="8.11.0"
FILEBEAT_TAR="filebeat-${FILEBEAT_VERSION}-linux-${FILEBEAT_ARCH}.tar.gz"
FILEBEAT_URL="https://artifacts.elastic.co/downloads/beats/filebeat/${FILEBEAT_TAR}"

# Download if not already present
if [ ! -f "/usr/local/bin/filebeat" ]; then
    echo "Downloading Filebeat ${FILEBEAT_VERSION}..."
    cd /tmp
    curl -L -O "$FILEBEAT_URL"
    tar xzf "$FILEBEAT_TAR"

    # Install binary
    sudo cp "filebeat-${FILEBEAT_VERSION}-linux-${FILEBEAT_ARCH}/filebeat" /usr/local/bin/
    sudo chmod +x /usr/local/bin/filebeat

    # Cleanup
    rm -rf "$FILEBEAT_TAR" "filebeat-${FILEBEAT_VERSION}-linux-${FILEBEAT_ARCH}"

    echo "Filebeat installed"
else
    echo "Filebeat already installed"
fi

# Create directories
sudo mkdir -p /etc/filebeat
sudo mkdir -p /var/lib/filebeat
sudo mkdir -p /var/log/filebeat

# Set permissions
sudo chmod 755 /etc/filebeat
sudo chmod 755 /var/lib/filebeat
sudo chmod 755 /var/log/filebeat
ENDSSH

    if [ $? -eq 0 ]; then
        log_success "Filebeat downloaded and installed on $validator_ip"
        return 0
    else
        log_error "Failed to download Filebeat on $validator_ip"
        return 1
    fi
}

# Deploy Filebeat configuration
deploy_filebeat_config() {
    local idx=$1
    local validator_name=$(get_validator_info "$idx" "name")
    local validator_ip=$(get_validator_info "$idx" "ip")
    local validator_role=$(get_validator_info "$idx" "role")
    local ssh_user=$(get_validator_info "$idx" "ssh_user")

    log_info "Deploying Filebeat config to $validator_name ($validator_ip)..."

    # Create customized config from template
    local temp_config="/tmp/filebeat-${validator_name}.yml"

    sed -e "s|__VALIDATOR_NAME__|${validator_name}|g" \
        -e "s|__VALIDATOR_IP__|${validator_ip}|g" \
        -e "s|__VALIDATOR_ROLE__|${validator_role}|g" \
        -e "s|__NODE_HOME__|${NODE_HOME}|g" \
        -e "s|__SUBNET_ID__|${SUBNET_ID}|g" \
        -e "s|__PARENT_RPC__|${PARENT_RPC}|g" \
        -e "s|__PARENT_CHAIN_ID__|${PARENT_CHAIN_ID}|g" \
        -e "s|__LOGSTASH_HOST__|${SERVER_IP}|g" \
        "$ELK_DIR/filebeat/filebeat.yml.template" > "$temp_config"

    # Copy config to validator
    if ! scp -o StrictHostKeyChecking=no "$temp_config" "$ssh_user@$validator_ip:/tmp/filebeat.yml" >/dev/null 2>&1; then
        log_error "Failed to copy config to $validator_name"
        rm -f "$temp_config"
        return 1
    fi

    # Move config to /etc/filebeat
    ssh -o StrictHostKeyChecking=no "$ssh_user@$validator_ip" \
        "sudo mv /tmp/filebeat.yml /etc/filebeat/filebeat.yml && sudo chmod 644 /etc/filebeat/filebeat.yml" >/dev/null 2>&1

    if [ $? -eq 0 ]; then
        log_success "Config deployed to $validator_name"
        rm -f "$temp_config"
        return 0
    else
        log_error "Failed to deploy config to $validator_name"
        rm -f "$temp_config"
        return 1
    fi
}

# Deploy systemd service
deploy_systemd_service() {
    local idx=$1
    local validator_name=$(get_validator_info "$idx" "name")
    local validator_ip=$(get_validator_info "$idx" "ip")
    local ssh_user=$(get_validator_info "$idx" "ssh_user")

    log_info "Deploying systemd service to $validator_name..."

    # Copy service file
    if ! scp -o StrictHostKeyChecking=no "$ELK_DIR/filebeat/filebeat.service.template" "$ssh_user@$validator_ip:/tmp/filebeat.service" >/dev/null 2>&1; then
        log_error "Failed to copy service file to $validator_name"
        return 1
    fi

    # Install service
    ssh -o StrictHostKeyChecking=no "$ssh_user@$validator_ip" bash <<'ENDSSH'
set -e
sudo mv /tmp/filebeat.service /etc/systemd/system/filebeat.service
sudo chmod 644 /etc/systemd/system/filebeat.service
sudo systemctl daemon-reload
sudo systemctl enable filebeat.service
ENDSSH

    if [ $? -eq 0 ]; then
        log_success "Systemd service installed on $validator_name"
        return 0
    else
        log_error "Failed to install systemd service on $validator_name"
        return 1
    fi
}

# Start Filebeat
start_filebeat() {
    local idx=$1
    local validator_name=$(get_validator_info "$idx" "name")
    local validator_ip=$(get_validator_info "$idx" "ip")
    local ssh_user=$(get_validator_info "$idx" "ssh_user")

    log_info "Starting Filebeat on $validator_name..."

    ssh -o StrictHostKeyChecking=no "$ssh_user@$validator_ip" \
        "sudo systemctl restart filebeat.service" >/dev/null 2>&1

    if [ $? -eq 0 ]; then
        log_success "Filebeat started on $validator_name"

        # Check status
        sleep 2
        local status=$(ssh -o StrictHostKeyChecking=no "$ssh_user@$validator_ip" \
            "sudo systemctl is-active filebeat.service" 2>/dev/null)

        if [ "$status" = "active" ]; then
            log_success "Filebeat is running on $validator_name"
        else
            log_warn "Filebeat may not be running on $validator_name (status: $status)"
        fi

        return 0
    else
        log_error "Failed to start Filebeat on $validator_name"
        return 1
    fi
}

# Test log flow
test_log_flow() {
    local idx=$1
    local validator_name=$(get_validator_info "$idx" "name")
    local validator_ip=$(get_validator_info "$idx" "ip")
    local ssh_user=$(get_validator_info "$idx" "ssh_user")

    log_info "Testing log flow from $validator_name..."

    # Generate a test log entry
    ssh -o StrictHostKeyChecking=no "$ssh_user@$validator_ip" \
        "logger -t ipc-elk-test 'Test log from $validator_name at $(date)'" >/dev/null 2>&1

    log_info "Test log sent from $validator_name"
}

# Deploy to single validator
deploy_to_validator() {
    local idx=$1
    local validator_name=$(get_validator_info "$idx" "name")

    echo ""
    log_info "========================================"
    log_info "  Deploying to $validator_name"
    log_info "========================================"

    local validator_ip=$(get_validator_info "$idx" "ip")
    local ssh_user=$(get_validator_info "$idx" "ssh_user")

    # Test SSH connection first
    if ! ssh -o StrictHostKeyChecking=no -o ConnectTimeout=5 "$ssh_user@$validator_ip" "echo test" >/dev/null 2>&1; then
        log_error "Cannot connect to $validator_name ($validator_ip)"
        return 1
    fi

    # Deploy steps
    download_filebeat "$validator_ip" "$ssh_user" || return 1
    deploy_filebeat_config "$idx" || return 1
    deploy_systemd_service "$idx" || return 1
    start_filebeat "$idx" || return 1
    test_log_flow "$idx" || true

    log_success "Deployment complete for $validator_name"
    return 0
}

# Main deployment function
main() {
    echo ""
    echo "========================================"
    echo "  IPC Filebeat Deployment"
    echo "========================================"
    echo ""

    check_prerequisites
    load_config

    local validator_count=$(get_validator_count)
    log_info "Found $validator_count validators"

    local success_count=0
    local fail_count=0

    # Deploy to each validator
    for idx in $(seq 0 $((validator_count - 1))); do
        if deploy_to_validator "$idx"; then
            success_count=$((success_count + 1))
        else
            fail_count=$((fail_count + 1))
        fi
    done

    # Summary
    echo ""
    echo "========================================"
    echo "  Deployment Summary"
    echo "========================================"
    echo "  Successful: $success_count"
    echo "  Failed: $fail_count"
    echo ""

    if [ $fail_count -eq 0 ]; then
        log_success "All validators deployed successfully!"
        echo ""
        log_info "Next steps:"
        echo "  1. Check logs are flowing: $SCRIPT_DIR/check-log-flow.sh"
        echo "  2. Open Kibana: http://${SERVER_IP}:5601"
        echo "  3. Create index pattern: ipc-logs-*"
    else
        log_warn "Some validators failed. Check logs above for details."
    fi

    echo "========================================"
}

# Run main function
main "$@"

