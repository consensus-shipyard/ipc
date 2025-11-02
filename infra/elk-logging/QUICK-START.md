# ELK Stack Quick Start Guide

Get your IPC validator log aggregation up and running in 30 minutes.

## Prerequisites

- ✅ Central server (GCP instance or local machine)
- ✅ Docker and Docker Compose installed on central server
- ✅ SSH access to all 3 validators
- ✅ `yq` installed on your machine: `brew install yq` (macOS)

## Step-by-Step Setup

### Step 1: Setup Central Server (10 minutes)

SSH into your central logging server:

```bash
# Clone or navigate to IPC repo
cd /path/to/ipc/infra/elk-logging

# Run automated setup
./scripts/setup-central-server.sh
```

**📝 Important:** Save the credentials displayed at the end!

Expected output:
```
======================================
  ELK Stack Setup Complete! 🎉
======================================

Service URLs:
  Elasticsearch: http://YOUR_IP:9200
  Kibana:        http://YOUR_IP:5601
  Grafana:       http://YOUR_IP:3000

Credentials:
  Elasticsearch:
    Username: elastic
    Password: [generated-password]

  Kibana:
    Username: elastic
    Password: [same-as-above]

  Grafana:
    Username: admin
    Password: [generated-password]
======================================
```

### Step 2: Configure Firewall (5 minutes)

**For GCP:**

```bash
# Allow Filebeat to connect to Logstash
gcloud compute firewall-rules create allow-elk-filebeat \
  --allow tcp:5044 \
  --source-ranges <VALIDATOR_1_IP>,<VALIDATOR_2_IP>,<VALIDATOR_3_IP> \
  --description "Allow Filebeat to Logstash"

# Allow you to access Kibana (replace YOUR_IP)
gcloud compute firewall-rules create allow-kibana \
  --allow tcp:5601,tcp:3000 \
  --source-ranges <YOUR_IP>/32 \
  --description "Allow Kibana/Grafana access"
```

**For other cloud providers:**

Open ports in security groups:
- `5044` (Filebeat → Logstash) from validator IPs
- `5601` (Kibana) from your IP
- `3000` (Grafana) from your IP

### Step 3: Deploy Filebeat to Validators (10 minutes)

From your local machine:

```bash
cd /path/to/ipc/infra/elk-logging

# Set config path (adjust if yours is different)
export IPC_CONFIG="$HOME/github/ipc/scripts/ipc-subnet-manager/ipc-subnet-config.yml"

# Deploy to all validators
./scripts/deploy-filebeat.sh
```

Expected output:
```
======================================
  IPC Filebeat Deployment
======================================

Loading configuration...
Found 3 validators

======================================
  Deploying to validator-1
======================================
✓ Filebeat downloaded and installed
✓ Config deployed
✓ Systemd service installed
✓ Filebeat started
✓ Deployment complete for validator-1

[... same for validator-2 and validator-3 ...]

======================================
  Deployment Summary
======================================
  Successful: 3
  Failed: 0

✓ All validators deployed successfully!
```

### Step 4: Verify Logs Are Flowing (5 minutes)

Wait 2-3 minutes for logs to start flowing:

```bash
# Wait a bit
sleep 180

# Check log flow
./scripts/check-log-flow.sh
```

Expected output:
```
======================================
  ELK Log Flow Check
======================================

✓ Elasticsearch is running
✓ Logstash is running
✓ Found IPC log indices:
  - ipc-logs-validator-1-2025.11.02
  - ipc-logs-validator-2-2025.11.02
  - ipc-logs-validator-3-2025.11.02
✓ Found 1247 log documents
✓ Received 89 logs in the last 5 minutes

======================================
  Summary
======================================
✓ ELK stack is receiving logs!

Access your logs:
  Kibana:  http://YOUR_IP:5601
  Grafana: http://YOUR_IP:3000
```

### Step 5: Access Kibana (5 minutes)

1. **Open Kibana**: `http://YOUR_SERVER_IP:5601`

2. **Login** with credentials from Step 1

3. **Create Data View:**
   - Click hamburger menu (☰) → Management → Stack Management
   - Under Kibana, click "Data Views"
   - Click "Create data view"
   - Name: `IPC Validator Logs`
   - Index pattern: `ipc-logs-*`
   - Timestamp field: `@timestamp`
   - Click "Create data view"

4. **View Logs:**
   - Click hamburger menu (☰) → Analytics → Discover
   - Select "IPC Validator Logs" data view
   - You should see logs streaming in!

## Quick Usage Examples

### Search Logs in Kibana

#### View all errors:
```
log_level:"ERROR"
```

#### View logs from specific validator:
```
validator:"validator-1"
```

#### View CometBFT consensus logs:
```
tags:"cometbft_consensus"
```

#### View recent checkpoint submissions:
```
service:"ipc-relayer" AND message:*checkpoint*
```

#### Combine filters:
```
validator:"validator-1" AND log_level:"ERROR" AND @timestamp >= now-1h
```

### Create a Simple Visualization

1. Go to Analytics → Visualize Library
2. Click "Create visualization"
3. Select "Lens"
4. Configure:
   - **Vertical axis**: Count
   - **Horizontal axis**: Date histogram on `@timestamp`
   - **Break down by**: `validator.keyword`
5. Save as "Log Volume by Validator"

### Create Your First Dashboard

1. Go to Analytics → Dashboard
2. Click "Create dashboard"
3. Click "Add visualization"
4. Select "Log Volume by Validator"
5. Add more visualizations as needed
6. Click "Save" → Name: "IPC Validator Overview"

## Common Quick Fixes

### No logs appearing?

```bash
# Check Filebeat on each validator
ssh validator-1 'sudo systemctl status filebeat'
ssh validator-2 'sudo systemctl status filebeat'
ssh validator-3 'sudo systemctl status filebeat'

# Check Filebeat logs
ssh validator-1 'sudo journalctl -u filebeat -n 20'
```

### Can't connect to Kibana?

```bash
# Check services are running
docker-compose ps

# Check Kibana specifically
docker-compose logs kibana | tail -20
```

### Elasticsearch not starting?

```bash
# Check if vm.max_map_count is set
sysctl vm.max_map_count

# Should be 262144 or higher
# If not:
sudo sysctl -w vm.max_map_count=262144

# Restart Elasticsearch
docker-compose restart elasticsearch
```

## Next Steps

Now that your ELK stack is running:

1. **Explore Kibana Features:**
   - Create more visualizations
   - Build comprehensive dashboards
   - Set up alerts (requires additional setup)

2. **Optimize Performance:**
   - Review ILM policies
   - Adjust retention periods
   - Monitor disk usage

3. **Secure Your Stack:**
   - Enable TLS/SSL
   - Restrict firewall rules
   - Set up proper authentication

4. **Read Full Documentation:**
   - [README.md](README.md) - Complete guide
   - [TROUBLESHOOTING.md](TROUBLESHOOTING.md) - Detailed troubleshooting

## Useful Commands

```bash
# View all service logs
docker-compose logs -f

# Restart all services
docker-compose restart

# Stop all services
docker-compose down

# Start all services
docker-compose up -d

# Check log flow
./scripts/check-log-flow.sh

# View Elasticsearch indices
curl -u "elastic:${ELASTIC_PASSWORD}" \
  "http://localhost:9200/_cat/indices/ipc-logs-*?v"
```

## Getting Help

If something goes wrong:

1. Check [TROUBLESHOOTING.md](TROUBLESHOOTING.md)
2. View service logs: `docker-compose logs <service>`
3. Run diagnostics: `./scripts/check-log-flow.sh`

---

**That's it!** You now have a fully functional ELK stack aggregating logs from all your IPC validators. 🎉

