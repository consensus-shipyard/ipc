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
import shutil
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
        self.testnet_id = None  # Will be set during network startup

    def setup_materializer_config(self, validators: int) -> Path:
        """Generate materializer manifest for the test"""

        # Generate accounts and validators
        accounts = {}
        validator_accounts = {}
        balances = {}
        nodes = {}

        # Create accounts for validators
        for i in range(validators):
            account_name = f"validator{i}"
            accounts[account_name] = {}
            validator_accounts[account_name] = "100"  # Minimum collateral in atto
            balances[account_name] = "100000000000000000000"  # 100 FIL in atto

            # Create nodes for validators
            node_name = f"node-{i}"
            nodes[node_name] = {
                "mode": {
                    "type": "Validator",
                    "validator": account_name
                },
                "ethapi": i == 0,  # First node has ethapi
                "seed_nodes": [] if i == 0 else ["node-0"]  # Others seed from first node
            }

        # Add some extra accounts for testing
        accounts["alice"] = {}
        accounts["bob"] = {}
        balances["alice"] = "200000000000000000000"  # 200 FIL
        balances["bob"] = "300000000000000000000"    # 300 FIL

        # Create the manifest
        manifest = {
            "accounts": accounts,
            "rootnet": {
                "type": "New",
                "validators": validator_accounts,
                "balances": balances,
                "ipc_contracts_owner": "validator0",
                "env": {
                    "CMT_CONSENSUS_TIMEOUT_COMMIT": "1s",
                    "CMT_CONSENSUS_TIMEOUT_PROPOSE": "2s",
                    "FM_LOG_LEVEL": "info,fendermint=debug"
                },
                "nodes": nodes
            }
        }

        config_path = self.results_dir / f"materializer_config_{validators}v_{self.timestamp}.yaml"

        # Write YAML manifest
        with open(config_path, 'w') as f:
            yaml.dump(manifest, f, default_flow_style=False, sort_keys=False)

        return config_path

    def start_test_network(self, validators: int) -> bool:
        """Start a test network using materializer"""
        logger.info(f"Starting test network with {validators} validators")

        # Generate materializer configuration
        config_path = self.setup_materializer_config(validators)

        try:
            # Use the fendermint binary with materializer subcommand
            fendermint_bin = self.base_dir.parent.parent.parent.parent / "target" / "release" / "fendermint"
            data_dir = self.results_dir / f"testnet_{validators}v_{self.timestamp}"

            # Create data directory
            data_dir.mkdir(parents=True, exist_ok=True)

            # Set environment variables to avoid tracing conflicts
            env = os.environ.copy()
            env["RUST_LOG"] = "info"
            env["FM_MATERIALIZER__DATA_DIR"] = str(data_dir)

            # Start the network
            cmd = [
                str(fendermint_bin), "materializer",
                "--data-dir", str(data_dir),
                "setup",
                "--manifest-file", str(config_path),
                "--validate"
            ]

            result = subprocess.run(cmd, capture_output=True, text=True, timeout=600, env=env)

            if result.returncode != 0:
                logger.error(f"Failed to start test network: {result.stderr}")
                return False

            logger.info("Test network started successfully")
            logger.info(f"Materializer output: {result.stdout}")

            # Store testnet ID for cleanup
            self.testnet_id = config_path.stem

            # Wait for network to be ready
            time.sleep(20)

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

    def cleanup_test_network(self):
        """Clean up the test network"""
        logger.info("Cleaning up test network")

        try:
            # Use fendermint materializer remove
            fendermint_bin = self.base_dir.parent.parent.parent.parent / "target" / "release" / "fendermint"

            # If we have a testnet ID, use it for cleanup
            if hasattr(self, 'testnet_id') and self.testnet_id:
                data_dir = self.results_dir / f"testnet_{self.testnet_id.split('_')[1]}"

                # Set environment variables
                env = os.environ.copy()
                env["RUST_LOG"] = "info"
                env["FM_MATERIALIZER__DATA_DIR"] = str(data_dir)

                cmd = [
                    str(fendermint_bin), "materializer",
                    "--data-dir", str(data_dir),
                    "remove",
                    "--testnet-id", self.testnet_id
                ]

                result = subprocess.run(cmd, capture_output=True, text=True, timeout=120, env=env)

                if result.returncode != 0:
                    logger.warning(f"Failed to remove testnet {self.testnet_id}: {result.stderr}")
                    logger.info(f"Attempting manual cleanup...")
                    # Try to remove manually
                    try:
                        if data_dir.exists():
                            shutil.rmtree(data_dir)
                            logger.info(f"Manually removed {data_dir}")
                    except Exception as e:
                        logger.warning(f"Failed to manually remove {data_dir}: {e}")
                else:
                    logger.info(f"Successfully removed testnet {self.testnet_id}")
            else:
                logger.info("No testnet ID found, trying manual cleanup")
                # Find and clean up data directories manually
                data_dirs = list(self.results_dir.glob("testnet_*"))

                for data_dir in data_dirs:
                    if data_dir.is_dir():
                        try:
                            shutil.rmtree(data_dir)
                            logger.info(f"Manually removed {data_dir}")
                        except Exception as e:
                            logger.warning(f"Failed to remove {data_dir}: {e}")

        except Exception as e:
            logger.error(f"Error during cleanup: {e}")

        logger.info("Cleanup completed")

    def run_throughput_benchmark(self) -> Optional[Dict]:
        """Run the throughput benchmark with real blockchain transactions"""
        logger.info("Starting REAL blockchain throughput benchmark")

        # Check if rust-script is available
        benchmark_script = self.base_dir / "simple_real_benchmark.rs"

        if not benchmark_script.exists():
            logger.error("Real blockchain benchmark script not found")
            logger.info("Falling back to standalone test")
            return self.run_standalone_benchmark()

        # Check if rust-script is installed
        try:
            subprocess.run(["rust-script", "--version"],
                         capture_output=True, text=True, timeout=5)
        except (subprocess.TimeoutExpired, FileNotFoundError):
            logger.warning("rust-script not found, installing...")
            try:
                subprocess.run(["cargo", "install", "rust-script"],
                             capture_output=True, text=True, timeout=300)
            except Exception as e:
                logger.error(f"Failed to install rust-script: {e}")
                logger.info("Falling back to standalone test")
                return self.run_standalone_benchmark()

        # Run the real blockchain benchmark
        result_file = self.results_dir / f"{self.test_name}_{self.timestamp}_results.json"

        # Get network endpoints
        endpoints = self.config.get('network', {}).get('endpoints', ['http://localhost:8545'])
        endpoint = endpoints[0] if endpoints else "http://localhost:8545"

        # Get test parameters from config
        target_tps = self.config.get('performance', {}).get('target_tps', 100)
        duration = self.config.get('performance', {}).get('duration', 30)
        concurrent_users = self.config.get('performance', {}).get('concurrent_users', 50)

        cmd = [
            "rust-script",
            str(benchmark_script),
            "--endpoint", endpoint,
            "--target-tps", str(target_tps),
            "--duration", str(duration),
            "--concurrent-users", str(concurrent_users),
            "--output", str(result_file)
        ]

        logger.info(f"Running REAL blockchain benchmark: {' '.join(cmd)}")
        logger.warning("This will send REAL transactions to the blockchain!")

        try:
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=1800)

            if result.returncode != 0:
                logger.error(f"Real blockchain benchmark failed: {result.stderr}")
                logger.info("Output from failed benchmark:")
                logger.info(result.stdout)
                logger.info("Falling back to standalone test")
                return self.run_standalone_benchmark()

            logger.info("Real blockchain benchmark completed successfully")
            logger.info(f"Benchmark output: {result.stdout}")

            # Load and return results
            try:
                with open(result_file) as f:
                    results = json.load(f)

                # Add metadata to indicate this was a real blockchain test
                results["test_type"] = "real_blockchain"
                results["framework"] = "rust_script_standalone"

                return results
            except Exception as e:
                logger.error(f"Failed to load results: {e}")
                return None

        except subprocess.TimeoutExpired:
            logger.error("Real blockchain benchmark timeout")
            return None
        except Exception as e:
            logger.error(f"Error running real blockchain benchmark: {e}")
            return None

    def run_standalone_benchmark(self) -> Optional[Dict]:
        """Run the standalone benchmark as fallback"""
        logger.info("Running standalone benchmark")

        # Build standalone test
        benchmark_bin = self.base_dir / "target" / "release" / "basic_throughput_test"

        if not benchmark_bin.exists():
            logger.info("Building standalone benchmark")
            result = subprocess.run([
                "rustc", "basic_throughput_test.rs", "-o", "target/release/basic_throughput_test"
            ], cwd=self.base_dir, capture_output=True, text=True)

            if result.returncode != 0:
                logger.error(f"Failed to build standalone benchmark: {result.stderr}")
                return None

        # Run standalone benchmark
        result_file = self.results_dir / f"{self.test_name}_{self.timestamp}_results.json"

        cmd = [str(benchmark_bin)]

        try:
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=1800)

            if result.returncode != 0:
                logger.error(f"Standalone benchmark failed: {result.stderr}")
                return None

            logger.info("Standalone benchmark completed successfully")
            logger.info(f"Output: {result.stdout}")

            # Create a simple results structure
            results = {
                "test_name": self.test_name,
                "timestamp": self.timestamp,
                "config": str(self.config),
                "success": True,
                "output": result.stdout,
                "error": result.stderr,
                "note": "This was a standalone simulation, not real blockchain transactions"
            }

            # Save results
            with open(result_file, 'w') as f:
                json.dump(results, f, indent=2)

            logger.info(f"Results saved to: {result_file}")
            return results

        except subprocess.TimeoutExpired:
            logger.error("Standalone benchmark timeout")
            return None
        except Exception as e:
            logger.error(f"Error running standalone benchmark: {e}")
            return None

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

                try:
                    results = self.run_throughput_benchmark()
                    if results:
                        self.generate_report(results)
                        return True
                    return False
                finally:
                    # Always cleanup
                    self.cleanup_test_network()

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