#!/bin/bash
# Check if logs are flowing from validators to Elasticsearch
# This script verifies the entire ELK pipeline

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

# Load environment
if [ ! -f "$ELK_DIR/.env" ]; then
    log_error ".env file not found"
    exit 1
fi
source "$ELK_DIR/.env"

echo ""
echo "========================================"
echo "  ELK Log Flow Check"
echo "========================================"
echo ""

# Check Elasticsearch is running
log_info "Checking Elasticsearch..."
if curl -s -u "elastic:${ELASTIC_PASSWORD}" "http://localhost:9200/_cluster/health" >/dev/null 2>&1; then
    log_success "Elasticsearch is running"
else
    log_error "Elasticsearch is not accessible"
    exit 1
fi

# Check Logstash is running
log_info "Checking Logstash..."
if curl -s "http://localhost:9600/_node/stats" >/dev/null 2>&1; then
    log_success "Logstash is running"
else
    log_error "Logstash is not accessible"
    exit 1
fi

# Check if indices exist
log_info "Checking for IPC log indices..."
indices=$(curl -s -u "elastic:${ELASTIC_PASSWORD}" "http://localhost:9200/_cat/indices/ipc-logs-*?h=index" 2>/dev/null)

if [ -z "$indices" ]; then
    log_warn "No IPC log indices found yet"
    log_info "Logs may take a few minutes to appear after Filebeat deployment"
else
    log_success "Found IPC log indices:"
    echo "$indices" | while read index; do
        echo "  - $index"
    done
fi

# Check document count
log_info "Checking document count..."
doc_count=$(curl -s -u "elastic:${ELASTIC_PASSWORD}" "http://localhost:9200/ipc-logs-*/_count" 2>/dev/null | grep -o '"count":[0-9]*' | cut -d: -f2)

if [ -z "$doc_count" ] || [ "$doc_count" = "0" ]; then
    log_warn "No documents found in IPC logs"
    log_info "This is normal if Filebeat was just deployed"
else
    log_success "Found $doc_count log documents"
fi

# Check recent logs
log_info "Checking for recent logs (last 5 minutes)..."
recent_logs=$(curl -s -u "elastic:${ELASTIC_PASSWORD}" -X GET "http://localhost:9200/ipc-logs-*/_search" \
    -H 'Content-Type: application/json' \
    -d '{
      "size": 5,
      "sort": [{"@timestamp": {"order": "desc"}}],
      "query": {
        "range": {
          "@timestamp": {
            "gte": "now-5m"
          }
        }
      },
      "_source": ["@timestamp", "validator", "service", "message"]
    }' 2>/dev/null)

hit_count=$(echo "$recent_logs" | grep -o '"total":{"value":[0-9]*' | cut -d: -f3)

if [ -z "$hit_count" ] || [ "$hit_count" = "0" ]; then
    log_warn "No logs received in the last 5 minutes"
    log_info "Troubleshooting steps:"
    echo "  1. Check Filebeat is running on validators:"
    echo "     ssh <validator> 'sudo systemctl status filebeat'"
    echo "  2. Check Filebeat logs:"
    echo "     ssh <validator> 'sudo journalctl -u filebeat -n 50'"
    echo "  3. Check network connectivity to Logstash (port 5044)"
else
    log_success "Received $hit_count logs in the last 5 minutes"
    echo ""
    log_info "Recent log samples:"
    echo "$recent_logs" | python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    for hit in data.get('hits', {}).get('hits', []):
        source = hit.get('_source', {})
        print(f\"  [{source.get('validator', 'unknown')}] {source.get('service', 'unknown')}: {source.get('message', '')[:80]}...\")
except:
    pass
" 2>/dev/null || echo "  (Could not parse sample logs)"
fi

# Check logs per validator
log_info "Checking logs per validator..."
validator_stats=$(curl -s -u "elastic:${ELASTIC_PASSWORD}" -X GET "http://localhost:9200/ipc-logs-*/_search" \
    -H 'Content-Type: application/json' \
    -d '{
      "size": 0,
      "aggs": {
        "validators": {
          "terms": {
            "field": "validator.keyword",
            "size": 10
          }
        }
      }
    }' 2>/dev/null)

echo "$validator_stats" | python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    buckets = data.get('aggregations', {}).get('validators', {}).get('buckets', [])
    if buckets:
        print('  Validator log counts:')
        for bucket in buckets:
            print(f\"    {bucket['key']}: {bucket['doc_count']} logs\")
    else:
        print('  No validator data available yet')
except:
    print('  Could not parse validator stats')
" 2>/dev/null || echo "  (Could not parse validator stats)"

# Check Logstash stats
log_info "Checking Logstash pipeline stats..."
logstash_stats=$(curl -s "http://localhost:9600/_node/stats/pipelines" 2>/dev/null)

events_in=$(echo "$logstash_stats" | python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    for pipeline in data.get('pipelines', {}).values():
        events = pipeline.get('events', {})
        print(events.get('in', 0))
        break
except:
    print(0)
" 2>/dev/null)

events_out=$(echo "$logstash_stats" | python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    for pipeline in data.get('pipelines', {}).values():
        events = pipeline.get('events', {})
        print(events.get('out', 0))
        break
except:
    print(0)
" 2>/dev/null)

log_info "Logstash pipeline:"
echo "  Events in:  $events_in"
echo "  Events out: $events_out"

echo ""
echo "========================================"
echo "  Summary"
echo "========================================"

if [ ! -z "$doc_count" ] && [ "$doc_count" -gt 0 ]; then
    log_success "ELK stack is receiving logs!"
    echo ""
    echo "Access your logs:"
    echo "  Kibana:  http://${SERVER_IP}:5601"
    echo "  Grafana: http://${SERVER_IP}:3000"
    echo ""
    echo "In Kibana:"
    echo "  1. Go to Management > Stack Management > Kibana > Data Views"
    echo "  2. Create data view with pattern: ipc-logs-*"
    echo "  3. Go to Analytics > Discover to view logs"
else
    log_warn "No logs received yet"
    echo ""
    echo "If Filebeat was just deployed, wait a few minutes and run this script again."
    echo "If still no logs after 5 minutes, check:"
    echo "  1. Filebeat service status on validators"
    echo "  2. Network connectivity (port 5044 open)"
    echo "  3. Filebeat logs: sudo journalctl -u filebeat -n 50"
fi

echo "========================================"

