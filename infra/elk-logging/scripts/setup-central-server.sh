#!/bin/bash
# Setup ELK Stack Central Logging Server
# This script sets up Elasticsearch, Logstash, Kibana, and Grafana

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ELK_DIR="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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

# Check if running as root or with sudo
check_privileges() {
    if [ "$EUID" -eq 0 ]; then
        log_warn "Running as root. This is fine for setup."
    else
        log_info "Not running as root. May need sudo for some operations."
    fi
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."

    # Check Docker
    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed. Please install Docker first."
        log_info "Visit: https://docs.docker.com/engine/install/"
        exit 1
    fi
    log_success "Docker is installed: $(docker --version)"

    # Check Docker Compose
    if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
        log_error "Docker Compose is not installed."
        log_info "Visit: https://docs.docker.com/compose/install/"
        exit 1
    fi
    log_success "Docker Compose is installed"

    # Check if Docker daemon is running
    if ! docker ps &> /dev/null; then
        log_error "Docker daemon is not running. Please start Docker."
        exit 1
    fi
    log_success "Docker daemon is running"
}

# Setup environment file
setup_env_file() {
    log_info "Setting up environment configuration..."

    if [ -f "$ELK_DIR/.env" ]; then
        log_warn ".env file already exists. Skipping creation."
        return 0
    fi

    # Generate random passwords
    ELASTIC_PASSWORD=$(openssl rand -base64 32 | tr -dc 'A-Za-z0-9' | head -c 20)
    KIBANA_ENCRYPTION_KEY=$(openssl rand -base64 32)
    GRAFANA_PASSWORD=$(openssl rand -base64 16 | tr -dc 'A-Za-z0-9' | head -c 16)

    # Get server IP
    SERVER_IP=$(curl -s ifconfig.me || echo "localhost")

    cat > "$ELK_DIR/.env" <<EOF
# ELK Stack Environment Configuration
# Generated on $(date)

# Elasticsearch
ELASTIC_PASSWORD=${ELASTIC_PASSWORD}

# Kibana
KIBANA_ENCRYPTION_KEY=${KIBANA_ENCRYPTION_KEY}

# Grafana
GRAFANA_USER=admin
GRAFANA_PASSWORD=${GRAFANA_PASSWORD}

# Server Configuration
SERVER_IP=${SERVER_IP}
EOF

    chmod 600 "$ELK_DIR/.env"
    log_success "Environment file created at $ELK_DIR/.env"
    log_info "Passwords have been generated. Please save them securely!"
    echo ""
    echo "========================================"
    echo "Elasticsearch Password: $ELASTIC_PASSWORD"
    echo "Grafana User: admin"
    echo "Grafana Password: $GRAFANA_PASSWORD"
    echo "Server IP: $SERVER_IP"
    echo "========================================"
    echo ""
}

# Configure system settings for Elasticsearch
configure_system() {
    log_info "Configuring system settings for Elasticsearch..."

    # Increase vm.max_map_count for Elasticsearch
    current_value=$(sysctl -n vm.max_map_count 2>/dev/null || echo 0)
    if [ "$current_value" -lt 262144 ]; then
        log_info "Increasing vm.max_map_count to 262144..."
        if [ "$EUID" -eq 0 ]; then
            sysctl -w vm.max_map_count=262144
            echo "vm.max_map_count=262144" >> /etc/sysctl.conf
            log_success "vm.max_map_count updated"
        else
            log_warn "Cannot set vm.max_map_count without root. Run:"
            echo "  sudo sysctl -w vm.max_map_count=262144"
            echo "  echo 'vm.max_map_count=262144' | sudo tee -a /etc/sysctl.conf"
        fi
    else
        log_success "vm.max_map_count is already configured"
    fi

    # Create required directories
    log_info "Creating data directories..."
    mkdir -p "$ELK_DIR/elasticsearch/data"
    mkdir -p "$ELK_DIR/logstash/patterns"
    mkdir -p "$ELK_DIR/kibana/data"
    mkdir -p "$ELK_DIR/grafana/dashboards"

    # Set permissions (if not root, this might fail)
    chmod -R 755 "$ELK_DIR/elasticsearch" 2>/dev/null || true
    chmod -R 755 "$ELK_DIR/logstash" 2>/dev/null || true
    chmod -R 755 "$ELK_DIR/kibana" 2>/dev/null || true
    chmod -R 755 "$ELK_DIR/grafana" 2>/dev/null || true

    log_success "Directories created"
}

# Start ELK stack
start_elk_stack() {
    log_info "Starting ELK stack..."

    cd "$ELK_DIR"

    # Pull images first
    log_info "Pulling Docker images (this may take a while)..."
    docker-compose pull

    # Start services
    log_info "Starting services..."
    docker-compose up -d

    log_success "ELK stack started"
    echo ""
    log_info "Waiting for services to be healthy (this may take 2-3 minutes)..."
}

# Wait for services to be ready
wait_for_services() {
    log_info "Checking service health..."

    # Wait for Elasticsearch
    log_info "Waiting for Elasticsearch..."
    for i in {1..60}; do
        if docker-compose exec -T elasticsearch curl -s -u "elastic:${ELASTIC_PASSWORD:-changeme}" http://localhost:9200/_cluster/health &>/dev/null; then
            log_success "Elasticsearch is ready"
            break
        fi
        if [ $i -eq 60 ]; then
            log_error "Elasticsearch failed to start within 5 minutes"
            return 1
        fi
        echo -n "."
        sleep 5
    done

    # Wait for Logstash
    log_info "Waiting for Logstash..."
    for i in {1..30}; do
        if docker-compose exec -T logstash curl -s http://localhost:9600/_node/stats &>/dev/null; then
            log_success "Logstash is ready"
            break
        fi
        if [ $i -eq 30 ]; then
            log_error "Logstash failed to start within 2.5 minutes"
            return 1
        fi
        echo -n "."
        sleep 5
    done

    # Wait for Kibana
    log_info "Waiting for Kibana..."
    for i in {1..60}; do
        if docker-compose exec -T kibana curl -s http://localhost:5601/api/status &>/dev/null; then
            log_success "Kibana is ready"
            break
        fi
        if [ $i -eq 60 ]; then
            log_error "Kibana failed to start within 5 minutes"
            return 1
        fi
        echo -n "."
        sleep 5
    done

    log_success "All services are healthy!"
}

# Setup Elasticsearch index template and ILM policy
setup_elasticsearch() {
    log_info "Setting up Elasticsearch index template and lifecycle policy..."

    cd "$ELK_DIR"
    source .env

    # Create ILM policy
    log_info "Creating ILM policy..."
    curl -X PUT "http://localhost:9200/_ilm/policy/ipc-logs-policy" \
        -u "elastic:${ELASTIC_PASSWORD}" \
        -H 'Content-Type: application/json' \
        -d @elasticsearch/ilm-policy.json \
        &>/dev/null

    if [ $? -eq 0 ]; then
        log_success "ILM policy created"
    else
        log_warn "Failed to create ILM policy (may already exist)"
    fi

    # Create index template
    log_info "Creating index template..."
    curl -X PUT "http://localhost:9200/_index_template/ipc-logs-template" \
        -u "elastic:${ELASTIC_PASSWORD}" \
        -H 'Content-Type: application/json' \
        -d @elasticsearch/index-template.json \
        &>/dev/null

    if [ $? -eq 0 ]; then
        log_success "Index template created"
    else
        log_warn "Failed to create index template (may already exist)"
    fi
}

# Display access information
display_access_info() {
    cd "$ELK_DIR"
    source .env

    echo ""
    echo "========================================"
    echo "  ELK Stack Setup Complete! 🎉"
    echo "========================================"
    echo ""
    echo "Service URLs:"
    echo "  Elasticsearch: http://${SERVER_IP}:9200"
    echo "  Kibana:        http://${SERVER_IP}:5601"
    echo "  Grafana:       http://${SERVER_IP}:3000"
    echo "  Logstash:      ${SERVER_IP}:5044 (Beats input)"
    echo ""
    echo "Credentials:"
    echo "  Elasticsearch:"
    echo "    Username: elastic"
    echo "    Password: ${ELASTIC_PASSWORD}"
    echo ""
    echo "  Kibana:"
    echo "    Username: elastic"
    echo "    Password: ${ELASTIC_PASSWORD}"
    echo ""
    echo "  Grafana:"
    echo "    Username: ${GRAFANA_USER}"
    echo "    Password: ${GRAFANA_PASSWORD}"
    echo ""
    echo "Next Steps:"
    echo "  1. Open Kibana at http://${SERVER_IP}:5601"
    echo "  2. Configure GCP firewall rules for ports 5044, 5601, 3000"
    echo "  3. Run deploy-filebeat.sh to install Filebeat on validators"
    echo ""
    echo "Useful Commands:"
    echo "  View logs:    docker-compose logs -f"
    echo "  Stop stack:   docker-compose down"
    echo "  Restart:      docker-compose restart"
    echo ""
    echo "Configuration saved in: $ELK_DIR/.env"
    echo "========================================"
}

# Main execution
main() {
    echo ""
    echo "========================================"
    echo "  IPC ELK Stack Setup"
    echo "========================================"
    echo ""

    check_privileges
    check_prerequisites
    setup_env_file
    configure_system
    start_elk_stack

    cd "$ELK_DIR"
    wait_for_services
    setup_elasticsearch
    display_access_info
}

# Run main function
main "$@"

