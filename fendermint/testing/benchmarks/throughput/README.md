# Throughput Benchmarking

This module provides comprehensive throughput testing for IPC subnets, measuring transactions per second (TPS) under various conditions.

## Features

- **Variable Validator Count**: Test with 1, 4, 7, 10, and 20 validators
- **Transaction Types**: Simple transfers, ERC-20 transfers, contract calls, deployments
- **Load Patterns**: Sustained load, burst testing, periodic patterns
- **Real-time Monitoring**: Live TPS and latency metrics
- **Automated Reports**: Performance analysis and regression detection

## Usage

### Basic Throughput Test

```bash
# Single validator, simple transfers
make throughput-test VALIDATORS=1 TX_TYPE=transfer DURATION=300s

# Multi-validator setup
make throughput-test VALIDATORS=4 TX_TYPE=transfer DURATION=600s
```

### Contract Interaction Tests

```bash
# ERC-20 transfers
make throughput-test VALIDATORS=4 TX_TYPE=erc20 DURATION=300s

# Contract deployments
make throughput-test VALIDATORS=1 TX_TYPE=deploy DURATION=120s
```

### Stress Testing

```bash
# Find maximum sustainable TPS
make stress-test VALIDATORS=4 RAMP_UP=true DURATION=1800s
```

## Test Configuration

Configuration is handled through YAML files in the `configs/` directory:

- `single-validator.yaml`: Single validator performance baseline
- `multi-validator.yaml`: Multi-validator consensus testing
- `contract-heavy.yaml`: Contract interaction focused tests
- `stress-test.yaml`: Stress testing configuration

## Metrics Collected

- **Throughput**: TPS over time, peak TPS, sustained TPS
- **Latency**: Transaction confirmation latency percentiles
- **Resource Usage**: CPU, memory, disk I/O per validator
- **Network**: Bandwidth usage, message propagation times
- **Errors**: Transaction failure rates, consensus issues

## Output

Results are stored in `results/` directory with:
- Raw metrics in JSON format
- Processed statistics and percentiles
- Performance graphs and charts
- Automated performance reports