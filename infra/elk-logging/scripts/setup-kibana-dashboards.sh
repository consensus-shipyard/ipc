#!/bin/bash
# Setup Kibana index patterns and dashboards

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ELK_DIR="$(dirname "$SCRIPT_DIR")"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

# Load environment
if [ ! -f "$ELK_DIR/.env" ]; then
    echo "Error: .env file not found"
    exit 1
fi
source "$ELK_DIR/.env"

echo ""
echo "========================================"
echo "  Setting up Kibana Dashboards"
echo "========================================"
echo ""

log_info "Creating index pattern in Kibana..."

# Wait for Kibana to be ready
log_info "Waiting for Kibana to be ready..."
for i in {1..30}; do
    if curl -s -u "elastic:${ELASTIC_PASSWORD}" "http://localhost:5601/api/status" | grep -q "available"; then
        log_success "Kibana is ready"
        break
    fi
    if [ $i -eq 30 ]; then
        echo "Error: Kibana not ready after 2.5 minutes"
        exit 1
    fi
    sleep 5
done

# Create data view (index pattern)
log_info "Creating data view for ipc-logs-*..."

curl -X POST "http://localhost:5601/api/data_views/data_view" \
    -u "elastic:${ELASTIC_PASSWORD}" \
    -H 'kbn-xsrf: true' \
    -H 'Content-Type: application/json' \
    -d '{
      "data_view": {
        "title": "ipc-logs-*",
        "timeFieldName": "@timestamp",
        "name": "IPC Validator Logs"
      }
    }' >/dev/null 2>&1

if [ $? -eq 0 ]; then
    log_success "Data view created successfully"
else
    log_info "Data view may already exist (this is OK)"
fi

# Import saved objects if available
if [ -f "$ELK_DIR/kibana/dashboards/ipc-validator-overview.ndjson" ]; then
    log_info "Importing dashboards..."

    curl -X POST "http://localhost:5601/api/saved_objects/_import" \
        -u "elastic:${ELASTIC_PASSWORD}" \
        -H "kbn-xsrf: true" \
        --form file=@"$ELK_DIR/kibana/dashboards/ipc-validator-overview.ndjson" \
        >/dev/null 2>&1

    if [ $? -eq 0 ]; then
        log_success "Dashboards imported"
    else
        log_info "Dashboard import may have failed (you can create manually)"
    fi
fi

echo ""
log_success "Kibana setup complete!"
echo ""
echo "Access Kibana at: http://${SERVER_IP}:5601"
echo "Username: elastic"
echo "Password: ${ELASTIC_PASSWORD}"
echo ""
echo "Next steps:"
echo "  1. Go to Analytics > Discover to view logs"
echo "  2. Go to Analytics > Dashboard to view pre-built dashboards"
echo "  3. Create custom visualizations as needed"
echo ""

