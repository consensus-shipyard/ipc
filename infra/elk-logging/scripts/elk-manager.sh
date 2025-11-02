#!/bin/bash
# ELK Stack Management Script
# Convenient commands for managing the ELK stack

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ELK_DIR="$(dirname "$SCRIPT_DIR")"

# Colors
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

# Load environment if available
if [ -f "$ELK_DIR/.env" ]; then
    source "$ELK_DIR/.env"
fi

# Help text
show_help() {
    cat <<EOF
ELK Stack Manager - IPC Validator Logs

Usage: $0 <command> [options]

Commands:
  status                Show status of all services
  start                 Start all ELK services
  stop                  Stop all ELK services
  restart [service]     Restart all services or specific service
  logs [service]        View logs (follows by default)
  health                Check health of all components
  indices               List Elasticsearch indices
  search <query>        Quick search logs
  delete-old-logs <days> Delete logs older than N days
  backup                Create Elasticsearch snapshot
  update                Update all Docker images
  clean                 Clean up old Docker resources
  filebeat-status       Check Filebeat status on all validators
  help                  Show this help message

Examples:
  $0 status
  $0 restart logstash
  $0 logs elasticsearch
  $0 search "validator:validator-1 AND ERROR"
  $0 delete-old-logs 30
  $0 filebeat-status

EOF
}

# Check if Docker Compose is available
check_docker_compose() {
    cd "$ELK_DIR"
    if docker compose version >/dev/null 2>&1; then
        DOCKER_COMPOSE="docker compose"
    elif docker-compose --version >/dev/null 2>&1; then
        DOCKER_COMPOSE="docker-compose"
    else
        log_error "Docker Compose not found"
        exit 1
    fi
}

# Show status
cmd_status() {
    cd "$ELK_DIR"
    log_info "ELK Stack Status:"
    echo ""
    $DOCKER_COMPOSE ps
}

# Start services
cmd_start() {
    cd "$ELK_DIR"
    log_info "Starting ELK stack..."
    $DOCKER_COMPOSE up -d
    log_success "ELK stack started"
}

# Stop services
cmd_stop() {
    cd "$ELK_DIR"
    log_info "Stopping ELK stack..."
    $DOCKER_COMPOSE down
    log_success "ELK stack stopped"
}

# Restart services
cmd_restart() {
    cd "$ELK_DIR"
    local service="$1"

    if [ -z "$service" ]; then
        log_info "Restarting all services..."
        $DOCKER_COMPOSE restart
    else
        log_info "Restarting $service..."
        $DOCKER_COMPOSE restart "$service"
    fi
    log_success "Restart complete"
}

# View logs
cmd_logs() {
    cd "$ELK_DIR"
    local service="$1"

    if [ -z "$service" ]; then
        $DOCKER_COMPOSE logs -f --tail=100
    else
        $DOCKER_COMPOSE logs -f --tail=100 "$service"
    fi
}

# Health check
cmd_health() {
    echo ""
    echo "========================================"
    echo "  ELK Stack Health Check"
    echo "========================================"
    echo ""

    # Elasticsearch
    log_info "Checking Elasticsearch..."
    if curl -s -u "elastic:${ELASTIC_PASSWORD:-changeme}" \
        "http://localhost:9200/_cluster/health" >/dev/null 2>&1; then
        local health=$(curl -s -u "elastic:${ELASTIC_PASSWORD:-changeme}" \
            "http://localhost:9200/_cluster/health" | grep -o '"status":"[^"]*"' | cut -d'"' -f4)
        if [ "$health" = "green" ]; then
            log_success "Elasticsearch: healthy (green)"
        elif [ "$health" = "yellow" ]; then
            log_warn "Elasticsearch: degraded (yellow)"
        else
            log_error "Elasticsearch: unhealthy (red)"
        fi
    else
        log_error "Elasticsearch: not accessible"
    fi

    # Logstash
    log_info "Checking Logstash..."
    if curl -s "http://localhost:9600/_node/stats" >/dev/null 2>&1; then
        log_success "Logstash: healthy"
    else
        log_error "Logstash: not accessible"
    fi

    # Kibana
    log_info "Checking Kibana..."
    if curl -s "http://localhost:5601/api/status" >/dev/null 2>&1; then
        log_success "Kibana: healthy"
    else
        log_error "Kibana: not accessible"
    fi

    # Grafana
    log_info "Checking Grafana..."
    if curl -s "http://localhost:3000/api/health" >/dev/null 2>&1; then
        log_success "Grafana: healthy"
    else
        log_error "Grafana: not accessible"
    fi

    echo ""
}

# List indices
cmd_indices() {
    log_info "Elasticsearch Indices:"
    echo ""
    curl -s -u "elastic:${ELASTIC_PASSWORD:-changeme}" \
        "http://localhost:9200/_cat/indices/ipc-logs-*?v&s=index:desc&h=index,docs.count,store.size,health" | \
        head -20
}

# Quick search
cmd_search() {
    local query="$1"

    if [ -z "$query" ]; then
        log_error "Please provide a search query"
        echo "Example: $0 search \"validator:validator-1 AND ERROR\""
        exit 1
    fi

    log_info "Searching for: $query"
    echo ""

    curl -s -u "elastic:${ELASTIC_PASSWORD:-changeme}" \
        -X GET "http://localhost:9200/ipc-logs-*/_search?pretty" \
        -H 'Content-Type: application/json' \
        -d "{
          \"size\": 10,
          \"sort\": [{\"@timestamp\": \"desc\"}],
          \"query\": {
            \"query_string\": {
              \"query\": \"$query\"
            }
          },
          \"_source\": [\"@timestamp\", \"validator\", \"service\", \"log_level\", \"message\"]
        }" | jq '.hits.hits[]._source' 2>/dev/null || echo "Error: Could not parse results"
}

# Delete old logs
cmd_delete_old_logs() {
    local days="$1"

    if [ -z "$days" ]; then
        log_error "Please specify number of days"
        echo "Example: $0 delete-old-logs 30"
        exit 1
    fi

    log_warn "This will delete indices older than $days days"
    read -p "Are you sure? (yes/no): " confirm

    if [ "$confirm" != "yes" ]; then
        log_info "Cancelled"
        exit 0
    fi

    log_info "Deleting indices older than $days days..."

    curl -s -u "elastic:${ELASTIC_PASSWORD:-changeme}" \
        -X DELETE "http://localhost:9200/ipc-logs-*" \
        -H 'Content-Type: application/json' \
        -d "{
          \"query\": {
            \"range\": {
              \"@timestamp\": {
                \"lt\": \"now-${days}d\"
              }
            }
          }
        }" | jq '.' 2>/dev/null

    log_success "Old logs deleted"
}

# Backup
cmd_backup() {
    log_info "Creating Elasticsearch snapshot..."

    local snapshot_name="snapshot_$(date +%Y%m%d_%H%M%S)"

    curl -s -X PUT -u "elastic:${ELASTIC_PASSWORD:-changeme}" \
        "http://localhost:9200/_snapshot/backup/$snapshot_name?wait_for_completion=true" | \
        jq '.' 2>/dev/null

    log_success "Snapshot created: $snapshot_name"
}

# Update images
cmd_update() {
    cd "$ELK_DIR"
    log_info "Pulling latest Docker images..."
    $DOCKER_COMPOSE pull

    log_info "Restarting services with new images..."
    $DOCKER_COMPOSE up -d

    log_success "Update complete"
}

# Clean up
cmd_clean() {
    log_warn "This will remove unused Docker resources"
    read -p "Continue? (yes/no): " confirm

    if [ "$confirm" != "yes" ]; then
        log_info "Cancelled"
        exit 0
    fi

    log_info "Cleaning up Docker resources..."
    docker system prune -f
    log_success "Cleanup complete"
}

# Check Filebeat status
cmd_filebeat_status() {
    if [ ! -f "$HOME/github/ipc/scripts/ipc-subnet-manager/ipc-subnet-config.yml" ]; then
        log_error "IPC config not found"
        exit 1
    fi

    echo ""
    echo "========================================"
    echo "  Filebeat Status on Validators"
    echo "========================================"
    echo ""

    # Get validator IPs from config
    local validator_ips=$(yq eval '.validators[].ip' \
        "$HOME/github/ipc/scripts/ipc-subnet-manager/ipc-subnet-config.yml" 2>/dev/null)
    local validator_names=$(yq eval '.validators[].name' \
        "$HOME/github/ipc/scripts/ipc-subnet-manager/ipc-subnet-config.yml" 2>/dev/null)
    local validator_users=$(yq eval '.validators[].ssh_user' \
        "$HOME/github/ipc/scripts/ipc-subnet-manager/ipc-subnet-config.yml" 2>/dev/null)

    local idx=0
    while read -r ip; do
        local name=$(echo "$validator_names" | sed -n "$((idx+1))p")
        local user=$(echo "$validator_users" | sed -n "$((idx+1))p")

        log_info "Checking $name ($ip)..."

        local status=$(ssh -o ConnectTimeout=5 -o StrictHostKeyChecking=no \
            "$user@$ip" "sudo systemctl is-active filebeat" 2>/dev/null || echo "error")

        if [ "$status" = "active" ]; then
            log_success "$name: Filebeat is running"
        else
            log_error "$name: Filebeat is not running (status: $status)"
        fi

        idx=$((idx+1))
    done <<< "$validator_ips"

    echo ""
}

# Main command dispatcher
main() {
    local command="$1"
    shift

    check_docker_compose

    case "$command" in
        status)
            cmd_status "$@"
            ;;
        start)
            cmd_start "$@"
            ;;
        stop)
            cmd_stop "$@"
            ;;
        restart)
            cmd_restart "$@"
            ;;
        logs)
            cmd_logs "$@"
            ;;
        health)
            cmd_health "$@"
            ;;
        indices)
            cmd_indices "$@"
            ;;
        search)
            cmd_search "$@"
            ;;
        delete-old-logs)
            cmd_delete_old_logs "$@"
            ;;
        backup)
            cmd_backup "$@"
            ;;
        update)
            cmd_update "$@"
            ;;
        clean)
            cmd_clean "$@"
            ;;
        filebeat-status)
            cmd_filebeat_status "$@"
            ;;
        help|--help|-h)
            show_help
            ;;
        *)
            log_error "Unknown command: $command"
            echo ""
            show_help
            exit 1
            ;;
    esac
}

# Run main function
if [ $# -eq 0 ]; then
    show_help
    exit 0
fi

main "$@"

