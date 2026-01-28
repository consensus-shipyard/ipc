# ELK Stack Log Aggregation - Project Summary

Complete ELK (Elasticsearch, Logstash, Kibana) stack for IPC validator log aggregation.

## 📦 What Was Created

### Directory Structure

```
infra/elk-logging/
├── docker-compose.yml                    # Main ELK stack orchestration
├── .env.example                          # Environment template (blocked by gitignore)
├── README.md                             # Complete documentation
├── QUICK-START.md                        # 30-minute setup guide
├── TROUBLESHOOTING.md                    # Comprehensive troubleshooting
├── PROJECT-SUMMARY.md                    # This file
│
├── elasticsearch/
│   ├── config/
│   │   └── elasticsearch.yml             # Elasticsearch configuration
│   ├── index-template.json               # Index mapping template
│   └── ilm-policy.json                   # Lifecycle management (90-day retention)
│
├── logstash/
│   ├── config/
│   │   └── logstash.yml                  # Logstash configuration
│   └── pipeline/
│       └── ipc-logs.conf                 # IPC-specific log parsing pipeline
│
├── kibana/
│   ├── config/
│   │   └── kibana.yml                    # Kibana configuration
│   └── dashboards/
│       ├── ipc-validator-overview.ndjson # Pre-built dashboard
│       └── (create more in Kibana UI)
│
├── grafana/
│   └── provisioning/
│       ├── datasources/
│       │   └── elasticsearch.yml         # Auto-configure Elasticsearch datasource
│       └── dashboards/
│           └── default.yml               # Dashboard provisioning
│
├── filebeat/
│   ├── filebeat.yml.template             # Filebeat config template (for validators)
│   └── filebeat.service.template         # Systemd service template
│
└── scripts/
    ├── setup-central-server.sh           # 🚀 Setup ELK stack on central server
    ├── deploy-filebeat.sh                # 🚀 Deploy Filebeat to all validators
    ├── check-log-flow.sh                 # ✅ Verify logs are flowing
    ├── setup-kibana-dashboards.sh        # 📊 Setup Kibana dashboards
    └── elk-manager.sh                    # 🛠️ Management utility
```

## 🎯 Key Features

### 1. Complete ELK Stack
- **Elasticsearch 8.11.0**: Log storage and search engine
- **Logstash 8.11.0**: Log processing with IPC-specific parsing
- **Kibana 8.11.0**: Web UI for visualization and analysis
- **Grafana 10.2.0**: Alternative visualization (bonus)

### 2. IPC-Specific Log Parsing
Automatically extracts and indexes:
- ✅ Log levels (ERROR, WARN, INFO, DEBUG)
- ✅ CometBFT consensus data (block heights, rounds, votes)
- ✅ Checkpoint relayer events
- ✅ Ethereum/FEVM transactions
- ✅ Validator metadata (name, IP, role)
- ✅ Subnet information

### 3. Multiple Log Sources
Collects from each validator:
- Systemd journal (`ipc-node.service`, `ipc-relayer.service`)
- File logs (`~/.ipc-node/logs/*.log`)
- CometBFT logs

### 4. Production-Ready Features
- ✅ 90-day log retention with automatic cleanup
- ✅ Index lifecycle management (hot/warm/cold/delete)
- ✅ Automatic log rotation and compression
- ✅ Secure authentication (auto-generated passwords)
- ✅ Health monitoring and diagnostics
- ✅ GCP-optimized configuration

### 5. Easy Management
- One-command central server setup
- One-command Filebeat deployment to all validators
- Management CLI for common operations
- Comprehensive troubleshooting guides

## 🚀 Quick Start Commands

### Initial Setup (One Time)

```bash
# 1. Setup central server (run on central server)
cd /path/to/ipc/infra/elk-logging
./scripts/setup-central-server.sh
# Save the displayed credentials!

# 2. Configure GCP firewall
gcloud compute firewall-rules create allow-elk-logging \
  --allow tcp:5044,tcp:5601,tcp:3000 \
  --source-ranges <VALIDATOR_IPS>,<YOUR_IP>

# 3. Deploy to validators (run from your machine)
export IPC_CONFIG="$HOME/github/ipc/scripts/ipc-subnet-manager/ipc-subnet-config.yml"
./scripts/deploy-filebeat.sh

# 4. Wait 2-3 minutes, then verify
./scripts/check-log-flow.sh

# 5. Access Kibana
open http://<SERVER_IP>:5601
# Login with credentials from step 1
```

### Daily Operations

```bash
# Check status
./scripts/elk-manager.sh status

# View logs
./scripts/elk-manager.sh logs

# Health check
./scripts/elk-manager.sh health

# Search logs
./scripts/elk-manager.sh search "validator:validator-1 AND ERROR"

# Check Filebeat on validators
./scripts/elk-manager.sh filebeat-status

# List indices
./scripts/elk-manager.sh indices
```

## 📊 Access URLs

Once deployed, you can access:

- **Kibana**: `http://<SERVER_IP>:5601`
  - Username: `elastic`
  - Password: (from setup script)
  - Use for: Log viewing, searching, dashboards

- **Grafana**: `http://<SERVER_IP>:3000`
  - Username: `admin`
  - Password: (from setup script)
  - Use for: Alternative visualization, metrics dashboards

- **Elasticsearch API**: `http://<SERVER_IP>:9200`
  - Username: `elastic`
  - Password: (from setup script)
  - Use for: Direct API access, automation

## 🔧 Configuration

### Central Server Requirements

| Resource | Minimum | Recommended |
|----------|---------|-------------|
| CPU | 2 vCPUs | 4 vCPUs |
| RAM | 4GB | 8GB |
| Disk | 50GB SSD | 100GB+ SSD |
| OS | Ubuntu 22.04 | Ubuntu 22.04 LTS |

### Ports Required

| Port | Service | Access From |
|------|---------|-------------|
| 5044 | Logstash (Beats) | Validators only |
| 5601 | Kibana | Your IP |
| 3000 | Grafana | Your IP |
| 9200 | Elasticsearch API | Localhost (optional: Your IP) |

### Resource Allocation (Adjustable)

Edit `docker-compose.yml`:

```yaml
# Elasticsearch heap size
ES_JAVA_OPTS=-Xms2g -Xmx2g    # 2GB default, increase for more data

# Logstash heap size
LS_JAVA_OPTS=-Xms1g -Xmx1g    # 1GB default
```

### Log Retention (Adjustable)

Edit `elasticsearch/ilm-policy.json`:

```json
"delete": { "min_age": "90d" }  // Change from 90 days to desired
```

## 📈 Usage Examples

### Kibana Query Language (KQL) Examples

```bash
# All errors
log_level:"ERROR"

# Specific validator
validator:"validator-1"

# CometBFT consensus logs
tags:"cometbft_consensus" AND block_height > 1000

# Checkpoint relayer
service:"ipc-relayer" AND message:*checkpoint*

# Recent errors (last hour)
log_level:"ERROR" AND @timestamp >= now-1h

# Combine filters
validator:"validator-2" AND service:"ipc-node" AND log_level:("ERROR" OR "WARN")

# Block production rate
tags:"cometbft_consensus" AND message:*Committed*

# Failed transactions
message:*failed* OR message:*error*
```

### CLI Search Examples

```bash
# Quick search
./scripts/elk-manager.sh search "validator:validator-1 AND ERROR"

# Using curl directly
curl -u "elastic:${ELASTIC_PASSWORD}" \
  -X GET "http://localhost:9200/ipc-logs-*/_search?pretty" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": {
      "query_string": {
        "query": "validator:validator-1 AND log_level:ERROR"
      }
    },
    "size": 10,
    "sort": [{"@timestamp": "desc"}]
  }'
```

## 🔍 Monitoring & Alerts

### Built-in Monitoring

The stack includes:
- Elasticsearch cluster health monitoring
- Logstash pipeline statistics
- Filebeat registry tracking
- Service health checks

Access monitoring:
```bash
./scripts/elk-manager.sh health
```

### Setting Up Alerts (Optional)

Kibana supports alerting for:
- Error rate thresholds
- Service downtime
- Log volume anomalies
- Custom queries

Configure in Kibana: Management > Stack Management > Alerts and Insights

## 🛠️ Maintenance

### Regular Tasks

**Daily:**
- Monitor disk space: `df -h`
- Check service health: `./scripts/elk-manager.sh health`

**Weekly:**
- Review log volume: `./scripts/elk-manager.sh indices`
- Check for errors in services: `docker-compose logs | grep ERROR`

**Monthly:**
- Update Filebeat on validators
- Review and adjust retention policies
- Backup Elasticsearch data: `./scripts/elk-manager.sh backup`

**Quarterly:**
- Update ELK stack: `./scripts/elk-manager.sh update`
- Review and optimize dashboards
- Audit security settings

### Backup Strategy

```bash
# Create snapshot
./scripts/elk-manager.sh backup

# Or manually
curl -X PUT "http://localhost:9200/_snapshot/backup/snapshot_$(date +%Y%m%d)" \
  -u "elastic:${ELASTIC_PASSWORD}"
```

## 🔐 Security Considerations

### Production Checklist

- ✅ Strong passwords (auto-generated by setup script)
- ✅ Elasticsearch security enabled
- ✅ Kibana encryption key configured
- ⚠️ TLS/SSL not configured (consider for production)
- ⚠️ Firewall rules (restrict to specific IPs)
- ⚠️ Regular security updates needed

### Recommended Enhancements

1. **Enable TLS for Filebeat → Logstash**
2. **Use VPC/VPN for validator → central server communication**
3. **Implement log forwarding authentication**
4. **Set up regular security audits**
5. **Enable Elasticsearch audit logging**

## 📚 Documentation Files

| File | Purpose |
|------|---------|
| `README.md` | Complete guide (architecture, setup, usage, troubleshooting) |
| `QUICK-START.md` | 30-minute setup guide for quick deployment |
| `TROUBLESHOOTING.md` | Comprehensive troubleshooting (errors, fixes, diagnostics) |
| `PROJECT-SUMMARY.md` | This file - overview and quick reference |

## 🎓 Learning Resources

### Kibana
- Create visualizations: Analytics > Visualize Library
- Build dashboards: Analytics > Dashboard
- KQL syntax: [Kibana Query Language](https://www.elastic.co/guide/en/kibana/current/kuery-query.html)

### Elasticsearch
- Query DSL: [Elasticsearch Query DSL](https://www.elastic.co/guide/en/elasticsearch/reference/current/query-dsl.html)
- Aggregations: For analytics and statistics
- Index management: `/_cat/indices`, `/_stats`

### Logstash
- Grok patterns: For custom log parsing
- Filter plugins: For log enrichment
- Pipeline debugging: Enable debug logs

## 🆘 Getting Help

1. **Check documentation**:
   - `README.md` for detailed information
   - `TROUBLESHOOTING.md` for specific issues

2. **Run diagnostics**:
   ```bash
   ./scripts/elk-manager.sh health
   ./scripts/check-log-flow.sh
   ```

3. **View service logs**:
   ```bash
   docker-compose logs <service>
   ```

4. **Common issues**:
   - No logs? Check Filebeat status on validators
   - Services not starting? Check `docker-compose logs`
   - Can't connect? Check firewall rules
   - Slow performance? Check disk space and resources

## 🎉 Success Metrics

You'll know it's working when:

- ✅ All Docker services show as "healthy"
- ✅ `check-log-flow.sh` shows logs from all validators
- ✅ Kibana displays real-time logs
- ✅ Search queries return expected results
- ✅ Dashboards show validator activity

## 📊 Expected Results

After successful deployment:

- **Log volume**: ~1000-10,000 logs/day per validator (depends on activity)
- **Disk usage**: ~100-500MB/day for 3 validators
- **Search latency**: <100ms for recent logs
- **CPU usage**: 10-30% on 2 vCPU server
- **Memory usage**: 3-4GB total

## 🔄 Next Steps

After deployment:

1. **Explore Kibana**: Create custom visualizations and dashboards
2. **Set up alerts**: Configure notifications for critical events
3. **Optimize queries**: Save frequently used searches
4. **Integrate metrics**: Add Prometheus for system metrics
5. **Document workflows**: Create runbooks for your team

## 💡 Tips & Best Practices

1. **Use KQL in Kibana** - Faster and more intuitive than Lucene
2. **Create index patterns early** - Easier to query across time ranges
3. **Tag important searches** - Save them for quick access
4. **Set up dashboards per use case** - One for operations, one for debugging, etc.
5. **Monitor disk space** - Set up alerts before it fills up
6. **Regular backups** - Schedule weekly Elasticsearch snapshots
7. **Test recovery** - Ensure you can restore from backups

## 🏆 Advanced Features (Future)

Consider adding:
- **Alerting**: Slack/Discord/Email notifications
- **Metrics**: Prometheus + Node Exporter for system metrics
- **Tracing**: Jaeger or Zipkin for distributed tracing
- **APM**: Elastic APM for application performance
- **Machine Learning**: Anomaly detection in Kibana
- **Geographic visualization**: Map validators by location

---

## Summary

You now have a production-ready ELK stack that:
- ✅ Automatically collects logs from 3 validators
- ✅ Parses IPC-specific log formats
- ✅ Provides searchable, indexed logs
- ✅ Includes visualization tools (Kibana + Grafana)
- ✅ Retains 90 days of logs with automatic cleanup
- ✅ Is fully documented and maintainable

**Total setup time**: ~30-45 minutes
**Monthly cost**: ~$35 for GCP instance (or $0 if using existing server)

🎉 **Your IPC validator logging infrastructure is complete and ready to use!**

