#!/bin/bash

# IPC Throughput Test Runner
# Simple interface for running throughput benchmarks

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BASE_DIR="$(dirname "$SCRIPT_DIR")"
CONFIGS_DIR="$BASE_DIR/configs"
RESULTS_DIR="$BASE_DIR/results"

# Create necessary directories
mkdir -p "$RESULTS_DIR"

# Function to print colored output
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check dependencies
check_dependencies() {
    local missing_deps=()

    # Check for required tools
    for tool in cargo python3 curl; do
        if ! command -v "$tool" &> /dev/null; then
            missing_deps+=("$tool")
        fi
    done

    # Check for Python dependencies
    if ! python3 -c "import yaml" &> /dev/null; then
        missing_deps+=("python3-yaml")
    fi

    if [ ${#missing_deps[@]} -ne 0 ]; then
        print_error "Missing dependencies: ${missing_deps[*]}"
        echo "Please install the missing dependencies and try again."
        exit 1
    fi

    print_success "All dependencies are available"
}

# Function to build the benchmarking tool
build_benchmark() {
    print_info "Building throughput benchmark tool..."

    cd "$BASE_DIR"
    if rustc basic_throughput_test.rs -o target/release/basic_throughput_test; then
        print_success "Benchmark tool built successfully"
    else
        print_error "Failed to build benchmark tool"
        exit 1
    fi
}

# Function to run a specific test
run_test() {
    local config_file="$1"
    local verbose="$2"

    if [ ! -f "$config_file" ]; then
        print_error "Configuration file not found: $config_file"
        return 1
    fi

    print_info "Running test: $(basename "$config_file")"

    local python_args=("$SCRIPT_DIR/run_throughput_tests.py" "$config_file")
    if [ "$verbose" == "true" ]; then
        python_args+=("--verbose")
    fi

    if python3 "${python_args[@]}"; then
        print_success "Test completed successfully"
        return 0
    else
        print_error "Test failed"
        return 1
    fi
}

# Function to list available configurations
list_configs() {
    print_info "Available test configurations:"

    if [ -d "$CONFIGS_DIR" ]; then
        for config in "$CONFIGS_DIR"/*.yaml; do
            if [ -f "$config" ]; then
                local name=$(basename "$config" .yaml)
                local desc=$(grep "description:" "$config" | head -1 | sed 's/description: *"\(.*\)"/\1/')
                echo "  • $name - $desc"
            fi
        done
    else
        print_warning "No configurations directory found"
    fi
}

# Function to create a sample configuration
create_sample_config() {
    local config_name="$1"
    local config_file="$CONFIGS_DIR/${config_name}.yaml"

    mkdir -p "$CONFIGS_DIR"

    cat > "$config_file" << EOF
name: "$config_name"
description: "Custom throughput test configuration"

# Network configuration
network:
  type: "single_subnet"
  validators: 4
  endpoints:
    - "http://localhost:8545"
    - "http://localhost:8546"
    - "http://localhost:8547"
    - "http://localhost:8548"

# Load configuration
load:
  pattern: "constant"
  target_tps: 500
  duration: "3m"
  ramp_up_duration: "30s"
  concurrent_connections: 50

# Transaction configuration
transactions:
  - type: "transfer"
    weight: 100
    amount: "1000000000000000000"  # 1 ETH in wei
    gas_limit: 21000

# Test configuration
test:
  warmup_duration: "30s"
  measurement_duration: "2m"
  cooldown_duration: "30s"
  max_retries: 3
  timeout: "30s"

# Metrics configuration
metrics:
  collection_interval: "1s"
  resource_monitoring: true
  detailed_latency: true
  percentiles: [50, 90, 95, 99]

# Output configuration
output:
  format: "json"
  file: "${config_name}_results.json"
  detailed: true
  charts: true
EOF

    print_success "Sample configuration created: $config_file"
    print_info "Edit this file to customize your test parameters"
}

# Function to show test results
show_results() {
    print_info "Recent test results:"

    if [ -d "$RESULTS_DIR" ]; then
        local count=0
        for result in "$RESULTS_DIR"/*_results.json; do
            if [ -f "$result" ]; then
                local basename=$(basename "$result")
                local timestamp=$(stat -c %Y "$result" 2>/dev/null || stat -f %m "$result" 2>/dev/null || echo "0")
                local date=$(date -d "@$timestamp" 2>/dev/null || date -r "$timestamp" 2>/dev/null || echo "unknown")
                echo "  • $basename - $date"
                count=$((count + 1))
            fi
        done

        if [ $count -eq 0 ]; then
            print_info "No test results found"
        fi
    else
        print_info "No results directory found"
    fi
}

# Function to clean up old results
cleanup_results() {
    local days="$1"

    if [ -z "$days" ]; then
        days=7
    fi

    print_info "Cleaning up results older than $days days..."

    if [ -d "$RESULTS_DIR" ]; then
        local count=0
        find "$RESULTS_DIR" -name "*_results.json" -mtime +$days -type f | while read -r file; do
            rm "$file"
            count=$((count + 1))
            print_info "Removed: $(basename "$file")"
        done

        find "$RESULTS_DIR" -name "*_report.md" -mtime +$days -type f | while read -r file; do
            rm "$file"
            print_info "Removed: $(basename "$file")"
        done

        print_success "Cleanup completed"
    fi
}

# Function to validate configuration
validate_config() {
    local config_file="$1"

    if [ ! -f "$config_file" ]; then
        print_error "Configuration file not found: $config_file"
        return 1
    fi

    print_info "Validating configuration: $(basename "$config_file")"

    # Check if it's valid YAML
    if ! python3 -c "import yaml; yaml.safe_load(open('$config_file'))" 2>/dev/null; then
        print_error "Invalid YAML syntax"
        return 1
    fi

    # Additional validation can be added here
    print_success "Configuration is valid"
    return 0
}

# Function to display usage
usage() {
    cat << EOF
Usage: $0 <command> [options]

Commands:
  run <config>           Run throughput test with specified configuration
  list                   List available test configurations
  create <name>          Create a sample configuration file
  results               Show recent test results
  cleanup [days]        Clean up old results (default: 7 days)
  validate <config>     Validate configuration file
  build                 Build the benchmark tool
  help                  Show this help message

Options:
  -v, --verbose         Enable verbose output
  -h, --help           Show this help message

Examples:
  $0 build                                    # Build the benchmark tool
  $0 run configs/basic_throughput.yaml       # Run basic throughput test
  $0 run configs/stress_test.yaml -v         # Run stress test with verbose output
  $0 list                                     # List available configurations
  $0 create my_test                           # Create sample configuration
  $0 results                                  # Show recent results
  $0 cleanup 14                               # Clean up results older than 14 days
  $0 validate configs/my_test.yaml           # Validate configuration

EOF
}

# Main script logic
main() {
    local command="$1"
    local verbose=false

    # Parse global options
    while [[ $# -gt 0 ]]; do
        case $1 in
            -v|--verbose)
                verbose=true
                shift
                ;;
            -h|--help)
                usage
                exit 0
                ;;
            *)
                break
                ;;
        esac
    done

    # Re-parse command after options
    command="$1"
    shift || true

    case "$command" in
        run)
            local config_file="$1"
            if [ -z "$config_file" ]; then
                print_error "Configuration file required for run command"
                usage
                exit 1
            fi

            check_dependencies
            build_benchmark
            run_test "$config_file" "$verbose"
            ;;
        list)
            list_configs
            ;;
        create)
            local config_name="$1"
            if [ -z "$config_name" ]; then
                print_error "Configuration name required for create command"
                usage
                exit 1
            fi

            create_sample_config "$config_name"
            ;;
        results)
            show_results
            ;;
        cleanup)
            local days="$1"
            cleanup_results "$days"
            ;;
        validate)
            local config_file="$1"
            if [ -z "$config_file" ]; then
                print_error "Configuration file required for validate command"
                usage
                exit 1
            fi

            validate_config "$config_file"
            ;;
        build)
            check_dependencies
            build_benchmark
            ;;
        help)
            usage
            ;;
        *)
            print_error "Unknown command: $command"
            usage
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@"