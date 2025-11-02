# IPC Subnet Monitoring Setup

This guide shows how to set up monitoring for IPC subnet parent finality.

## Quick Start

The monitoring script checks if your subnet's parent finality is falling behind:

```bash
# Basic usage
./monitor-parent-finality-simple.sh

# With custom thresholds
./monitor-parent-finality-simple.sh 34.73.187.192 100 1000 text

# Get just the lag number (for Zabbix)
./monitor-parent-finality-simple.sh 34.73.187.192 100 1000 zabbix
```

**Parameters:**
1. Validator IP (default: 34.73.187.192)
2. Warning threshold in epochs (default: 100)
3. Critical threshold in epochs (default: 1000)
4. Output format: text|json|zabbix|prometheus

**Exit Codes:**
- `0` = OK (finality is healthy)
- `1` = WARNING (lag exceeds warning threshold)
- `2` = CRITICAL (lag exceeds critical threshold)
- `3` = UNKNOWN (unable to fetch metrics)

## Zabbix Integration

### Method 1: User Parameters (Remote Execution)

1. **Install Zabbix Agent on monitoring server** (not on validator):

```bash
# On your monitoring/management server
sudo apt install zabbix-agent2
```

2. **Configure user parameters**:

Edit `/etc/zabbix/zabbix_agent2.conf`:

```ini
# IPC Subnet Monitoring
UserParameter=ipc.finality.lag,/path/to/monitor-parent-finality-simple.sh 34.73.187.192 100 1000 zabbix
UserParameter=ipc.finality.status,/path/to/monitor-parent-finality-simple.sh 34.73.187.192 100 1000 text; echo $?
```

3. **Restart Zabbix agent**:

```bash
sudo systemctl restart zabbix-agent2
```

4. **Create Zabbix items**:

In Zabbix frontend:
- Host: Your monitoring server
- Item name: `IPC Finality Lag`
- Key: `ipc.finality.lag`
- Type: Zabbix agent
- Type of information: Numeric (unsigned)
- Units: epochs

### Method 2: External Check (Recommended)

1. **Copy script to Zabbix external scripts directory**:

```bash
sudo cp monitor-parent-finality-simple.sh /usr/lib/zabbix/externalscripts/
sudo chmod +x /usr/lib/zabbix/externalscripts/monitor-parent-finality-simple.sh
sudo chown zabbix:zabbix /usr/lib/zabbix/externalscripts/monitor-parent-finality-simple.sh
```

2. **Create external check item in Zabbix**:

- Key: `monitor-parent-finality-simple.sh[34.73.187.192,100,1000,zabbix]`
- Type: External check
- Type of information: Numeric (unsigned)
- Update interval: 5m

### Method 3: SSH-based Monitoring (Most Reliable)

1. **Set up SSH key for Zabbix**:

```bash
# On Zabbix server, as zabbix user
sudo -u zabbix ssh-keygen -t ed25519 -f /var/lib/zabbix/.ssh/id_ed25519 -N ""

# Copy public key to validator (as your user)
ssh-copy-id -i /var/lib/zabbix/.ssh/id_ed25519.pub your_user@validator_ip

# Test
sudo -u zabbix ssh -i /var/lib/zabbix/.ssh/id_ed25519 your_user@validator_ip "echo success"
```

2. **Configure SSH items in Zabbix**:

Create items using SSH agent type with the monitoring script.

## Zabbix Template

Here's a complete Zabbix template configuration:

### Items

**1. IPC Finality Lag**
- Name: `IPC Finality Lag`
- Type: External check
- Key: `monitor-parent-finality-simple.sh[{$IPC_VALIDATOR_IP},100,1000,zabbix]`
- Type of information: Numeric (unsigned)
- Units: epochs
- Update interval: 5m

**2. IPC Finality Status**
- Name: `IPC Finality Status`
- Type: External check
- Key: `monitor-parent-finality-simple.sh[{$IPC_VALIDATOR_IP},100,1000,text]`
- Type of information: Text
- Update interval: 5m

### Triggers

**1. Warning: High Parent Finality Lag**
```
{HOSTNAME:monitor-parent-finality-simple.sh[{$IPC_VALIDATOR_IP},100,1000,zabbix].last()}>100
```
- Severity: Warning
- Description: IPC subnet parent finality lag is high ({ITEM.LASTVALUE} epochs)

**2. Critical: Parent Finality Stuck**
```
{HOSTNAME:monitor-parent-finality-simple.sh[{$IPC_VALIDATOR_IP},100,1000,zabbix].last()}>1000
```
- Severity: High
- Description: IPC subnet parent finality is stuck! Lag: {ITEM.LASTVALUE} epochs. Cross-chain messages won't process.

**3. Critical: Monitoring Script Failed**
```
{HOSTNAME:monitor-parent-finality-simple.sh[{$IPC_VALIDATOR_IP},100,1000,zabbix].nodata(10m)}=1
```
- Severity: High
- Description: IPC finality monitoring script is not returning data

### Macros

- `{$IPC_VALIDATOR_IP}` = `34.73.187.192`
- `{$IPC_WARNING_THRESHOLD}` = `100`
- `{$IPC_CRITICAL_THRESHOLD}` = `1000`

## Prometheus Integration

For Prometheus/Grafana monitoring:

```bash
# Run script in prometheus format
./monitor-parent-finality-simple.sh 34.73.187.192 100 1000 prometheus
```

Output:
```
ipc_subnet_height 813593
ipc_subnet_finality 3135525
ipc_parent_height 3156148
ipc_finality_lag 20623
ipc_finality_status 2
```

### Prometheus Exporter Setup

Create a simple text file exporter:

```bash
# Add to crontab
*/5 * * * * /path/to/monitor-parent-finality-simple.sh 34.73.187.192 100 1000 prometheus > /var/lib/node_exporter/textfile_collector/ipc_finality.prom
```

Then configure node_exporter to read from `/var/lib/node_exporter/textfile_collector/`.

## Grafana Dashboard

Example PromQL queries:

```promql
# Finality lag
ipc_finality_lag

# Rate of change (should be close to 1 when healthy)
rate(ipc_subnet_finality[5m])

# Alert when lag > 100 epochs
ipc_finality_lag > 100
```

## Testing

Test all output formats:

```bash
# Text output
./monitor-parent-finality-simple.sh 34.73.187.192 100 1000 text

# JSON output
./monitor-parent-finality-simple.sh 34.73.187.192 100 1000 json

# Zabbix output (just the lag number)
./monitor-parent-finality-simple.sh 34.73.187.192 100 1000 zabbix

# Prometheus format
./monitor-parent-finality-simple.sh 34.73.187.192 100 1000 prometheus
```

Check exit codes:
```bash
./monitor-parent-finality-simple.sh 34.73.187.192 100 1000 text
echo "Exit code: $?"
```

## Troubleshooting

### Script returns UNKNOWN

- Check SSH connectivity to validator
- Verify validator is running: `ssh validator "systemctl status ipc-node"`
- Check if you need to accept SSH host key first

### Values seem wrong

- Verify validator IP is correct
- Check parent RPC is accessible: `curl https://api.calibration.node.glif.io/rpc/v1`
- Review validator logs for errors

### High lag but subnet is running

This is the current state! Parent finality is stuck due to RPC lookback limits.
Solution: Use a Lotus full node or archive node as parent RPC.

## Best Practices

1. **Set appropriate thresholds**:
   - Warning: 100 epochs (~50 minutes)
   - Critical: 1000 epochs (~8 hours)
   - Adjust based on your needs

2. **Monitor regularly**:
   - Check every 5 minutes
   - Alert on sustained lag, not single spikes

3. **Set up alerts**:
   - Email/SMS for CRITICAL status
   - Slack/Discord for WARNING status
   - Weekly reports on finality health

4. **Create runbooks**:
   - Document what to do when finality lags
   - Include steps to restart validators
   - Note when to switch parent RPC

## Example Alerting Logic

```bash
#!/bin/bash
# Add to cron: */5 * * * * /path/to/alert-on-finality.sh

LAG=$(./monitor-parent-finality-simple.sh 34.73.187.192 100 1000 zabbix)
EXIT_CODE=$?

if [ $EXIT_CODE -eq 2 ]; then
    # CRITICAL - send urgent alert
    echo "CRITICAL: IPC finality lag is ${LAG} epochs!" | \
        mail -s "IPC CRITICAL ALERT" admin@example.com
elif [ $EXIT_CODE -eq 1 ]; then
    # WARNING - log and notify
    echo "$(date): WARNING - Finality lag: ${LAG} epochs" >> /var/log/ipc-finality.log
fi
```

## Support

For issues or questions:
- Check validator logs: `journalctl -u ipc-node -f`
- Review parent finality status: `./ipc-manager info`
- Monitor dashboard: `./ipc-manager dashboard`

