# ELK Stack Log Aggregation for IPC Validators

Complete log aggregation solution for IPC (InterPlanetary Consensus) validator nodes using the ELK (Elasticsearch, Logstash, Kibana) stack with Grafana.

## 📋 Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Prerequisites](#prerequisites)
- [Quick Start](#quick-start)
- [Detailed Setup](#detailed-setup)
- [Configuration](#configuration)
- [Usage](#usage)
- [Troubleshooting](#troubleshooting)
- [Maintenance](#maintenance)
- [Security](#security)

## Overview

This setup provides centralized log aggregation for 3 IPC validator nodes running on Google Cloud Platform (GCP). It includes:

- **Filebeat**: Lightweight log shipper running on each validator
- **Logstash**: Log processing pipeline with IPC-specific parsing
- **Elasticsearch**: Log storage and search engine
- **Kibana**: Web UI for log visualization and analysis
- **Grafana**: Alternative visualization with Elasticsearch datasource

### Features

- ✅ Automatic log collection from systemd services (`ipc-node`, `ipc-relayer`)
- ✅ File-based log collection from node home directories
- ✅ IPC-specific log parsing (CometBFT, checkpoints, transactions)
- ✅ Real-time log streaming and search
- ✅ Pre-built dashboards and visualizations
- ✅ 90-day log retention with Index Lifecycle Management (ILM)
- ✅ Automatic log rotation and compression

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     Validator Nodes (GCP)                       │
├─────────────────┬─────────────────┬─────────────────────────────┤
│  Validator-1    │  Validator-2    │  Validator-3                │
│  ┌───────────┐  │  ┌───────────┐  │  ┌───────────┐              │
│  │ Filebeat  │  │  │ Filebeat  │  │  │ Filebeat  │              │
│  │ (systemd) │  │  │ (systemd) │  │  │ (systemd) │              │
│  └─────┬─────┘  │  └─────┬─────┘  │  └─────┬─────┘              │
│        │        │        │        │        │                    │
│  • systemd logs │  • systemd logs │  • systemd logs            │
│  • file logs    │  • file logs    │  • file logs               │
└────────┼────────┴────────┼────────┴────────┼────────────────────┘
         │                 │                 │
         │                 │                 │
         └─────────────────┼─────────────────┘
                           │ Port 5044 (Beats protocol)
                           ▼
         ┌─────────────────────────────────────┐
         │      Central Logging Server         │
         │      (GCP Instance or Local)        │
         ├─────────────────────────────────────┤
         │  ┌──────────────────────────────┐   │
         │  │        Logstash              │   │
         │  │  • Parse logs                │   │
         │  │  • Extract fields            │   │
         │  │  • Enrich metadata           │   │
         │  └──────────┬───────────────────┘   │
         │             ▼                        │
         │  ┌──────────────────────────────┐   │
         │  │      Elasticsearch           │   │
         │  │  • Store logs                │   │
         │  │  • Index & search            │   │
         │  │  • ILM policies              │   │
         │  └──────────┬───────────────────┘   │
         │             │                        │
         │    ┌────────┴────────┐               │
         │    ▼                 ▼               │
         │  ┌─────────┐    ┌─────────┐         │
         │  │ Kibana  │    │ Grafana │         │
         │  │:5601    │    │:3000    │         │
         │  └─────────┘    └─────────┘         │
         └─────────────────────────────────────┘
                      │
                      ▼
                 Your Browser
```

## Prerequisites

### Central Server Requirements

**Minimum Specs:**
- **CPU**: 2 vCPUs
- **RAM**: 4GB (8GB recommended for production)
- **Disk**: 50GB SSD minimum (adjust based on log volume)
- **OS**: Ubuntu 22.04 LTS or similar
- **Network**: Static IP, ports 5044, 5601, 3000 open

**Software:**
- Docker 24.0+
- Docker Compose 2.0+
- curl, openssl

### Validator Node Requirements

- SSH access with sudo privileges
- Systemd (already configured)
- Internet access to download Filebeat
- Outbound access to central server on port 5044

### Your Machine

- SSH access to all validators
- `yq` for YAML parsing: `brew install yq` (macOS) or `snap install yq` (Linux)
- IPC subnet config file: `scripts/ipc-subnet-manager/ipc-subnet-config.yml`

## Quick Start

### 1. Setup Central Server

```bash
# SSH into your central logging server
cd /path/to/ipc/infra/elk-logging

# Run setup script
./scripts/setup-central-server.sh
```

This will:
- Install and configure ELK stack
- Generate secure passwords
- Start all services
- Setup Elasticsearch index templates
- Display access credentials

**Save the credentials displayed at the end!**

### 2. Configure GCP Firewall

Allow incoming traffic to your central server:

```bash
# From your local machine
gcloud compute firewall-rules create allow-elk-logging \
  --allow tcp:5044,tcp:5601,tcp:3000 \
  --source-ranges 0.0.0.0/0 \
  --description "Allow ELK logging traffic"

# For production, restrict source-ranges to your validator IPs
```

### 3. Deploy Filebeat to Validators

```bash
# From your local machine
cd /path/to/ipc/infra/elk-logging

# Set your config path (if not default)
export IPC_CONFIG="$HOME/github/ipc/scripts/ipc-subnet-manager/ipc-subnet-config.yml"

# Deploy to all validators
./scripts/deploy-filebeat.sh
```

### 4. Verify Log Flow

```bash
# Wait 2-3 minutes for logs to start flowing
sleep 180

# Check log flow
./scripts/check-log-flow.sh
```

### 5. Access Kibana

1. Open browser: `http://<SERVER_IP>:5601`
2. Login with credentials from setup
3. Go to **Management** > **Stack Management** > **Kibana** > **Data Views**
4. Create data view: `ipc-logs-*`
5. Go to **Analytics** > **Discover** to view logs

## Detailed Setup

### Central Server Setup

#### Manual Docker Compose Setup

If you prefer manual setup:

```bash
cd /path/to/ipc/infra/elk-logging

# Create .env file
cp .env.example .env
# Edit .env and set passwords

# Configure system settings
sudo sysctl -w vm.max_map_count=262144
echo "vm.max_map_count=262144" | sudo tee -a /etc/sysctl.conf

# Start services
docker-compose up -d

# View logs
docker-compose logs -f
```

#### Service Management

```bash
# Stop all services
docker-compose down

# Restart a specific service
docker-compose restart elasticsearch

# View service logs
docker-compose logs -f logstash

# Check service status
docker-compose ps
```

### Filebeat Configuration

The Filebeat configuration template (`filebeat/filebeat.yml.template`) is automatically customized for each validator during deployment. It includes:

**Inputs:**
- Systemd journal for `ipc-node.service`
- Systemd journal for `ipc-relayer.service`
- File logs from `~/.ipc-node/logs/`
- CometBFT logs

**Processors:**
- Add host metadata
- Add cloud metadata (GCP)
- Add subnet information
- Drop empty lines

**Output:**
- Sends to Logstash on port 5044
- Includes load balancing and retry logic

### Logstash Pipeline

The Logstash pipeline (`logstash/pipeline/ipc-logs.conf`) performs:

**Parsing:**
- Extracts log levels (ERROR, WARN, INFO, DEBUG)
- Parses CometBFT consensus messages (block height, rounds, votes)
- Parses checkpoint relayer messages
- Parses Ethereum/FEVM transactions
- Extracts timestamps

**Enrichment:**
- Tags errors and warnings
- Adds metadata from Filebeat
- Normalizes field names

**Output:**
- Writes to Elasticsearch with daily indices
- Index pattern: `ipc-logs-<hostname>-YYYY.MM.DD`

## Configuration

### Environment Variables

Edit `.env` file on central server:

```bash
# Elasticsearch
ELASTIC_PASSWORD=your-strong-password

# Kibana
KIBANA_ENCRYPTION_KEY=min-32-char-random-string

# Grafana
GRAFANA_USER=admin
GRAFANA_PASSWORD=your-grafana-password

# Server
SERVER_IP=your-server-ip
```

### Log Retention

Edit `elasticsearch/ilm-policy.json` to change retention:

```json
{
  "policy": {
    "phases": {
      "hot": { "min_age": "0ms" },      // Active indices
      "warm": { "min_age": "7d" },      // Older, read-only
      "cold": { "min_age": "30d" },     // Very old, frozen
      "delete": { "min_age": "90d" }    // Delete after 90 days
    }
  }
}
```

Apply changes:

```bash
curl -X PUT "http://localhost:9200/_ilm/policy/ipc-logs-policy" \
  -u "elastic:${ELASTIC_PASSWORD}" \
  -H 'Content-Type: application/json' \
  -d @elasticsearch/ilm-policy.json
```

### Resource Limits

Edit `docker-compose.yml` to adjust resource allocation:

```yaml
services:
  elasticsearch:
    environment:
      - "ES_JAVA_OPTS=-Xms4g -Xmx4g"  # Increase heap size

  logstash:
    environment:
      - "LS_JAVA_OPTS=-Xms2g -Xmx2g"  # Increase heap size
```

## Usage

### Kibana

#### Create Data View

1. Go to **Management** > **Stack Management** > **Kibana** > **Data Views**
2. Click **Create data view**
3. Name: `IPC Validator Logs`
4. Index pattern: `ipc-logs-*`
5. Timestamp field: `@timestamp`
6. Click **Create data view**

#### View Logs

1. Go to **Analytics** > **Discover**
2. Select **IPC Validator Logs** data view
3. Use filters and queries to search logs

#### Useful KQL Queries

```
# All errors
log_level:"ERROR"

# Logs from specific validator
validator:"validator-1"

# CometBFT consensus logs
tags:"cometbft_consensus"

# Checkpoint relayer logs
service:"ipc-relayer"

# High block heights
block_height > 1000

# Recent errors (last 15 minutes)
log_level:"ERROR" AND @timestamp >= now-15m

# Failed checkpoints
service:"ipc-relayer" AND message:*failed*
```

#### Create Visualizations

1. Go to **Analytics** > **Visualize Library**
2. Click **Create visualization**
3. Choose visualization type (Line, Bar, Pie, etc.)
4. Select data view and configure

**Example: Log Volume by Validator**
- Type: Vertical bar chart
- Y-axis: Count
- X-axis: Terms aggregation on `validator.keyword`
- Split series: Terms on `log_level.keyword`

#### Create Dashboards

1. Go to **Analytics** > **Dashboard**
2. Click **Create dashboard**
3. Add visualizations
4. Save dashboard

### Grafana

#### Access Grafana

1. Open: `http://<SERVER_IP>:3000`
2. Login with Grafana credentials
3. Elasticsearch datasource is pre-configured

#### Create Dashboard

1. Click **+** > **Dashboard**
2. Add panel
3. Select **Elasticsearch-IPC-Logs** datasource
4. Configure query using Lucene syntax

### CLI Tools

#### Check Elasticsearch Health

```bash
curl -u "elastic:${ELASTIC_PASSWORD}" \
  "http://localhost:9200/_cluster/health?pretty"
```

#### View Indices

```bash
curl -u "elastic:${ELASTIC_PASSWORD}" \
  "http://localhost:9200/_cat/indices/ipc-logs-*?v"
```

#### Search Logs

```bash
curl -u "elastic:${ELASTIC_PASSWORD}" \
  -X GET "http://localhost:9200/ipc-logs-*/_search?pretty" \
  -H 'Content-Type: application/json' \
  -d '{
    "size": 10,
    "sort": [{"@timestamp": "desc"}],
    "query": {
      "match": {
        "validator": "validator-1"
      }
    }
  }'
```

## Troubleshooting

### No Logs in Elasticsearch

**Check 1: Filebeat is running**
```bash
ssh validator-1 'sudo systemctl status filebeat'
```

**Check 2: Filebeat logs**
```bash
ssh validator-1 'sudo journalctl -u filebeat -n 50 --no-pager'
```

**Check 3: Network connectivity**
```bash
ssh validator-1 "telnet <SERVER_IP> 5044"
```

**Check 4: Logstash receiving logs**
```bash
curl "http://localhost:9600/_node/stats/pipelines?pretty"
```

### Elasticsearch Not Starting

**Check logs:**
```bash
docker-compose logs elasticsearch
```

**Common issues:**
- `vm.max_map_count` too low → Run: `sudo sysctl -w vm.max_map_count=262144`
- Out of disk space → Free up space or add more storage
- Insufficient memory → Increase RAM or reduce heap size

### Kibana Connection Error

**Wait for Elasticsearch:**
```bash
# Check if Elasticsearch is healthy
curl -u "elastic:${ELASTIC_PASSWORD}" \
  "http://localhost:9200/_cluster/health"
```

**Restart Kibana:**
```bash
docker-compose restart kibana
```

### Logstash Pipeline Errors

**View logs:**
```bash
docker-compose logs logstash | grep ERROR
```

**Validate pipeline config:**
```bash
docker-compose exec logstash bin/logstash --config.test_and_exit \
  -f /usr/share/logstash/pipeline/ipc-logs.conf
```

### High Disk Usage

**Check index sizes:**
```bash
curl -u "elastic:${ELASTIC_PASSWORD}" \
  "http://localhost:9200/_cat/indices/ipc-logs-*?v&s=store.size:desc"
```

**Manually delete old indices:**
```bash
curl -X DELETE -u "elastic:${ELASTIC_PASSWORD}" \
  "http://localhost:9200/ipc-logs-validator-1-2024.10.01"
```

**Adjust ILM policy** to delete logs sooner (see Configuration section)

## Maintenance

### Backup Elasticsearch Data

```bash
# Create snapshot repository
curl -X PUT -u "elastic:${ELASTIC_PASSWORD}" \
  "http://localhost:9200/_snapshot/backup" \
  -H 'Content-Type: application/json' \
  -d '{
    "type": "fs",
    "settings": {
      "location": "/usr/share/elasticsearch/backups"
    }
  }'

# Create snapshot
curl -X PUT -u "elastic:${ELASTIC_PASSWORD}" \
  "http://localhost:9200/_snapshot/backup/snapshot_$(date +%Y%m%d)?wait_for_completion=true"
```

### Update Filebeat

```bash
# On each validator
ssh validator-1 'sudo systemctl stop filebeat'
ssh validator-1 'sudo curl -L -o /usr/local/bin/filebeat \
  https://artifacts.elastic.co/downloads/beats/filebeat/filebeat-8.11.0-linux-amd64'
ssh validator-1 'sudo chmod +x /usr/local/bin/filebeat'
ssh validator-1 'sudo systemctl start filebeat'
```

### Monitor Stack Health

Create a monitoring script:

```bash
#!/bin/bash
# Check all services
docker-compose ps
curl -s "http://localhost:9200/_cluster/health" | jq '.status'
curl -s "http://localhost:9600/_node/stats" | jq '.pipelines'
```

### Log Rotation

Elasticsearch automatically rotates indices based on ILM policy. No manual intervention needed.

## Security

### Production Security Checklist

- [ ] Enable TLS/SSL for Elasticsearch, Logstash, Kibana
- [ ] Use strong passwords (generated by setup script)
- [ ] Restrict firewall rules to specific IPs only
- [ ] Enable Elasticsearch security features (already enabled)
- [ ] Use TLS for Filebeat → Logstash communication
- [ ] Regular security updates for all components
- [ ] Enable authentication for Grafana (already enabled)
- [ ] Backup encryption keys securely

### Enable TLS for Filebeat → Logstash

1. Generate certificates (on central server)
2. Update Logstash input to require SSL
3. Update Filebeat output to use SSL
4. Redeploy Filebeat configuration

(Detailed TLS setup guide available on request)

## Resources

- [Elasticsearch Documentation](https://www.elastic.co/guide/en/elasticsearch/reference/current/index.html)
- [Logstash Documentation](https://www.elastic.co/guide/en/logstash/current/index.html)
- [Filebeat Documentation](https://www.elastic.co/guide/en/beats/filebeat/current/index.html)
- [Kibana Documentation](https://www.elastic.co/guide/en/kibana/current/index.html)
- [IPC Project](https://github.com/consensus-shipyard/ipc)

## Support

For issues or questions:
1. Check this documentation
2. View Troubleshooting section
3. Check service logs: `docker-compose logs -f`
4. Review IPC subnet manager documentation

## License

This configuration is part of the IPC project and follows the same license terms.

