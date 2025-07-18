#!/usr/bin/env python3
"""
IPC Throughput Test Runner

This script orchestrates throughput tests by:
1. Setting up test networks using materializer
2. Running throughput benchmarks
3. Collecting and analyzing results
4. Cleaning up test environments
"""

import argparse
import json
import logging
import os
import subprocess
import sys
import time
import yaml
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s',
    handlers=[
        logging.FileHandler('throughput_test.log'),
        logging.StreamHandler()
    ]
)

logger = logging.getLogger(__name__)


class ThroughputTestRunner:
    """Orchestrates throughput testing with materializer integration"""

    def __init__(self, config_path: str):
        self.config_path = Path(config_path)
        self.base_dir = Path(__file__).parent.parent
        self.materializer_dir = self.base_dir.parent.parent / "materializer"
        self.results_dir = self.base_dir / "results"
        self.results_dir.mkdir(exist_ok=True)

        # Load test configuration
        with open(self.config_path) as f:
            self.config = yaml.safe_load(f)

        self.test_name = self.config.get('name', 'throughput_test')
        self.timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')

    def setup_materializer_config(self, validators: int) -> Path:
        """Generate materializer configuration for the test"""
        materializer_config = {
            'materializer': {
                'mode': 'LocalNet',
                'nodes': validators,
                'validators': validators,
                'consensus': {
                    'timeout_commit': '1s',
                    'timeout_propose': '3s',
                    'create_empty_blocks': True,
                    'create_empty_blocks_interval': '1s'
                },
                'fendermint': {
                    'gas_limit': 10000000,
                    'gas_price': 1000000000,
                    'block_time': '1s'
                }
            }
        }

        config_path = self.results_dir / f"materializer_config_{validators}v_{self.timestamp}.toml"

        # Convert to TOML format and save
        with open(config_path, 'w') as f:
            # Simple TOML generation for our specific config
            f.write(f"""[materializer]
mode = "LocalNet"
nodes = {validators}
validators = {validators}

[materializer.consensus]
timeout_commit = "1s"
timeout_propose = "3s"
create_empty_blocks = true
create_empty_blocks_interval = "1s"

[materializer.fendermint]
gas_limit = 10000000
gas_price = 1000000000
block_time = "1s"
""")

        return config_path

    def start_test_network(self, validators: int) -> bool:
        """Start a test network using materializer"""
        logger.info(f"Starting test network with {validators} validators")

        # Generate materializer configuration
        config_path = self.setup_materializer_config(validators)

        try:
            # Change to materializer directory
            os.chdir(self.materializer_dir)

            # Start the network
            cmd = [
                "cargo", "run", "--bin", "materializer",
                "--", "setup",
                "--config", str(config_path),
                "--output-dir", str(self.results_dir / f"testnet_{validators}v_{self.timestamp}")
            ]

            result = subprocess.run(cmd, capture_output=True, text=True, timeout=300)

            if result.returncode != 0:
                logger.error(f"Failed to start test network: {result.stderr}")
                return False

            logger.info("Test network started successfully")

            # Wait for network to be ready
            time.sleep(10)

            # Verify network is responding
            return self.verify_network_ready()

        except subprocess.TimeoutExpired:
            logger.error("Timeout starting test network")
            return False
        except Exception as e:
            logger.error(f"Error starting test network: {e}")
            return False

    def verify_network_ready(self) -> bool:
        """Verify that the test network is ready to accept transactions"""
        logger.info("Verifying network readiness")

        endpoints = self.config.get('network', {}).get('endpoints', ['http://localhost:8545'])

        for endpoint in endpoints:
            try:
                # Simple health check using curl
                result = subprocess.run([
                    'curl', '-s', '-X', 'POST',
                    '-H', 'Content-Type: application/json',
                    '-d', '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}',
                    endpoint
                ], capture_output=True, text=True, timeout=5)

                if result.returncode != 0:
                    logger.warning(f"Endpoint {endpoint} not ready")
                    return False

                # Check if response is valid JSON
                try:
                    response = json.loads(result.stdout)
                    if 'result' not in response:
                        logger.warning(f"Invalid response from {endpoint}")
                        return False
                except json.JSONDecodeError:
                    logger.warning(f"Invalid JSON response from {endpoint}")
                    return False

            except subprocess.TimeoutExpired:
                logger.warning(f"Timeout checking endpoint {endpoint}")
                return False

        logger.info("Network is ready")
        return True

    def run_throughput_benchmark(self) -> Optional[Dict]:
        """Run the throughput benchmark"""
        logger.info("Starting throughput benchmark")

        # Prepare benchmark command
        benchmark_bin = self.base_dir / "target" / "release" / "throughput_bench"

        # Build if not exists
        if not benchmark_bin.exists():
            logger.info("Building throughput benchmark")
            result = subprocess.run([
                "cargo", "build", "--release", "--bin", "throughput_bench"
            ], cwd=self.base_dir, capture_output=True, text=True)

            if result.returncode != 0:
                logger.error(f"Failed to build benchmark: {result.stderr}")
                return None

        # Run benchmark
        result_file = self.results_dir / f"{self.test_name}_{self.timestamp}_results.json"

        cmd = [
            str(benchmark_bin), "run",
            "--config", str(self.config_path),
            "--output", str(result_file),
            "--verbose"
        ]

        try:
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=1800)  # 30 min timeout

            if result.returncode != 0:
                logger.error(f"Benchmark failed: {result.stderr}")
                return None

            logger.info("Benchmark completed successfully")
            logger.info(f"Results saved to: {result_file}")

            # Load and return results
            with open(result_file) as f:
                return json.load(f)

        except subprocess.TimeoutExpired:
            logger.error("Benchmark timeout")
            return None
        except Exception as e:
            logger.error(f"Error running benchmark: {e}")
            return None

    def cleanup_test_network(self):
        """Clean up test network resources"""
        logger.info("Cleaning up test network")

        try:
            # Kill any remaining processes
            subprocess.run(["pkill", "-f", "fendermint"], capture_output=True)
            subprocess.run(["pkill", "-f", "cometbft"], capture_output=True)

            # Clean up temporary files
            testnet_dirs = list(self.results_dir.glob(f"testnet_*{self.timestamp}"))
            for testnet_dir in testnet_dirs:
                if testnet_dir.is_dir():
                    import shutil
                    shutil.rmtree(testnet_dir)

            logger.info("Cleanup completed")

        except Exception as e:
            logger.warning(f"Error during cleanup: {e}")

    def generate_report(self, results: Dict) -> str:
        """Generate a summary report"""
        report = []
        report.append(f"# Throughput Test Report - {self.test_name}")
        report.append(f"**Timestamp:** {self.timestamp}")
        report.append(f"**Duration:** {results.get('duration', 'N/A')}")
        report.append("")

        # Performance metrics
        if 'performance' in results:
            perf = results['performance']
            report.append("## Performance Metrics")
            report.append(f"- **Average TPS:** {perf.get('avg_tps', 'N/A'):.2f}")
            report.append(f"- **Peak TPS:** {perf.get('peak_tps', 'N/A'):.2f}")
            report.append(f"- **Total Transactions:** {perf.get('total_transactions', 'N/A')}")
            report.append(f"- **Success Rate:** {perf.get('success_rate', 'N/A'):.2%}")
            report.append("")

        # Latency metrics
        if 'latency' in results:
            latency = results['latency']
            report.append("## Latency Metrics")
            report.append(f"- **Average Latency:** {latency.get('avg_ms', 'N/A'):.2f}ms")
            report.append(f"- **P95 Latency:** {latency.get('p95_ms', 'N/A'):.2f}ms")
            report.append(f"- **P99 Latency:** {latency.get('p99_ms', 'N/A'):.2f}ms")
            report.append("")

        # Resource usage
        if 'resources' in results:
            resources = results['resources']
            report.append("## Resource Usage")
            report.append(f"- **Peak CPU:** {resources.get('peak_cpu_percent', 'N/A'):.1f}%")
            report.append(f"- **Peak Memory:** {resources.get('peak_memory_mb', 'N/A'):.1f}MB")
            report.append(f"- **Network I/O:** {resources.get('network_io_mb', 'N/A'):.1f}MB")
            report.append("")

        report_content = "\n".join(report)

        # Save report
        report_file = self.results_dir / f"{self.test_name}_{self.timestamp}_report.md"
        with open(report_file, 'w') as f:
            f.write(report_content)

        logger.info(f"Report saved to: {report_file}")
        return report_content

    def run_test(self) -> bool:
        """Run the complete throughput test"""
        logger.info(f"Starting throughput test: {self.test_name}")

        try:
            # Determine validator count
            network_config = self.config.get('network', {})
            validators = network_config.get('validators', 4)

            if network_config.get('type') == 'multi_config':
                # Handle multi-configuration tests
                all_results = {}

                for test_config in network_config.get('test_configs', []):
                    config_name = test_config['name']
                    config_validators = test_config['validators']

                    logger.info(f"Running test configuration: {config_name}")

                    # Start network for this configuration
                    if not self.start_test_network(config_validators):
                        logger.error(f"Failed to start network for {config_name}")
                        continue

                    # Update config endpoints for this test
                    original_endpoints = self.config['network']['endpoints']
                    self.config['network']['endpoints'] = test_config['endpoints']

                    # Run benchmark
                    results = self.run_throughput_benchmark()
                    if results:
                        all_results[config_name] = results

                    # Restore original endpoints
                    self.config['network']['endpoints'] = original_endpoints

                    # Cleanup
                    self.cleanup_test_network()

                    # Wait between configurations
                    inter_delay = self.config.get('test', {}).get('inter_test_delay', '2m')
                    logger.info(f"Waiting {inter_delay} before next configuration")
                    time.sleep(self.parse_duration(inter_delay))

                # Generate combined report
                combined_results = {'multi_config_results': all_results}
                self.generate_report(combined_results)

                return len(all_results) > 0

            else:
                # Single configuration test
                if not self.start_test_network(validators):
                    return False

                results = self.run_throughput_benchmark()
                if results:
                    self.generate_report(results)
                    return True

                return False

        except Exception as e:
            logger.error(f"Test failed: {e}")
            return False

        finally:
            self.cleanup_test_network()

    def parse_duration(self, duration_str: str) -> int:
        """Parse duration string to seconds"""
        if duration_str.endswith('s'):
            return int(duration_str[:-1])
        elif duration_str.endswith('m'):
            return int(duration_str[:-1]) * 60
        elif duration_str.endswith('h'):
            return int(duration_str[:-1]) * 3600
        else:
            return int(duration_str)


def main():
    parser = argparse.ArgumentParser(description='Run IPC throughput tests')
    parser.add_argument('config', help='Path to test configuration file')
    parser.add_argument('--verbose', '-v', action='store_true', help='Enable verbose logging')

    args = parser.parse_args()

    if args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)

    if not Path(args.config).exists():
        logger.error(f"Configuration file not found: {args.config}")
        sys.exit(1)

    runner = ThroughputTestRunner(args.config)

    success = runner.run_test()

    if success:
        logger.info("Test completed successfully")
        sys.exit(0)
    else:
        logger.error("Test failed")
        sys.exit(1)


if __name__ == '__main__':
    main()