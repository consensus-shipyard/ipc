# ELK Stack Troubleshooting Guide

Comprehensive troubleshooting guide for the IPC ELK logging stack.

## Table of Contents

- [Quick Diagnostics](#quick-diagnostics)
- [Central Server Issues](#central-server-issues)
- [Validator Node Issues](#validator-node-issues)
- [Network Issues](#network-issues)
- [Performance Issues](#performance-issues)
- [Data Issues](#data-issues)
- [Common Error Messages](#common-error-messages)

## Quick Diagnostics

Run these commands to quickly diagnose issues:

```bash
# Check all services status
cd /path/to/elk-logging
docker-compose ps

# Check log flow
./scripts/check-log-flow.sh

# Check Elasticsearch cluster health
curl -u "elastic:${ELASTIC_PASSWORD}" \
  "http://localhost:9200/_cluster/health?pretty"

# Check Logstash pipeline stats
curl "http://localhost:9600/_node/stats/pipelines?pretty"

# Check Filebeat on validator
ssh validator-1 'sudo systemctl status filebeat'
```

## Central Server Issues

### Elasticsearch Won't Start

**Symptom:** Elasticsearch container exits immediately or won't start.

**Check logs:**
```bash
docker-compose logs elasticsearch | tail -50
```

**Common causes and fixes:**

#### 1. vm.max_map_count Too Low

**Error:** `max virtual memory areas vm.max_map_count [65530] is too low`

**Fix:**
```bash
sudo sysctl -w vm.max_map_count=262144
echo "vm.max_map_count=262144" | sudo tee -a /etc/sysctl.conf
docker-compose restart elasticsearch
```

#### 2. Insufficient Memory

**Error:** `Java heap space` or `OutOfMemoryError`

**Fix:** Reduce heap size in `docker-compose.yml`:
```yaml
elasticsearch:
  environment:
    - "ES_JAVA_OPTS=-Xms1g -Xmx1g"  # Reduce from 2g
```

Then restart:
```bash
docker-compose restart elasticsearch
```

#### 3. Disk Space Full

**Error:** `no space left on device`

**Check disk usage:**
```bash
df -h
docker system df
```

**Fix:** Free up space or delete old indices:
```bash
# Delete old indices
curl -X DELETE -u "elastic:${ELASTIC_PASSWORD}" \
  "http://localhost:9200/ipc-logs-*-2024.10.*"

# Clean up Docker
docker system prune -a
```

#### 4. Permission Denied

**Error:** `AccessDeniedException` or permission errors

**Fix:**
```bash
sudo chown -R 1000:1000 elasticsearch/data
docker-compose restart elasticsearch
```

### Logstash Won't Start

**Check logs:**
```bash
docker-compose logs logstash | tail -50
```

#### 1. Pipeline Configuration Error

**Error:** `Invalid configuration` or syntax errors

**Test pipeline:**
```bash
docker-compose run --rm logstash \
  bin/logstash --config.test_and_exit \
  -f /usr/share/logstash/pipeline/ipc-logs.conf
```

**Fix:** Review and fix `logstash/pipeline/ipc-logs.conf`

#### 2. Cannot Connect to Elasticsearch

**Error:** `Connection refused` to Elasticsearch

**Check:**
```bash
# From logstash container
docker-compose exec logstash curl http://elasticsearch:9200
```

**Fix:** Ensure Elasticsearch is running and healthy first.

#### 3. Port Already in Use

**Error:** `Port 5044 is already in use`

**Find process:**
```bash
sudo lsof -i :5044
```

**Fix:** Stop conflicting process or change port in `docker-compose.yml`

### Kibana Won't Start

**Check logs:**
```bash
docker-compose logs kibana | tail -50
```

#### 1. Wrong Elasticsearch Password

**Error:** `Authentication failed`

**Fix:** Check password in `docker-compose.yml` matches Elasticsearch:
```bash
# Get current password
source .env
echo $ELASTIC_PASSWORD

# Reset if needed
docker-compose exec elasticsearch \
  bin/elasticsearch-reset-password -u elastic
```

#### 2. Kibana Timeout

**Error:** `Elasticsearch is not ready yet`

**Fix:** Wait longer, Elasticsearch can take 2-3 minutes to start:
```bash
# Watch Elasticsearch become ready
watch -n 5 'curl -s -u "elastic:${ELASTIC_PASSWORD}" \
  http://localhost:9200/_cluster/health | jq .status'
```

### All Services Keep Restarting

**Check Docker resources:**
```bash
docker stats

# Check system resources
free -h
df -h
```

**Fix:** Increase resources or reduce heap sizes in `docker-compose.yml`

## Validator Node Issues

### Filebeat Not Running

**Check status:**
```bash
ssh validator-1 'sudo systemctl status filebeat'
```

#### 1. Service Failed to Start

**Check logs:**
```bash
ssh validator-1 'sudo journalctl -u filebeat -n 100 --no-pager'
```

**Common causes:**
- Configuration syntax error
- Cannot connect to Logstash
- Permission denied on log files

**Fix configuration errors:**
```bash
# Test configuration
ssh validator-1 'sudo /usr/local/bin/filebeat test config -c /etc/filebeat/filebeat.yml'

# Test output connection
ssh validator-1 'sudo /usr/local/bin/filebeat test output -c /etc/filebeat/filebeat.yml'
```

#### 2. Filebeat Binary Not Found

**Error:** `No such file or directory: /usr/local/bin/filebeat`

**Fix:**
```bash
# Re-run deployment
./scripts/deploy-filebeat.sh
```

#### 3. Permission Denied Reading Logs

**Error:** `Failed to open /var/log/...` or journald access denied

**Fix:**
```bash
ssh validator-1 'sudo usermod -a -G systemd-journal root'
ssh validator-1 'sudo usermod -a -G adm root'
ssh validator-1 'sudo systemctl restart filebeat'
```

### Filebeat Running But No Logs

**Check registry:**
```bash
ssh validator-1 'sudo cat /var/lib/filebeat/registry/filebeat/log.json | jq'
```

**Check if files are being read:**
```bash
ssh validator-1 'sudo /usr/local/bin/filebeat export config -c /etc/filebeat/filebeat.yml'
```

**Force Filebeat to re-read logs:**
```bash
ssh validator-1 'sudo systemctl stop filebeat'
ssh validator-1 'sudo rm -rf /var/lib/filebeat/registry'
ssh validator-1 'sudo systemctl start filebeat'
```

### IPC Services Not Logging

**Check if IPC services are running:**
```bash
ssh validator-1 'sudo systemctl status ipc-node'
ssh validator-1 'sudo systemctl status ipc-relayer'
```

**Check journald logs directly:**
```bash
ssh validator-1 'sudo journalctl -u ipc-node -n 20 --no-pager'
```

**Check file logs exist:**
```bash
ssh validator-1 'ls -lh ~/.ipc-node/logs/'
```

## Network Issues

### Cannot Connect to Logstash (Port 5044)

**Test connectivity from validator:**
```bash
ssh validator-1 "telnet <CENTRAL_SERVER_IP> 5044"
# or
ssh validator-1 "nc -zv <CENTRAL_SERVER_IP> 5044"
```

**If connection refused:**

1. **Check Logstash is listening:**
```bash
docker-compose ps logstash
docker-compose logs logstash | grep 5044
```

2. **Check firewall rules on central server:**
```bash
# Ubuntu/Debian
sudo ufw status

# Check if port is open
sudo netstat -tlnp | grep 5044
```

3. **Check GCP firewall rules:**
```bash
gcloud compute firewall-rules list | grep 5044

# Create rule if missing
gcloud compute firewall-rules create allow-elk-filebeat \
  --allow tcp:5044 \
  --source-ranges <VALIDATOR_IP_1>,<VALIDATOR_IP_2>,<VALIDATOR_IP_3> \
  --description "Allow Filebeat to Logstash"
```

4. **Check if Docker is exposing the port:**
```bash
docker-compose ps
# Port 5044 should show as 0.0.0.0:5044->5044/tcp
```

### Cannot Access Kibana (Port 5601)

**Check if Kibana is running:**
```bash
docker-compose ps kibana
curl -s http://localhost:5601/api/status | jq .status.overall.state
```

**Check GCP firewall:**
```bash
gcloud compute firewall-rules create allow-kibana \
  --allow tcp:5601 \
  --source-ranges <YOUR_IP>/32 \
  --description "Allow Kibana access"
```

**Access via SSH tunnel (secure alternative):**
```bash
ssh -L 5601:localhost:5601 user@<CENTRAL_SERVER_IP>
# Then access http://localhost:5601 on your machine
```

### Slow Network / Timeouts

**Increase Filebeat timeout:**

Edit `/etc/filebeat/filebeat.yml` on validators:
```yaml
output.logstash:
  timeout: 60s  # Increase from 30s
  backoff.init: 2s
  backoff.max: 120s
```

**Enable compression:**
```yaml
output.logstash:
  compression_level: 3
```

## Performance Issues

### Elasticsearch Slow Queries

**Check slow logs:**
```bash
curl -u "elastic:${ELASTIC_PASSWORD}" \
  "http://localhost:9200/ipc-logs-*/_settings?pretty" | grep slow
```

**Enable slow query logging:**
```bash
curl -X PUT -u "elastic:${ELASTIC_PASSWORD}" \
  "http://localhost:9200/ipc-logs-*/_settings" \
  -H 'Content-Type: application/json' \
  -d '{
    "index.search.slowlog.threshold.query.warn": "10s",
    "index.search.slowlog.threshold.query.info": "5s"
  }'
```

**Check cluster stats:**
```bash
curl -u "elastic:${ELASTIC_PASSWORD}" \
  "http://localhost:9200/_cluster/stats?pretty"
```

### High CPU Usage

**Check which service:**
```bash
docker stats
```

**Reduce Logstash workers:**

Edit `logstash/config/logstash.yml`:
```yaml
pipeline.workers: 1  # Reduce from 2
```

**Reduce Elasticsearch threads:**

Edit `docker-compose.yml`:
```yaml
elasticsearch:
  environment:
    - "ES_JAVA_OPTS=-Xms2g -Xmx2g -XX:ActiveProcessorCount=2"
```

### High Memory Usage

**Check memory per container:**
```bash
docker stats --no-stream
```

**Add memory limits in `docker-compose.yml`:**
```yaml
services:
  elasticsearch:
    mem_limit: 4g
    mem_reservation: 2g

  logstash:
    mem_limit: 2g
    mem_reservation: 1g
```

### Logstash Queue Full

**Check queue stats:**
```bash
curl "http://localhost:9600/_node/stats/pipelines" | jq '.pipelines.main.queue'
```

**Increase queue size in `logstash/config/logstash.yml`:**
```yaml
queue.max_bytes: 2gb  # Increase from 1gb
```

## Data Issues

### Missing Logs / Gaps in Data

**Check Filebeat registry:**
```bash
ssh validator-1 'sudo journalctl -u filebeat | grep -i error'
```

**Check Logstash drops:**
```bash
curl "http://localhost:9600/_node/stats/pipelines" | \
  jq '.pipelines.main.plugins.filters[] | select(.name == "drop")'
```

**Check for grok parsing failures:**
```bash
curl -u "elastic:${ELASTIC_PASSWORD}" \
  -X GET "http://localhost:9200/ipc-logs-*/_search?pretty" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": {
      "term": {
        "tags": "_grokparsefailure"
      }
    }
  }'
```

### Duplicate Logs

**Cause:** Filebeat registry corruption or multiple Filebeat instances

**Fix:**
```bash
ssh validator-1 'sudo systemctl stop filebeat'
ssh validator-1 'sudo rm -rf /var/lib/filebeat/registry'
ssh validator-1 'sudo systemctl start filebeat'
```

### Incorrect Timestamps

**Check timezone settings:**
```bash
# On validators
ssh validator-1 'timedatectl'

# Ensure NTP is enabled
ssh validator-1 'sudo timedatectl set-ntp true'
```

**Fix timestamp parsing in Logstash:**

Edit `logstash/pipeline/ipc-logs.conf`, add timezone:
```ruby
date {
  match => ["timestamp", "ISO8601"]
  target => "@timestamp"
  timezone => "UTC"
}
```

### Old Indices Not Deleted

**Check ILM policy execution:**
```bash
curl -u "elastic:${ELASTIC_PASSWORD}" \
  "http://localhost:9200/ipc-logs-*/_ilm/explain?pretty"
```

**Manually trigger ILM:**
```bash
curl -X POST -u "elastic:${ELASTIC_PASSWORD}" \
  "http://localhost:9200/ipc-logs-*/_ilm/move/delete"
```

## Common Error Messages

### "Unable to parse date"

**Error in Logstash:**
```
Failed to parse date from field
```

**Fix:** Update date pattern in `logstash/pipeline/ipc-logs.conf`:
```ruby
date {
  match => [
    "timestamp",
    "ISO8601",
    "yyyy-MM-dd'T'HH:mm:ss.SSSZ",
    "yyyy-MM-dd HH:mm:ss.SSS"
  ]
}
```

### "Connection refused [Errno 111]"

**Filebeat cannot connect to Logstash**

**Check:**
1. Logstash is running: `docker-compose ps logstash`
2. Network connectivity: `telnet <SERVER_IP> 5044`
3. Firewall rules allow port 5044
4. Correct SERVER_IP in Filebeat config

### "No data views"

**Kibana shows "Create a data view"**

**Fix:**
```bash
./scripts/setup-kibana-dashboards.sh
```

Or manually create in Kibana UI:
- Management > Data Views > Create data view
- Pattern: `ipc-logs-*`
- Timestamp: `@timestamp`

### "Circuit breaker triggered"

**Elasticsearch rejecting requests**

**Fix:** Increase circuit breaker limits:
```bash
curl -X PUT -u "elastic:${ELASTIC_PASSWORD}" \
  "http://localhost:9200/_cluster/settings" \
  -H 'Content-Type: application/json' \
  -d '{
    "persistent": {
      "indices.breaker.total.limit": "80%"
    }
  }'
```

Or add more memory to Elasticsearch.

## Getting More Help

### Enable Debug Logging

**Filebeat:**
```yaml
# /etc/filebeat/filebeat.yml
logging.level: debug
logging.to_files: true
```

**Logstash:**
```yaml
# logstash/config/logstash.yml
log.level: debug
```

**Elasticsearch:**
```bash
curl -X PUT -u "elastic:${ELASTIC_PASSWORD}" \
  "http://localhost:9200/_cluster/settings" \
  -H 'Content-Type: application/json' \
  -d '{
    "transient": {
      "logger.org.elasticsearch": "DEBUG"
    }
  }'
```

### Collect Diagnostic Information

```bash
#!/bin/bash
# Save to diagnostics.sh

echo "=== Docker Compose Status ==="
docker-compose ps

echo -e "\n=== Elasticsearch Health ==="
curl -u "elastic:${ELASTIC_PASSWORD}" \
  "http://localhost:9200/_cluster/health?pretty"

echo -e "\n=== Indices ==="
curl -u "elastic:${ELASTIC_PASSWORD}" \
  "http://localhost:9200/_cat/indices/ipc-logs-*?v"

echo -e "\n=== Logstash Stats ==="
curl "http://localhost:9600/_node/stats?pretty"

echo -e "\n=== Recent Logs ==="
docker-compose logs --tail=50 elasticsearch logstash kibana

echo -e "\n=== System Resources ==="
free -h
df -h
docker stats --no-stream
```

Run and share output when seeking help.

### Contact Support

Include in your support request:
1. Output from `diagnostics.sh`
2. Relevant error messages
3. Steps to reproduce
4. When the issue started
5. Any recent changes

## Preventive Maintenance

### Regular Health Checks

Create a cron job:
```bash
# /etc/cron.daily/elk-health-check
#!/bin/bash
cd /path/to/elk-logging
./scripts/check-log-flow.sh | mail -s "ELK Health Check" admin@example.com
```

### Monitor Disk Space

```bash
# Alert when disk >80% full
df -h / | awk 'NR==2 {if ($5+0 > 80) print "WARNING: Disk space low " $5}'
```

### Regular Backups

Schedule weekly Elasticsearch snapshots (see README.md Maintenance section).

### Update Schedule

- **Monthly:** Update Filebeat on validators
- **Quarterly:** Update ELK stack (test in staging first)
- **Annually:** Review and optimize ILM policies

